//! Deterministic validation projections.

use agent_core::{
    AgentError, AgentEvent, AgentEventKind, AgentRequest, JsonValue, ValidationCheck,
    ValidationReport, ValidationStatus, Validator,
};
use std::collections::BTreeMap;

/// A value-level validation boundary for recorded tool output or arguments.
pub trait ValueValidator {
    fn validate_value(&self, value: &JsonValue) -> ValidationReport;
}

/// Validates the P0 fixture schema vocabulary: `type`, `required`,
/// `properties`, and `const`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaValidator {
    pub name: String,
    pub schema: JsonValue,
}

impl SchemaValidator {
    pub fn new(name: impl Into<String>, schema: JsonValue) -> Self {
        Self {
            name: name.into(),
            schema,
        }
    }
}

impl ValueValidator for SchemaValidator {
    fn validate_value(&self, value: &JsonValue) -> ValidationReport {
        let mut failures = Vec::new();
        validate_schema(&self.name, &self.schema, value, &mut failures);
        if failures.is_empty() {
            ValidationReport {
                status: ValidationStatus::Passed,
                checks: vec![ValidationCheck {
                    name: self.name.clone(),
                    status: ValidationStatus::Passed,
                    detail: "schema_satisfied".into(),
                }],
            }
        } else {
            ValidationReport {
                status: ValidationStatus::Failed,
                checks: failures,
            }
        }
    }
}

pub fn validate_value(
    name: impl Into<String>,
    schema: &JsonValue,
    value: &JsonValue,
) -> ValidationReport {
    SchemaValidator::new(name, schema.clone()).validate_value(value)
}

fn validate_schema(
    path: &str,
    schema: &JsonValue,
    value: &JsonValue,
    failures: &mut Vec<ValidationCheck>,
) {
    let Some(schema_object) = as_object(schema) else {
        failures.push(failure(path, "schema_must_be_object"));
        return;
    };
    if let Some(expected_type) = schema_object.get("type").and_then(as_string) {
        if !matches_type(value, expected_type) {
            failures.push(failure(path, format!("expected_{expected_type}")));
            return;
        }
    }
    if let Some(expected) = schema_object.get("const") {
        if expected != value {
            failures.push(failure(path, "const_mismatch"));
        }
    }
    let Some(object) = as_object(value) else {
        return;
    };
    if let Some(required) = schema_object.get("required").and_then(as_array) {
        for field in required.iter().filter_map(as_string) {
            if !object.contains_key(field) {
                failures.push(failure(
                    &format!("{path}.{field}"),
                    "missing_required_field",
                ));
            }
        }
    }
    if let Some(properties) = schema_object.get("properties").and_then(as_object) {
        for (field, property_schema) in properties {
            if let Some(property_value) = object.get(field) {
                validate_schema(
                    &format!("{path}.{field}"),
                    property_schema,
                    property_value,
                    failures,
                );
            }
        }
    }
}

fn failure(path: &str, detail: impl Into<String>) -> ValidationCheck {
    ValidationCheck {
        name: path.into(),
        status: ValidationStatus::Failed,
        detail: detail.into(),
    }
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(object) => Some(object),
        _ => None,
    }
}

fn as_array(value: &JsonValue) -> Option<&[JsonValue]> {
    match value {
        JsonValue::Array(array) => Some(array),
        _ => None,
    }
}

fn as_string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value),
        _ => None,
    }
}

fn matches_type(value: &JsonValue, expected: &str) -> bool {
    match expected {
        "null" => matches!(value, JsonValue::Null),
        "boolean" => matches!(value, JsonValue::Bool(_)),
        "integer" | "number" => matches!(value, JsonValue::Number(_)),
        "string" => matches!(value, JsonValue::String(_)),
        "array" => matches!(value, JsonValue::Array(_)),
        "object" => matches!(value, JsonValue::Object(_)),
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub struct RequiredOutputValidator {
    required: String,
}

impl RequiredOutputValidator {
    pub fn new(required: impl Into<String>) -> Self {
        Self {
            required: required.into(),
        }
    }
}

impl Validator for RequiredOutputValidator {
    fn validate(
        &self,
        _request: &AgentRequest,
        events: &[AgentEvent],
    ) -> Result<ValidationReport, AgentError> {
        let output = events.iter().rev().find_map(|event| match &event.kind {
            AgentEventKind::ModelActionReceived {
                action: agent_core::ModelAction::Finish { output },
            } => Some(output.as_str()),
            _ => None,
        });
        let passed = output.is_some_and(|value| value.contains(&self.required));
        Ok(ValidationReport {
            status: if passed {
                ValidationStatus::Passed
            } else {
                ValidationStatus::Failed
            },
            checks: vec![ValidationCheck {
                name: "required-output".into(),
                status: if passed {
                    ValidationStatus::Passed
                } else {
                    ValidationStatus::Failed
                },
                detail: format!("output contains '{}'", self.required),
            }],
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct AlwaysPassValidator;

impl Validator for AlwaysPassValidator {
    fn validate(
        &self,
        _request: &AgentRequest,
        _events: &[AgentEvent],
    ) -> Result<ValidationReport, AgentError> {
        Ok(ValidationReport {
            status: ValidationStatus::Passed,
            checks: vec![ValidationCheck {
                name: "always-pass".into(),
                status: ValidationStatus::Passed,
                detail: "deterministic validation passed".into(),
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{
        AgentEvent, AgentEventKind, AgentRequest, EventEnvelope, EventId, ModelAction, RunId,
        RunLimits, SessionId,
    };

    #[test]
    fn output_validator_passes_and_fails() {
        let request = AgentRequest {
            run_id: RunId("r".into()),
            session_id: SessionId("s".into()),
            instruction: "x".into(),
            initial_context: Vec::new(),
            deterministic_seed: 0,
            limits: RunLimits {
                max_steps: 1,
                max_tool_calls: 0,
                max_context_items: 1,
            },
        };
        let event = AgentEvent {
            envelope: EventEnvelope {
                schema_version: 1,
                event_id: EventId("e".into()),
                sequence: agent_core::Sequence(0),
                run_id: request.run_id.clone(),
                session_id: request.session_id.clone(),
            },
            kind: AgentEventKind::ModelActionReceived {
                action: ModelAction::Finish {
                    output: "hello".into(),
                },
            },
        };
        assert_eq!(
            RequiredOutputValidator::new("hell")
                .validate(&request, std::slice::from_ref(&event))
                .expect("report")
                .status,
            ValidationStatus::Passed
        );
        assert_eq!(
            RequiredOutputValidator::new("no")
                .validate(&request, &[event])
                .expect("report")
                .status,
            ValidationStatus::Failed
        );
    }

    fn profile_schema() -> JsonValue {
        JsonValue::object([
            ("type".into(), JsonValue::String("object".into())),
            (
                "required".into(),
                JsonValue::Array(vec![
                    JsonValue::String("profile_id".into()),
                    JsonValue::String("name".into()),
                ]),
            ),
            (
                "properties".into(),
                JsonValue::object([
                    (
                        "profile_id".into(),
                        JsonValue::object([("type".into(), JsonValue::String("string".into()))]),
                    ),
                    (
                        "name".into(),
                        JsonValue::object([("type".into(), JsonValue::String("string".into()))]),
                    ),
                ]),
            ),
        ])
    }

    #[test]
    fn schema_validator_passes_for_a_matching_value() {
        let validator = SchemaValidator::new("profile_shape", profile_schema());
        let value = JsonValue::object([
            ("profile_id".into(), JsonValue::String("P-7".into())),
            ("name".into(), JsonValue::String("Ada".into())),
        ]);
        let report = validator.validate_value(&value);
        assert_eq!(report.status, ValidationStatus::Passed);
        assert!(
            report
                .checks
                .iter()
                .all(|check| { check.status == ValidationStatus::Passed })
        );
    }

    #[test]
    fn schema_validator_reports_a_structured_field_failure() {
        let validator = SchemaValidator::new("profile_shape", profile_schema());
        let value = JsonValue::object([
            ("profile_id".into(), JsonValue::String("P-7".into())),
            ("name".into(), JsonValue::Number(17)),
        ]);
        let report = validator.validate_value(&value);
        assert_eq!(report.status, ValidationStatus::Failed);
        assert_eq!(report.checks[0].name, "profile_shape.name");
        assert_eq!(report.checks[0].detail, "expected_string");
    }
}

//! Pure context construction for the deterministic P0 profile.

use agent_core::{
    AgentError, AgentEvent, AgentEventKind, AgentRequest, ContextBuilder as CoreContextBuilder,
    JsonValue, ModelAction, ModelInput, ModelMessage, ModelRole, RunLimits, ToolResultStatus,
    ToolSpec,
};

/// The default bounded context builder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimpleContextBuilder {
    tools: Vec<ToolSpec>,
}

pub type ContextBuilder = SimpleContextBuilder;
pub type DeterministicContextBuilder = SimpleContextBuilder;
pub type ContextEngine = SimpleContextBuilder;

impl SimpleContextBuilder {
    pub fn new(tools: Vec<ToolSpec>) -> Self {
        Self { tools }
    }

    pub fn build(
        &self,
        request: &AgentRequest,
        history: &[AgentEvent],
        limits: &RunLimits,
    ) -> Result<ModelInput, AgentError> {
        <Self as CoreContextBuilder>::build(self, request, history, limits)
    }

    fn validate_history(request: &AgentRequest, history: &[AgentEvent]) -> Result<(), AgentError> {
        let mut previous_sequence = None;
        for event in history {
            if event.envelope.run_id != request.run_id
                || event.envelope.session_id != request.session_id
            {
                return Err(AgentError::ContextFailure {
                    detail: "event history contains an event from another run or session".into(),
                });
            }
            if let Some(previous) = previous_sequence
                && event.envelope.sequence <= previous
            {
                return Err(AgentError::ContextFailure {
                    detail: "event history must have strictly increasing sequences".into(),
                });
            }
            previous_sequence = Some(event.envelope.sequence);
        }
        Ok(())
    }

    fn history_messages(history: &[AgentEvent]) -> impl Iterator<Item = ModelMessage> + '_ {
        history.iter().filter_map(|event| match &event.kind {
            AgentEventKind::ToolFinished { result } => Some(ModelMessage {
                role: ModelRole::Tool,
                content: format_tool_result(result),
            }),
            AgentEventKind::ModelActionReceived { action } => Some(ModelMessage {
                role: ModelRole::System,
                content: format_action(action),
            }),
            AgentEventKind::OutcomeRecorded { outcome } => Some(ModelMessage {
                role: ModelRole::System,
                content: format!("outcome: {outcome:?}"),
            }),
            _ => None,
        })
    }
}

impl CoreContextBuilder for SimpleContextBuilder {
    fn build(
        &self,
        request: &AgentRequest,
        history: &[AgentEvent],
        limits: &RunLimits,
    ) -> Result<ModelInput, AgentError> {
        Self::validate_history(request, history)?;

        let history_messages: Vec<ModelMessage> = Self::history_messages(history).collect();
        let context_count = request
            .initial_context
            .len()
            .saturating_add(1)
            .saturating_add(history_messages.len());
        let limit = limits.max_context_items as usize;
        if context_count > limit {
            return Err(AgentError::ContextFailure {
                detail: format!(
                    "context has {context_count} items, limit is {limit}; required data was not dropped"
                ),
            });
        }

        let accepted_steps = history
            .iter()
            .filter(|event| matches!(event.kind, AgentEventKind::ModelActionReceived { .. }))
            .count() as u32;
        let accepted_tool_calls = history
            .iter()
            .filter(|event| {
                matches!(
                    event.kind,
                    AgentEventKind::ModelActionReceived {
                        action: ModelAction::CallTool(_)
                    }
                )
            })
            .count() as u32;

        let mut messages = Vec::with_capacity(context_count);
        messages.extend(request.initial_context.iter().map(|item| ModelMessage {
            role: ModelRole::System,
            content: format!("{}: {}", item.name, item.content),
        }));
        messages.push(ModelMessage {
            role: ModelRole::User,
            content: request.instruction.clone(),
        });
        messages.extend(history_messages);

        Ok(ModelInput {
            run_id: request.run_id.clone(),
            instruction: request.instruction.clone(),
            messages,
            available_tools: self.tools.clone(),
            remaining_steps: limits.max_steps.saturating_sub(accepted_steps),
            remaining_tool_calls: limits.max_tool_calls.saturating_sub(accepted_tool_calls),
        })
    }
}

pub fn echo_tool_spec() -> ToolSpec {
    let mut object = std::collections::BTreeMap::new();
    object.insert("type".into(), JsonValue::from("object"));
    ToolSpec {
        tool_id: "echo".into(),
        description: "Returns the supplied text".into(),
        input_schema: JsonValue::Object(object),
    }
}

fn format_action(action: &ModelAction) -> String {
    match action {
        ModelAction::Finish { output } => format!("action: finish {output}"),
        ModelAction::CallTool(call) => format!(
            "action: call_tool {} ({}) arguments={}",
            call.tool_id,
            call.call_id,
            format_json(&call.arguments)
        ),
    }
}

fn format_tool_result(result: &agent_core::ToolResult) -> String {
    let status = match result.status {
        ToolResultStatus::Succeeded => "succeeded",
        ToolResultStatus::Failed => "failed",
    };
    let output = result
        .output
        .as_ref()
        .map(format_json)
        .unwrap_or_else(|| "null".into());
    let error = result.error.as_deref().unwrap_or("none");
    format!(
        "tool_result {} {status} output={output} error={error}",
        result.call_id
    )
}

fn format_json(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".into(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => format!("\"{}\"", escape(value)),
        JsonValue::Array(values) => format!(
            "[{}]",
            values.iter().map(format_json).collect::<Vec<_>>().join(",")
        ),
        JsonValue::Object(values) => {
            let body = values
                .iter()
                .map(|(key, value)| format!("\"{}\":{}", escape(key), format_json(value)))
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{body}}}")
        }
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{ContextItem, EventEnvelope, EventId, RunId, SessionId};

    fn request(max_context_items: u32) -> AgentRequest {
        AgentRequest {
            run_id: RunId("r".into()),
            session_id: SessionId("s".into()),
            instruction: "go".into(),
            initial_context: vec![ContextItem {
                name: "a".into(),
                content: "b".into(),
            }],
            deterministic_seed: 1,
            limits: RunLimits {
                max_steps: 2,
                max_tool_calls: 1,
                max_context_items,
            },
        }
    }

    fn event(request: &AgentRequest, sequence: u64, kind: AgentEventKind) -> AgentEvent {
        AgentEvent {
            envelope: EventEnvelope {
                schema_version: 1,
                event_id: EventId::for_sequence(&request.run_id, sequence),
                sequence: agent_core::Sequence(sequence),
                run_id: request.run_id.clone(),
                session_id: request.session_id.clone(),
            },
            kind,
        }
    }

    #[test]
    fn order_and_budget_are_explicit() {
        let req = request(5);
        let history = vec![event(
            &req,
            0,
            AgentEventKind::ModelActionReceived {
                action: ModelAction::CallTool(agent_core::ToolCall {
                    call_id: "c1".into(),
                    tool_id: "echo".into(),
                    arguments: JsonValue::String("value".into()),
                }),
            },
        )];
        let input = SimpleContextBuilder::new(vec![echo_tool_spec()])
            .build(&req, &history, &req.limits)
            .expect("context fits");
        assert_eq!(input.messages[0].content, "a: b");
        assert_eq!(input.messages[1].content, "go");
        assert!(input.messages[2].content.contains("action: call_tool echo"));
        assert_eq!(input.available_tools[0].tool_id, "echo");
        assert_eq!(input.remaining_steps, 1);
        assert_eq!(input.remaining_tool_calls, 0);
    }

    #[test]
    fn tool_result_is_after_prior_action_and_json_is_stable() {
        let req = request(6);
        let result = agent_core::ToolResult {
            call_id: "c1".into(),
            status: ToolResultStatus::Succeeded,
            output: Some(JsonValue::Object(
                [
                    ("b".into(), JsonValue::Number(2)),
                    ("a".into(), JsonValue::Number(1)),
                ]
                .into_iter()
                .collect(),
            )),
            error: None,
        };
        let history = vec![
            event(
                &req,
                0,
                AgentEventKind::ModelActionReceived {
                    action: ModelAction::CallTool(agent_core::ToolCall {
                        call_id: "c1".into(),
                        tool_id: "echo".into(),
                        arguments: JsonValue::Null,
                    }),
                },
            ),
            event(&req, 1, AgentEventKind::ToolFinished { result }),
        ];
        let input = SimpleContextBuilder::default()
            .build(&req, &history, &req.limits)
            .expect("context fits");
        assert!(input.messages[2].content.contains("action: call_tool"));
        assert!(input.messages[3].content.contains("\"a\":1,\"b\":2"));
    }

    #[test]
    fn overflow_is_a_structured_error_without_dropping_data() {
        let req = request(2);
        let history = vec![event(
            &req,
            0,
            AgentEventKind::ModelActionReceived {
                action: ModelAction::Finish {
                    output: "done".into(),
                },
            },
        )];
        assert_eq!(
            SimpleContextBuilder::default().build(&req, &history, &req.limits),
            Err(AgentError::ContextFailure {
                detail: "context has 3 items, limit is 2; required data was not dropped".into()
            })
        );
    }

    #[test]
    fn out_of_order_history_is_rejected() {
        let req = request(5);
        let history = vec![
            event(
                &req,
                1,
                AgentEventKind::ModelActionReceived {
                    action: ModelAction::Finish {
                        output: "done".into(),
                    },
                },
            ),
            event(
                &req,
                0,
                AgentEventKind::OutcomeRecorded {
                    outcome: agent_core::RunOutcome::Terminated {
                        reason: agent_core::TerminationReason::BudgetExhausted,
                    },
                },
            ),
        ];
        assert!(matches!(
            SimpleContextBuilder::default().build(&req, &history, &req.limits),
            Err(AgentError::ContextFailure { .. })
        ));
    }

    #[test]
    fn repeated_builds_are_deterministic() {
        let req = request(5);
        let history = vec![event(
            &req,
            0,
            AgentEventKind::ModelActionReceived {
                action: ModelAction::Finish {
                    output: "done".into(),
                },
            },
        )];
        let builder = SimpleContextBuilder::new(vec![echo_tool_spec()]);
        let first = builder
            .build(&req, &history, &req.limits)
            .expect("context fits");
        let second = builder
            .build(&req, &history, &req.limits)
            .expect("context fits");
        assert_eq!(first, second);
    }
}

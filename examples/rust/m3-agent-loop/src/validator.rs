//! M3: minimal validator.
//!
//! `Finish` is only the model's intent, not a result. This substring check
//! stands in for "the result passed an acceptance check" — a real
//! implementation would check schema, required fields, or business rules,
//! but the control-flow point is the same: failing it must not become
//! `Completed`.

pub struct RequiredOutputValidator {
    required: String,
}

impl RequiredOutputValidator {
    pub fn new(required: impl Into<String>) -> Self {
        Self {
            required: required.into(),
        }
    }

    pub fn validate(&self, output: &str) -> bool {
        output.contains(&self.required)
    }
}

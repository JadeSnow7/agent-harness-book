//! Deterministic model adapters used by the P0 examples and tests.

use agent_core::{AgentError, ModelAction, ModelInput, ModelProvider, ToolCall};

/// One item in a [`ScriptedMockModel`] script.
///
/// `Invalid` is deliberately not a [`ModelAction`]. It allows tests to exercise
/// the model-output validation boundary without making the runner accept a
/// malformed action as valid core data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptedAction {
    CallTool(ToolCall),
    Finish { output: String },
    Invalid { detail: String },
}

impl From<ModelAction> for ScriptedAction {
    fn from(action: ModelAction) -> Self {
        match action {
            ModelAction::CallTool(call) => Self::CallTool(call),
            ModelAction::Finish { output } => Self::Finish { output },
        }
    }
}

impl ScriptedAction {
    pub fn call_tool(
        call_id: impl Into<String>,
        tool_id: impl Into<String>,
        arguments: agent_core::JsonValue,
    ) -> Self {
        Self::CallTool(ToolCall {
            call_id: call_id.into(),
            tool_id: tool_id.into(),
            arguments,
        })
    }

    pub fn finish(output: impl Into<String>) -> Self {
        Self::Finish {
            output: output.into(),
        }
    }

    pub fn invalid(detail: impl Into<String>) -> Self {
        Self::Invalid {
            detail: detail.into(),
        }
    }
}

/// A stateful provider that returns one deterministic scripted action per call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptedMockModel {
    script: Vec<ScriptedAction>,
    cursor: usize,
    call_count: usize,
    last_input: Option<ModelInput>,
}

impl ScriptedMockModel {
    pub fn new<T>(script: Vec<T>) -> Self
    where
        T: Into<ScriptedAction>,
    {
        Self {
            script: script.into_iter().map(Into::into).collect(),
            cursor: 0,
            call_count: 0,
            last_input: None,
        }
    }

    pub fn script(&self) -> &[ScriptedAction] {
        &self.script
    }

    pub fn calls(&self) -> usize {
        self.call_count
    }

    pub fn call_count(&self) -> usize {
        self.call_count
    }

    pub fn next_index(&self) -> usize {
        self.cursor
    }

    pub fn remaining(&self) -> usize {
        self.script.len().saturating_sub(self.cursor)
    }

    pub fn remaining_actions(&self) -> usize {
        self.remaining()
    }

    pub fn is_exhausted(&self) -> bool {
        self.cursor >= self.script.len()
    }

    pub fn last_input(&self) -> Option<&ModelInput> {
        self.last_input.as_ref()
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.call_count = 0;
        self.last_input = None;
    }
}

impl ModelProvider for ScriptedMockModel {
    fn next_action(&mut self, input: ModelInput) -> Result<ModelAction, AgentError> {
        self.call_count += 1;
        self.last_input = Some(input);
        let action =
            self.script
                .get(self.cursor)
                .cloned()
                .ok_or_else(|| AgentError::ModelFailure {
                    detail: format!("script exhausted after {} model calls", self.call_count),
                })?;
        self.cursor += 1;
        match action {
            ScriptedAction::CallTool(call) => Ok(ModelAction::CallTool(call)),
            ScriptedAction::Finish { output } => Ok(ModelAction::Finish { output }),
            ScriptedAction::Invalid { detail } => Err(AgentError::InvalidModelAction { detail }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{JsonValue, RunId};

    fn input() -> ModelInput {
        ModelInput {
            run_id: RunId("r".into()),
            instruction: "x".into(),
            messages: Vec::new(),
            available_tools: Vec::new(),
            remaining_steps: 1,
            remaining_tool_calls: 1,
        }
    }

    #[test]
    fn plain_response_returns_finish_action() {
        let mut model = ScriptedMockModel::new(vec![ScriptedAction::finish("hello")]);

        assert_eq!(
            model.next_action(input()),
            Ok(ModelAction::Finish {
                output: "hello".into()
            })
        );
        assert_eq!(model.call_count(), 1);
        assert_eq!(model.remaining_actions(), 0);
    }

    #[test]
    fn tool_action_preserves_call_shape() {
        let call = ToolCall {
            call_id: "call-1".into(),
            tool_id: "echo".into(),
            arguments: JsonValue::String("value".into()),
        };
        let mut model = ScriptedMockModel::new(vec![ScriptedAction::CallTool(call.clone())]);

        assert_eq!(model.next_action(input()), Ok(ModelAction::CallTool(call)));
    }

    #[test]
    fn invalid_action_is_explicit() {
        let mut model =
            ScriptedMockModel::new(vec![ScriptedAction::invalid("arguments must be an object")]);

        assert_eq!(
            model.next_action(input()),
            Err(AgentError::InvalidModelAction {
                detail: "arguments must be an object".into()
            })
        );
        assert!(model.is_exhausted());
    }

    #[test]
    fn exhaustion_is_structured_and_counted() {
        let mut model = ScriptedMockModel::new(Vec::<ModelAction>::new());

        assert_eq!(
            model.next_action(input()),
            Err(AgentError::ModelFailure {
                detail: "script exhausted after 1 model calls".into()
            })
        );
        assert_eq!(model.call_count(), 1);
        assert_eq!(model.next_index(), 0);
        assert!(model.is_exhausted());
    }

    #[test]
    fn repeated_fresh_models_are_deterministic() {
        let script = vec![
            ScriptedAction::call_tool("call-1", "echo", JsonValue::Number(7)),
            ScriptedAction::finish("done"),
        ];
        let mut first = ScriptedMockModel::new(script.clone());
        let mut second = ScriptedMockModel::new(script);

        let first_actions = vec![first.next_action(input()), first.next_action(input())];
        let second_actions = vec![second.next_action(input()), second.next_action(input())];
        assert_eq!(first_actions, second_actions);
        assert_eq!(first, second);
    }
}

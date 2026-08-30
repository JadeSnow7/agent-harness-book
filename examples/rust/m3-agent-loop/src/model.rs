//! M3: scripted model, for deterministic tests.
//!
//! Independent of `model-adapters::ScriptedMockModel` (that one implements
//! `agent_core::ModelProvider`, tied to the rich P0 `ModelInput` shape).
//! M3's loop only needs "the tool results seen so far" as input, so this
//! model works against the small [`HistoryItem`](crate::types::HistoryItem)
//! list instead.

use crate::types::{HistoryItem, ModelAction};

/// A stateful, deterministic replay of a predetermined action sequence.
pub struct ScriptedMockModel {
    script: Vec<ModelAction>,
    cursor: usize,
    received_inputs: Vec<Vec<HistoryItem>>,
}

/// Minimal model interface: given the history so far, produce the next
/// action. Exists so `run_loop` isn't hard-coded against one concrete
/// model type, while staying independent of `agent_core::ModelProvider`.
pub trait Model {
    fn next_action(&mut self, history: &[HistoryItem]) -> ModelAction;
}

impl ScriptedMockModel {
    pub fn new(script: Vec<ModelAction>) -> Self {
        Self {
            script,
            cursor: 0,
            received_inputs: Vec::new(),
        }
    }

    /// Number of times the model has been called — used to assert that a
    /// budget/cancel boundary stopped the loop before one extra call.
    pub fn call_count(&self) -> usize {
        self.received_inputs.len()
    }

    /// Every history snapshot the model was called with, in order —
    /// used to assert that a tool result from turn N is visible on turn
    /// N+1's input.
    pub fn received_inputs(&self) -> &[Vec<HistoryItem>] {
        &self.received_inputs
    }
}

impl Model for ScriptedMockModel {
    fn next_action(&mut self, history: &[HistoryItem]) -> ModelAction {
        self.received_inputs.push(history.to_vec());
        let action = self.script.get(self.cursor).cloned().unwrap_or_else(|| {
            panic!(
                "model called a {}-th time but script only has {} scripted actions",
                self.received_inputs.len(),
                self.script.len()
            )
        });
        self.cursor += 1;
        action
    }
}

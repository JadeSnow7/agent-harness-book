//! M3: minimal Agent Loop types.
//!
//! These are independent of P0's `DeterministicRunner` composition. Only
//! plain data types are reused from `agent_core` — `ModelAction`,
//! `ToolCall`, `ToolResult`, `ToolResultStatus`, `JsonValue` — the same
//! reuse boundary the Python side draws against `m2-tool-runtime`'s
//! `tool_types`. The orchestration-coupled pieces (`DeterministicRunner`,
//! `ModelProvider`, `ContextBuilder`, `Validator`, `EventSink`/`AgentEvent`)
//! are deliberately NOT reused; see `book/src/ch5.md` §5.6.

pub use agent_core::{ModelAction, ToolCall, ToolResult, ToolResultStatus};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Five mutually exclusive terminal outcomes.
///
/// Mutual exclusivity is structural: [`RunResult::outcome`] is a single
/// field, and every terminating branch in `run_loop` returns exactly once.
/// There is no code path that reports two outcomes for the same run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {
    Completed,
    Failed,
    BudgetExhausted,
    PolicyDenied,
    Cancelled,
}

/// Predetermined step budget. M3 only models step count; token/time/cost
/// budgets are later milestones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunLimits {
    pub max_steps: u32,
}

/// One entry the loop appends to the model-visible history after a tool
/// call resolves (success or failure).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryItem {
    pub call_id: String,
    pub tool_id: String,
    pub status: ToolResultStatus,
    pub output: Option<agent_core::JsonValue>,
    pub error: Option<String>,
}

impl From<&ToolResult> for HistoryItem {
    fn from(result: &ToolResult) -> Self {
        Self {
            call_id: result.call_id.clone(),
            tool_id: String::new(), // filled in by the loop, which knows the call's tool_id
            status: result.status.clone(),
            output: result.output.clone(),
            error: result.error.clone(),
        }
    }
}

/// One observable event inside a run — used by tests to assert on internal
/// behavior (e.g. "a tool failure was recorded but did not terminate the
/// run") without depending on `agent_core`'s P0 event envelope machinery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    ModelInput(Vec<HistoryItem>),
    ModelAction(ModelAction),
    ToolResult(ToolResult),
    InvalidAction { reason: &'static str, detail: String },
    PolicyDenied { tool_id: String },
    BudgetExhausted { max_steps: u32 },
    Cancelled,
    ValidationPassed(String),
    ValidationFailed(String),
}

/// The final result of one run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub outcome: Outcome,
    pub output: Option<String>,
    pub reason: Option<&'static str>,
    pub events: Vec<Event>,
    pub model_call_count: u32,
}

/// Cooperative cancellation flag: the loop checks this at the top of each
/// turn, before the next model call — it does not pre-empt a tool that is
/// already executing.
#[derive(Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation. Safe to call from inside a `Tool::execute`
    /// implementation to simulate a mid-run cancellation signal.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

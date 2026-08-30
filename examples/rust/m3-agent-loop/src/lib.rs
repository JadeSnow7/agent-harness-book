//! M3: minimal Agent Loop — Rust counterpart to
//! `examples/python/m3-agent-loop`. See `run::run_loop` for the control
//! flow and `book/src/ch5.md` §5.6 for the contract both implementations
//! satisfy.

pub mod model;
pub mod policy;
pub mod run;
pub mod tools;
pub mod types;
pub mod validator;

pub use model::{Model, ScriptedMockModel};
pub use policy::AllowListPolicy;
pub use run::run_loop;
pub use tools::{CancellingTool, EchoTool, FailingTool};
pub use types::{
    CancelToken, Event, HistoryItem, ModelAction, Outcome, RunLimits, RunResult, ToolCall,
    ToolResult, ToolResultStatus,
};
pub use validator::RequiredOutputValidator;

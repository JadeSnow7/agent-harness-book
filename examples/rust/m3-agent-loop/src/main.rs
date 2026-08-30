//! M3 demo: scripted model calls `echo`, then `Finish`; prints the outcome
//! and event count. Mirrors `examples/python/m3-agent-loop/demo.py` (run
//! `python3 examples/python/m3-agent-loop/demo.py`) so the two sides can be
//! compared directly.

use std::collections::BTreeMap;

use agent_core::JsonValue;
use m3_agent_loop::{
    AllowListPolicy, CancelToken, EchoTool, ModelAction, Outcome, RequiredOutputValidator,
    RunLimits, ScriptedMockModel, ToolCall, run_loop,
};
use tool_runtime::ToolRegistry;

fn main() {
    let mut registry = ToolRegistry::default();
    registry.register(EchoTool);

    let mut arguments = BTreeMap::new();
    arguments.insert("value".into(), JsonValue::from(7_i64));
    let script = vec![
        ModelAction::CallTool(ToolCall {
            call_id: "call-1".into(),
            tool_id: "echo".into(),
            arguments: JsonValue::Object(arguments),
        }),
        ModelAction::Finish {
            output: "echo completed".into(),
        },
    ];
    let mut model = ScriptedMockModel::new(script);

    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 3 },
        &CancelToken::new(),
    );

    println!("outcome={:?}", result.outcome);
    println!("event_count={}", result.events.len());
    println!("model_call_count={}", result.model_call_count);
    std::process::exit(if result.outcome == Outcome::Completed {
        0
    } else {
        1
    });
}

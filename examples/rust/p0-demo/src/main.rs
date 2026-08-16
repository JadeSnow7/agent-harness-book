use agent_core::{
    AgentRequest, DeterministicRunner, EventLog, ModelAction, RunId, RunLimits, SessionId, ToolCall,
};
use context_engine::SimpleContextBuilder;
use model_adapters::ScriptedMockModel;
use observability::summarize;
use policy_engine::AllowListPolicy;
use session_store::InMemoryEventStore;
use std::collections::BTreeMap;
use tool_runtime::{EchoTool, ToolRegistry};
use validators::RequiredOutputValidator;

fn main() -> Result<(), agent_core::AgentError> {
    let run_id = RunId("demo-run".into());
    let session_id = SessionId("demo-session".into());
    let mut arguments = BTreeMap::new();
    arguments.insert("value".into(), agent_core::JsonValue::from(7_i64));
    let script = vec![
        ModelAction::CallTool(ToolCall {
            call_id: "call-1".into(),
            tool_id: "echo".into(),
            arguments: agent_core::JsonValue::Object(arguments),
        }),
        ModelAction::Finish {
            output: "echo completed".into(),
        },
    ];

    let mut tools = ToolRegistry::default();
    tools.register(EchoTool);
    let context = SimpleContextBuilder::new(tools.specs());
    let mut runner = DeterministicRunner::new(
        ScriptedMockModel::new(script),
        context,
        AllowListPolicy::new(["echo"]),
        tools,
        RequiredOutputValidator::new("completed"),
        InMemoryEventStore::default(),
    );
    let outcome = runner.run(AgentRequest {
        run_id: run_id.clone(),
        session_id: session_id.clone(),
        instruction: "Use echo and report completion".into(),
        initial_context: Vec::new(),
        deterministic_seed: 42,
        limits: RunLimits {
            max_steps: 3,
            max_tool_calls: 1,
            max_context_items: 8,
        },
    })?;
    let events = runner.sink().read_run(&session_id, &run_id)?;
    let summary = summarize(&events);
    println!("outcome={outcome:?}");
    println!("event_count={}", summary.event_count);
    println!("evidence_count={}", summary.evidence.len());
    Ok(())
}

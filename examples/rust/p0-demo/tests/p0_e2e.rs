use agent_core::{
    AgentEventKind, AgentRequest, DeterministicRunner, EventLog, JsonValue, RunId, RunLimits,
    RunOutcome, SessionId, ToolCall,
};
use context_engine::SimpleContextBuilder;
use model_adapters::{ScriptedAction, ScriptedMockModel};
use policy_engine::AllowListPolicy;
use session_store::InMemoryEventStore;
use std::collections::BTreeMap;
use tool_runtime::{EchoTool, ToolRegistry};
use validators::RequiredOutputValidator;

fn request(run: &str) -> AgentRequest {
    AgentRequest {
        run_id: RunId(run.into()),
        session_id: SessionId("e2e-session".into()),
        instruction: "perform deterministic action".into(),
        initial_context: Vec::new(),
        deterministic_seed: 9,
        limits: RunLimits {
            max_steps: 3,
            max_tool_calls: 1,
            max_context_items: 8,
        },
    }
}

fn call() -> ToolCall {
    let mut args = BTreeMap::new();
    args.insert("value".into(), JsonValue::from(7_i64));
    ToolCall {
        call_id: "call-1".into(),
        tool_id: "echo".into(),
        arguments: JsonValue::Object(args),
    }
}

fn runner(
    actions: Vec<ScriptedAction>,
    allowed: &[&str],
) -> DeterministicRunner<
    ScriptedMockModel,
    SimpleContextBuilder,
    AllowListPolicy,
    ToolRegistry,
    RequiredOutputValidator,
    InMemoryEventStore,
> {
    let mut tools = ToolRegistry::default();
    tools.register(EchoTool);
    DeterministicRunner::new(
        ScriptedMockModel::new(actions),
        SimpleContextBuilder::new(tools.specs()),
        AllowListPolicy::new(allowed.iter().copied()),
        tools,
        RequiredOutputValidator::new("done"),
        InMemoryEventStore::default(),
    )
}

#[test]
fn normal_tool_path_records_policy_execution_validation_and_evidence() {
    let mut runner = runner(
        vec![
            ScriptedAction::CallTool(call()),
            ScriptedAction::finish("done"),
        ],
        &["echo"],
    );
    let outcome = runner.run(request("normal")).expect("run should complete");
    assert!(matches!(outcome, RunOutcome::Completed { .. }));
    let events = runner
        .sink()
        .read_run(&SessionId("e2e-session".into()), &RunId("normal".into()))
        .expect("events");
    let kinds: Vec<&str> = events
        .iter()
        .map(|event| match event.kind {
            AgentEventKind::RunStarted { .. } => "start",
            AgentEventKind::ModelInputBuilt { .. } => "input",
            AgentEventKind::ModelActionReceived { .. } => "action",
            AgentEventKind::PolicyEvaluated { .. } => "policy",
            AgentEventKind::ToolStarted { .. } => "tool-start",
            AgentEventKind::ToolFinished { .. } => "tool-finish",
            AgentEventKind::ValidationCompleted { .. } => "validation",
            AgentEventKind::EvidenceRecorded { .. } => "evidence",
            AgentEventKind::OutcomeRecorded { .. } => "outcome",
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            "start",
            "input",
            "action",
            "policy",
            "tool-start",
            "tool-finish",
            "input",
            "action",
            "validation",
            "evidence",
            "outcome"
        ]
    );
}

#[test]
fn policy_denial_is_terminal_and_has_no_tool_start() {
    let mut runner = runner(vec![ScriptedAction::CallTool(call())], &[]);
    let outcome = runner.run(request("denied")).expect("denial is structured");
    assert!(matches!(outcome, RunOutcome::Terminated { .. }));
    let events = runner
        .sink()
        .read_run(&SessionId("e2e-session".into()), &RunId("denied".into()))
        .expect("events");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, AgentEventKind::ToolStarted { .. }))
    );
}

#[test]
fn repeated_runs_are_semantically_equal() {
    let actions = vec![
        ScriptedAction::CallTool(call()),
        ScriptedAction::finish("done"),
    ];
    let mut first = runner(actions.clone(), &["echo"]);
    let mut second = runner(actions, &["echo"]);
    let first_outcome = first.run(request("same")).expect("first");
    let second_outcome = second.run(request("same")).expect("second");
    assert_eq!(first_outcome, second_outcome);
    assert_eq!(
        first
            .sink()
            .read_run(&SessionId("e2e-session".into()), &RunId("same".into()))
            .expect("first events"),
        second
            .sink()
            .read_run(&SessionId("e2e-session".into()), &RunId("same".into()))
            .expect("second events")
    );
    let replayed = first
        .sink()
        .read_run(&SessionId("e2e-session".into()), &RunId("same".into()))
        .expect("replay events");
    InMemoryEventStore::validate_run(&replayed).expect("event sequence is replayable");
}

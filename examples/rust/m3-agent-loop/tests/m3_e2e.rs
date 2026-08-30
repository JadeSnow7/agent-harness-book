//! M3 Agent Loop offline tests — Rust counterpart to
//! `examples/python/m3-agent-loop/test_loop.py`, item-for-item. See
//! `evals/m3-test-matrix.md` for the explicit Python/Rust correspondence.

use std::collections::BTreeMap;

use agent_core::JsonValue;
use m3_agent_loop::{
    AllowListPolicy, CancelToken, CancellingTool, EchoTool, Event, FailingTool, ModelAction,
    Outcome, RequiredOutputValidator, RunLimits, RunResult, ScriptedMockModel, ToolCall,
    ToolResultStatus, run_loop,
};
use tool_runtime::ToolRegistry;

fn echo_call(call_id: &str, value: i64) -> ModelAction {
    let mut arguments = BTreeMap::new();
    arguments.insert("value".into(), JsonValue::from(value));
    ModelAction::CallTool(ToolCall {
        call_id: call_id.into(),
        tool_id: "echo".into(),
        arguments: JsonValue::Object(arguments),
    })
}

fn finish(output: &str) -> ModelAction {
    ModelAction::Finish {
        output: output.into(),
    }
}

fn registry_with(echo: bool, fail: bool, cancel_trigger: Option<CancelToken>) -> ToolRegistry {
    let mut registry = ToolRegistry::default();
    if echo {
        registry.register(EchoTool);
    }
    if fail {
        registry.register(FailingTool);
    }
    if let Some(token) = cancel_trigger {
        registry.register(CancellingTool::new(token));
    }
    registry
}

#[test]
fn budget_exhausted_before_next_model_action() {
    // Three steps: CallTool, CallTool, Finish; a fourth CallTool is
    // unreachable. max_steps=2 means the budget runs out before the third
    // model call (Finish) ever happens.
    let script = vec![
        echo_call("c1", 1),
        echo_call("c2", 2),
        finish("completed"),
        echo_call("c4", 4), // unreachable
    ];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(true, false, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 2 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::BudgetExhausted);
    // The model was only called twice: the budget runs out before the next
    // model action, not after one extra call.
    assert_eq!(model.call_count(), 2);
    assert_eq!(result.model_call_count, 2);
}

#[test]
fn tool_result_visible_in_next_model_input() {
    let script = vec![echo_call("c1", 42), finish("completed")];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(true, false, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::Completed);
    assert_eq!(model.call_count(), 2);
    let inputs = model.received_inputs();
    assert!(inputs[0].is_empty());
    assert!(
        inputs[1]
            .iter()
            .any(|item| item.call_id == "c1" && item.output == Some(JsonValue::from(42_i64))),
        "second model input did not carry the first tool result: {:?}",
        inputs[1]
    );
}

#[test]
fn tool_failure_is_observable_not_uncaught() {
    let script = vec![
        ModelAction::CallTool(ToolCall {
            call_id: "c1".into(),
            tool_id: "fail".into(),
            arguments: JsonValue::Object(BTreeMap::new()),
        }),
        finish("completed"),
    ];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(true, true, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo", "fail"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    // A tool failure does not terminate the run or panic; the run reaches
    // Finish normally.
    assert_eq!(result.outcome, Outcome::Completed);
    let tool_results: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            Event::ToolResult(result) => Some(result),
            _ => None,
        })
        .collect();
    assert_eq!(tool_results.len(), 1);
    assert_eq!(tool_results[0].status, ToolResultStatus::Failed);
}

#[test]
fn finish_requires_validation_to_become_completed() {
    let mut model = ScriptedMockModel::new(vec![finish("nope")]);
    let mut registry = registry_with(true, false, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::Failed);
    assert_eq!(result.reason, Some("validation_failed"));
}

#[test]
fn duplicate_call_id_is_rejected() {
    let script = vec![echo_call("c1", 1), echo_call("c1", 2)];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(true, false, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::Failed);
    assert_eq!(result.reason, Some("duplicate_call_id"));
    // The second call never actually executes once judged a duplicate.
    let tool_result_count = result
        .events
        .iter()
        .filter(|event| matches!(event, Event::ToolResult(_)))
        .count();
    assert_eq!(tool_result_count, 1);
}

#[test]
fn unknown_tool_is_rejected_without_executing() {
    let script = vec![ModelAction::CallTool(ToolCall {
        call_id: "c1".into(),
        tool_id: "does-not-exist".into(),
        arguments: JsonValue::Object(BTreeMap::new()),
    })];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(true, false, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo", "does-not-exist"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::Failed);
    assert_eq!(result.reason, Some("unknown_tool"));
    assert!(
        !result
            .events
            .iter()
            .any(|event| matches!(event, Event::ToolResult(_)))
    );
}

#[test]
fn policy_denied_is_reachable() {
    let script = vec![ModelAction::CallTool(ToolCall {
        call_id: "c1".into(),
        tool_id: "fail".into(),
        arguments: JsonValue::Object(BTreeMap::new()),
    })];
    let mut model = ScriptedMockModel::new(script);
    // "fail" is registered but not allow-listed.
    let mut registry = registry_with(true, true, None);
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["echo"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &CancelToken::new(),
    );

    assert_eq!(result.outcome, Outcome::PolicyDenied);
    assert!(
        !result
            .events
            .iter()
            .any(|event| matches!(event, Event::ToolResult(_))),
        "policy-denied call must not execute"
    );
}

#[test]
fn cancel_is_reachable_mid_run() {
    let token = CancelToken::new();
    let script = vec![
        ModelAction::CallTool(ToolCall {
            call_id: "c1".into(),
            tool_id: "cancel_trigger".into(),
            arguments: JsonValue::Object(BTreeMap::new()),
        }),
        finish("completed"), // unreachable: cancellation happens before the next model call
    ];
    let mut model = ScriptedMockModel::new(script);
    let mut registry = registry_with(false, false, Some(token.clone()));
    let result = run_loop(
        &mut model,
        &mut registry,
        &AllowListPolicy::new(["cancel_trigger"]),
        &RequiredOutputValidator::new("completed"),
        &RunLimits { max_steps: 5 },
        &token,
    );

    assert_eq!(result.outcome, Outcome::Cancelled);
    assert_eq!(model.call_count(), 1);
}

/// The five outcomes are mutually exclusive and all reachable.
///
/// Mutual exclusivity is structural: [`RunResult::outcome`] is a single
/// field, and every terminating branch in `run_loop` (see `src/run.rs`)
/// returns exactly once — there is no code path that reports two outcomes
/// for the same run. This test additionally confirms all five are actually
/// exercised somewhere in the suite, not merely defined.
#[test]
fn all_five_outcomes_are_covered_by_the_suite() {
    fn completed() -> RunResult {
        let mut model = ScriptedMockModel::new(vec![finish("completed")]);
        let mut registry = registry_with(true, false, None);
        run_loop(
            &mut model,
            &mut registry,
            &AllowListPolicy::new(["echo"]),
            &RequiredOutputValidator::new("completed"),
            &RunLimits { max_steps: 5 },
            &CancelToken::new(),
        )
    }

    fn failed() -> RunResult {
        let mut model = ScriptedMockModel::new(vec![finish("nope")]);
        let mut registry = registry_with(true, false, None);
        run_loop(
            &mut model,
            &mut registry,
            &AllowListPolicy::new(["echo"]),
            &RequiredOutputValidator::new("completed"),
            &RunLimits { max_steps: 5 },
            &CancelToken::new(),
        )
    }

    fn budget_exhausted() -> RunResult {
        let mut model = ScriptedMockModel::new(vec![echo_call("c1", 1), finish("completed")]);
        let mut registry = registry_with(true, false, None);
        run_loop(
            &mut model,
            &mut registry,
            &AllowListPolicy::new(["echo"]),
            &RequiredOutputValidator::new("completed"),
            &RunLimits { max_steps: 1 },
            &CancelToken::new(),
        )
    }

    fn policy_denied() -> RunResult {
        let mut model = ScriptedMockModel::new(vec![ModelAction::CallTool(ToolCall {
            call_id: "c1".into(),
            tool_id: "fail".into(),
            arguments: JsonValue::Object(BTreeMap::new()),
        })]);
        let mut registry = registry_with(true, true, None);
        run_loop(
            &mut model,
            &mut registry,
            &AllowListPolicy::new(["echo"]),
            &RequiredOutputValidator::new("completed"),
            &RunLimits { max_steps: 5 },
            &CancelToken::new(),
        )
    }

    fn cancelled() -> RunResult {
        let token = CancelToken::new();
        let mut model = ScriptedMockModel::new(vec![ModelAction::CallTool(ToolCall {
            call_id: "c1".into(),
            tool_id: "cancel_trigger".into(),
            arguments: JsonValue::Object(BTreeMap::new()),
        })]);
        let mut registry = registry_with(false, false, Some(token.clone()));
        run_loop(
            &mut model,
            &mut registry,
            &AllowListPolicy::new(["cancel_trigger"]),
            &RequiredOutputValidator::new("completed"),
            &RunLimits { max_steps: 5 },
            &token,
        )
    }

    let scenarios: Vec<(Outcome, RunResult)> = vec![
        (Outcome::Completed, completed()),
        (Outcome::Failed, failed()),
        (Outcome::BudgetExhausted, budget_exhausted()),
        (Outcome::PolicyDenied, policy_denied()),
        (Outcome::Cancelled, cancelled()),
    ];

    for (expected, result) in &scenarios {
        assert_eq!(
            &result.outcome, expected,
            "scenario built for {expected:?} produced {:?} instead",
            result.outcome
        );
    }

    let achieved: std::collections::BTreeSet<Outcome> = [
        Outcome::Completed,
        Outcome::Failed,
        Outcome::BudgetExhausted,
        Outcome::PolicyDenied,
        Outcome::Cancelled,
    ]
    .into_iter()
    .collect();
    let all_outcomes: std::collections::BTreeSet<Outcome> = scenarios
        .iter()
        .map(|(_, result)| result.outcome)
        .collect();
    assert_eq!(achieved.len(), 5);
    assert_eq!(all_outcomes, achieved);
}

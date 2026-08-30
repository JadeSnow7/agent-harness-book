//! M3: minimal Agent Loop.
//!
//! Independent of P0's `DeterministicRunner` composition (that's the
//! Context/Policy/Runtime/Validation/Evidence reference slice, see
//! `book/src/ch5.md` §5.6) — this is M3's own control-flow-only
//! implementation.
//!
//! Each turn:
//!   1. Check cancellation first — cooperative cancellation takes effect
//!      before the next model call, it does not interrupt a tool that is
//!      already executing.
//!   2. Check the budget — exhaustion must happen before the *next* model
//!      action, not after one extra call past the limit.
//!   3. Build input, ask the model, record the action.
//!   4. `Finish` must pass Validation to become `Completed`; failing it is
//!      `Failed`.
//!   5. `CallTool` is checked, in order, before it is actually dispatched:
//!      duplicate `call_id` (structurally invalid), unknown tool
//!      (structurally invalid), Policy (business denial) — the first two
//!      terminate as `Failed`, the third as `PolicyDenied`. Only a call
//!      that passes all three is actually executed; execution failure is a
//!      structured observation fed back into history, and the loop
//!      continues — it does not terminate the run.

use std::collections::BTreeSet;

use agent_core::{ToolCall, ToolExecutor};
use tool_runtime::ToolRegistry;

use crate::model::Model;
use crate::policy::AllowListPolicy;
use crate::types::{Event, HistoryItem, ModelAction, Outcome, RunLimits, RunResult};
use crate::validator::RequiredOutputValidator;

pub fn run_loop<M: Model>(
    model: &mut M,
    registry: &mut ToolRegistry,
    policy: &AllowListPolicy,
    validator: &RequiredOutputValidator,
    limits: &RunLimits,
    cancel_token: &crate::types::CancelToken,
) -> RunResult {
    let mut history: Vec<HistoryItem> = Vec::new();
    let mut events: Vec<Event> = Vec::new();
    let mut seen_call_ids: BTreeSet<String> = BTreeSet::new();
    let mut steps: u32 = 0;

    loop {
        if cancel_token.is_cancelled() {
            events.push(Event::Cancelled);
            return RunResult {
                outcome: Outcome::Cancelled,
                output: None,
                reason: Some("cancelled"),
                events,
                model_call_count: steps,
            };
        }

        if steps >= limits.max_steps {
            events.push(Event::BudgetExhausted {
                max_steps: limits.max_steps,
            });
            return RunResult {
                outcome: Outcome::BudgetExhausted,
                output: None,
                reason: Some("max_steps_exceeded"),
                events,
                model_call_count: steps,
            };
        }

        events.push(Event::ModelInput(history.clone()));
        let action = model.next_action(&history);
        steps += 1;
        events.push(Event::ModelAction(action.clone()));

        match action {
            ModelAction::Finish { output } => {
                if validator.validate(&output) {
                    events.push(Event::ValidationPassed(output.clone()));
                    return RunResult {
                        outcome: Outcome::Completed,
                        output: Some(output),
                        reason: None,
                        events,
                        model_call_count: steps,
                    };
                }
                events.push(Event::ValidationFailed(output));
                return RunResult {
                    outcome: Outcome::Failed,
                    output: None,
                    reason: Some("validation_failed"),
                    events,
                    model_call_count: steps,
                };
            }
            ModelAction::CallTool(call) => {
                if let Some(result) = dispatch_tool_call(
                    call,
                    registry,
                    policy,
                    &mut seen_call_ids,
                    &mut history,
                    &mut events,
                    steps,
                ) {
                    return result;
                }
            }
        }
    }
}

/// Returns `Some(RunResult)` if the call terminated the run, `None` if the
/// run should continue to the next turn.
#[allow(clippy::too_many_arguments)]
fn dispatch_tool_call(
    call: ToolCall,
    registry: &mut ToolRegistry,
    policy: &AllowListPolicy,
    seen_call_ids: &mut BTreeSet<String>,
    history: &mut Vec<HistoryItem>,
    events: &mut Vec<Event>,
    steps: u32,
) -> Option<RunResult> {
    if seen_call_ids.contains(&call.call_id) {
        events.push(Event::InvalidAction {
            reason: "duplicate_call_id",
            detail: call.call_id.clone(),
        });
        return Some(RunResult {
            outcome: Outcome::Failed,
            output: None,
            reason: Some("duplicate_call_id"),
            events: events.clone(),
            model_call_count: steps,
        });
    }
    seen_call_ids.insert(call.call_id.clone());

    if !registry.contains(&call.tool_id) {
        events.push(Event::InvalidAction {
            reason: "unknown_tool",
            detail: call.tool_id.clone(),
        });
        return Some(RunResult {
            outcome: Outcome::Failed,
            output: None,
            reason: Some("unknown_tool"),
            events: events.clone(),
            model_call_count: steps,
        });
    }

    if !policy.check(&call.tool_id) {
        events.push(Event::PolicyDenied {
            tool_id: call.tool_id.clone(),
        });
        return Some(RunResult {
            outcome: Outcome::PolicyDenied,
            output: None,
            reason: Some("policy_denied"),
            events: events.clone(),
            model_call_count: steps,
        });
    }

    let tool_id = call.tool_id.clone();
    let result = registry
        .execute(call)
        .expect("ToolRegistry::execute never returns Err; failures become a Failed ToolResult");
    events.push(Event::ToolResult(result.clone()));

    let mut item = HistoryItem::from(&result);
    item.tool_id = tool_id;
    history.push(item);

    None
}

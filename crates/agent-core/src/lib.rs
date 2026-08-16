//! Contracts and deterministic orchestration for the P0 agent harness.
//!
//! The crate deliberately contains no I/O, provider SDK, or concrete tool
//! implementation.  Implementations of the boundary traits can be composed
//! with [`DeterministicRunner`] entirely in memory.

use std::collections::BTreeMap;
use std::fmt;

/// Identity of one execution attempt.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RunId(pub String);

/// Identity shared by related execution attempts.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub String);

/// Deterministic identity of one event.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub String);

/// Contiguous event position within a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sequence(pub u64);

macro_rules! impl_id_display {
    ($type:ty) => {
        impl fmt::Display for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

impl_id_display!(RunId);
impl_id_display!(SessionId);
impl_id_display!(EventId);

impl From<String> for RunId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for RunId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for SessionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for SessionId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for EventId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for EventId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl EventId {
    pub fn for_sequence(run_id: &RunId, sequence: u64) -> Self {
        Self(format!("{}:{sequence}", run_id.0))
    }
}

/// A deliberately small JSON-like value used at all deterministic boundaries.
/// Objects use a sorted map, so their semantic order is stable without a
/// serialization dependency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(i64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub fn object(entries: impl IntoIterator<Item = (String, JsonValue)>) -> Self {
        Self::Object(entries.into_iter().collect())
    }

    pub fn array(values: impl IntoIterator<Item = JsonValue>) -> Self {
        Self::Array(values.into_iter().collect())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }
}

impl From<bool> for JsonValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for JsonValue {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<i32> for JsonValue {
    fn from(value: i32) -> Self {
        Self::Number(i64::from(value))
    }
}

impl From<String> for JsonValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for JsonValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

/// A caller-provided piece of initial context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextItem {
    pub name: String,
    pub content: String,
}

/// Limits applied by the runner before accepting further work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunLimits {
    pub max_steps: u32,
    pub max_tool_calls: u32,
    pub max_context_items: u32,
}

/// Input accepted by one harness run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRequest {
    pub run_id: RunId,
    pub session_id: SessionId,
    pub instruction: String,
    pub initial_context: Vec<ContextItem>,
    pub deterministic_seed: u64,
    pub limits: RunLimits,
}

/// Authoritative state machine state for one run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentState {
    Created,
    Ready,
    AwaitingModel { step: u32 },
    ApplyingAction { step: u32 },
    PolicyChecking { call_id: String },
    ToolExecuting { call_id: String },
    RecordingResult { call_id: String },
    Validating,
    Completed,
    Failed,
    Terminated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInput {
    pub run_id: RunId,
    pub instruction: String,
    pub messages: Vec<ModelMessage>,
    pub available_tools: Vec<ToolSpec>,
    pub remaining_steps: u32,
    pub remaining_tool_calls: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelMessage {
    pub role: ModelRole,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelRole {
    System,
    User,
    Tool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolSpec {
    pub tool_id: String,
    pub description: String,
    pub input_schema: JsonValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelAction {
    CallTool(ToolCall),
    Finish { output: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCall {
    pub call_id: String,
    pub tool_id: String,
    pub arguments: JsonValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow { call_id: String },
    Deny { call_id: String, reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolResultStatus {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolResult {
    pub call_id: String,
    pub status: ToolResultStatus,
    pub output: Option<JsonValue>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationStatus {
    Passed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationCheck {
    pub name: String,
    pub status: ValidationStatus,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationReport {
    pub status: ValidationStatus,
    pub checks: Vec<ValidationCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub kind: String,
    pub summary: String,
    pub supporting_events: Vec<EventId>,
    pub details: JsonValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminationReason {
    PolicyDenied { call_id: String, reason: String },
    BudgetExhausted,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentError {
    InvalidRequest { detail: String },
    InvalidModelAction { detail: String },
    ModelFailure { detail: String },
    ContextFailure { detail: String },
    ToolRuntimeFailure { call_id: String, detail: String },
    ValidationFailure { detail: String },
    ReplayMismatch { detail: String },
    EventLogFailure { detail: String },
}

impl fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { detail } => write!(formatter, "invalid request: {detail}"),
            Self::InvalidModelAction { detail } => {
                write!(formatter, "invalid model action: {detail}")
            }
            Self::ModelFailure { detail } => write!(formatter, "model failure: {detail}"),
            Self::ContextFailure { detail } => write!(formatter, "context failure: {detail}"),
            Self::ToolRuntimeFailure { call_id, detail } => {
                write!(formatter, "tool runtime failure for {call_id}: {detail}")
            }
            Self::ValidationFailure { detail } => {
                write!(formatter, "validation failure: {detail}")
            }
            Self::ReplayMismatch { detail } => write!(formatter, "replay mismatch: {detail}"),
            Self::EventLogFailure { detail } => write!(formatter, "event log failure: {detail}"),
        }
    }
}

impl std::error::Error for AgentError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    Completed {
        output: String,
        validation: ValidationReport,
    },
    Failed {
        error: AgentError,
    },
    Terminated {
        reason: TerminationReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventEnvelope {
    pub schema_version: u16,
    pub event_id: EventId,
    pub sequence: Sequence,
    pub run_id: RunId,
    pub session_id: SessionId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEvent {
    pub envelope: EventEnvelope,
    pub kind: AgentEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentEventKind {
    RunStarted {
        request: AgentRequest,
    },
    ModelInputBuilt {
        input: ModelInput,
    },
    ModelActionReceived {
        action: ModelAction,
    },
    PolicyEvaluated {
        call: ToolCall,
        decision: PolicyDecision,
    },
    ToolStarted {
        call: ToolCall,
    },
    ToolFinished {
        result: ToolResult,
    },
    ValidationCompleted {
        report: ValidationReport,
    },
    EvidenceRecorded {
        record: EvidenceRecord,
    },
    OutcomeRecorded {
        outcome: RunOutcome,
    },
}

pub trait ModelProvider {
    fn next_action(&mut self, input: ModelInput) -> Result<ModelAction, AgentError>;
}

pub trait ContextBuilder {
    fn build(
        &self,
        request: &AgentRequest,
        history: &[AgentEvent],
        limits: &RunLimits,
    ) -> Result<ModelInput, AgentError>;
}

pub trait PolicyEvaluator {
    fn decide(&self, call: &ToolCall, state: &AgentState) -> PolicyDecision;
}

pub trait ToolExecutor {
    fn execute(&mut self, call: ToolCall) -> Result<ToolResult, AgentError>;
}

pub trait Validator {
    fn validate(
        &self,
        request: &AgentRequest,
        events: &[AgentEvent],
    ) -> Result<ValidationReport, AgentError>;
}

pub trait EventSink {
    fn append(&mut self, event: AgentEvent) -> Result<(), AgentError>;
}

pub trait EventLog {
    fn read_run(
        &self,
        session_id: &SessionId,
        run_id: &RunId,
    ) -> Result<Vec<AgentEvent>, AgentError>;
}

pub trait AgentRunner {
    fn run(&mut self, request: AgentRequest) -> Result<RunOutcome, AgentError>;
}

/// A simple append-only sink useful for deterministic in-memory composition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InMemoryEventSink {
    events: Vec<AgentEvent>,
}

impl InMemoryEventSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[AgentEvent] {
        &self.events
    }

    pub fn into_events(self) -> Vec<AgentEvent> {
        self.events
    }
}

impl EventSink for InMemoryEventSink {
    fn append(&mut self, event: AgentEvent) -> Result<(), AgentError> {
        self.events.push(event);
        Ok(())
    }
}

impl EventLog for InMemoryEventSink {
    fn read_run(
        &self,
        session_id: &SessionId,
        run_id: &RunId,
    ) -> Result<Vec<AgentEvent>, AgentError> {
        Ok(self
            .events
            .iter()
            .filter(|event| {
                &event.envelope.session_id == session_id && &event.envelope.run_id == run_id
            })
            .cloned()
            .collect())
    }
}

/// The P0 in-memory orchestrator.  All dependencies are supplied by the
/// caller, so the core remains independent of providers, tools, and storage.
pub struct DeterministicRunner<M, C, P, T, V, S> {
    model: M,
    context: C,
    policy: P,
    tools: T,
    validator: V,
    sink: S,
}

impl<M, C, P, T, V, S> DeterministicRunner<M, C, P, T, V, S> {
    pub fn new(model: M, context: C, policy: P, tools: T, validator: V, sink: S) -> Self {
        Self {
            model,
            context,
            policy,
            tools,
            validator,
            sink,
        }
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn sink_mut(&mut self) -> &mut S {
        &mut self.sink
    }

    pub fn into_sink(self) -> S {
        self.sink
    }
}

impl<M, C, P, T, V, S> DeterministicRunner<M, C, P, T, V, S>
where
    M: ModelProvider,
    C: ContextBuilder,
    P: PolicyEvaluator,
    T: ToolExecutor,
    V: Validator,
    S: EventSink,
{
    #[allow(unused_assignments)]
    pub fn run(&mut self, request: AgentRequest) -> Result<RunOutcome, AgentError> {
        validate_request(&request)?;

        let mut history = Vec::new();
        let mut next_sequence = 0_u64;
        append_event(
            &mut self.sink,
            &mut history,
            &mut next_sequence,
            &request,
            AgentEventKind::RunStarted {
                request: request.clone(),
            },
        )?;

        let mut accepted_steps = 0_u32;
        let mut accepted_tool_calls = 0_u32;
        let mut seen_call_ids = Vec::new();

        loop {
            if accepted_steps >= request.limits.max_steps {
                return record_outcome(
                    &mut self.sink,
                    &mut history,
                    &mut next_sequence,
                    &request,
                    RunOutcome::Terminated {
                        reason: TerminationReason::BudgetExhausted,
                    },
                );
            }

            let input = match self.context.build(&request, &history, &request.limits) {
                Ok(input) => input,
                Err(error) => {
                    return record_outcome(
                        &mut self.sink,
                        &mut history,
                        &mut next_sequence,
                        &request,
                        RunOutcome::Failed { error },
                    );
                }
            };
            if input.run_id != request.run_id {
                return record_outcome(
                    &mut self.sink,
                    &mut history,
                    &mut next_sequence,
                    &request,
                    RunOutcome::Failed {
                        error: AgentError::ContextFailure {
                            detail: "context builder returned a different run ID".to_owned(),
                        },
                    },
                );
            }
            append_event(
                &mut self.sink,
                &mut history,
                &mut next_sequence,
                &request,
                AgentEventKind::ModelInputBuilt {
                    input: input.clone(),
                },
            )?;

            let action = match self.model.next_action(input) {
                Ok(action) => action,
                Err(error) => {
                    return record_outcome(
                        &mut self.sink,
                        &mut history,
                        &mut next_sequence,
                        &request,
                        RunOutcome::Failed { error },
                    );
                }
            };
            append_event(
                &mut self.sink,
                &mut history,
                &mut next_sequence,
                &request,
                AgentEventKind::ModelActionReceived {
                    action: action.clone(),
                },
            )?;
            accepted_steps = accepted_steps.saturating_add(1);

            match action {
                ModelAction::Finish { output } => {
                    let report = match self.validator.validate(&request, &history) {
                        Ok(report) => report,
                        Err(error) => {
                            return record_outcome(
                                &mut self.sink,
                                &mut history,
                                &mut next_sequence,
                                &request,
                                RunOutcome::Failed { error },
                            );
                        }
                    };
                    append_event(
                        &mut self.sink,
                        &mut history,
                        &mut next_sequence,
                        &request,
                        AgentEventKind::ValidationCompleted {
                            report: report.clone(),
                        },
                    )?;
                    if report.status == ValidationStatus::Passed {
                        return record_outcome(
                            &mut self.sink,
                            &mut history,
                            &mut next_sequence,
                            &request,
                            RunOutcome::Completed {
                                output,
                                validation: report,
                            },
                        );
                    }
                    return record_outcome(
                        &mut self.sink,
                        &mut history,
                        &mut next_sequence,
                        &request,
                        RunOutcome::Failed {
                            error: AgentError::ValidationFailure {
                                detail: validation_failure_detail(&report),
                            },
                        },
                    );
                }
                ModelAction::CallTool(call) => {
                    if let Some(detail) = validate_call(&call, &seen_call_ids) {
                        return record_outcome(
                            &mut self.sink,
                            &mut history,
                            &mut next_sequence,
                            &request,
                            RunOutcome::Failed {
                                error: AgentError::InvalidModelAction { detail },
                            },
                        );
                    }
                    if accepted_tool_calls >= request.limits.max_tool_calls {
                        return record_outcome(
                            &mut self.sink,
                            &mut history,
                            &mut next_sequence,
                            &request,
                            RunOutcome::Terminated {
                                reason: TerminationReason::BudgetExhausted,
                            },
                        );
                    }
                    seen_call_ids.push(call.call_id.clone());
                    let policy_state = AgentState::PolicyChecking {
                        call_id: call.call_id.clone(),
                    };
                    let decision = self.policy.decide(&call, &policy_state);
                    if !decision_matches_call(&decision, &call.call_id) {
                        append_event(
                            &mut self.sink,
                            &mut history,
                            &mut next_sequence,
                            &request,
                            AgentEventKind::PolicyEvaluated {
                                call: call.clone(),
                                decision,
                            },
                        )?;
                        return record_outcome(
                            &mut self.sink,
                            &mut history,
                            &mut next_sequence,
                            &request,
                            RunOutcome::Failed {
                                error: AgentError::InvalidModelAction {
                                    detail: "policy decision call ID does not match tool call"
                                        .to_owned(),
                                },
                            },
                        );
                    }
                    append_event(
                        &mut self.sink,
                        &mut history,
                        &mut next_sequence,
                        &request,
                        AgentEventKind::PolicyEvaluated {
                            call: call.clone(),
                            decision: decision.clone(),
                        },
                    )?;
                    match decision {
                        PolicyDecision::Deny { call_id, reason } => {
                            return record_outcome(
                                &mut self.sink,
                                &mut history,
                                &mut next_sequence,
                                &request,
                                RunOutcome::Terminated {
                                    reason: TerminationReason::PolicyDenied { call_id, reason },
                                },
                            );
                        }
                        PolicyDecision::Allow { .. } => {
                            append_event(
                                &mut self.sink,
                                &mut history,
                                &mut next_sequence,
                                &request,
                                AgentEventKind::ToolStarted { call: call.clone() },
                            )?;
                            accepted_tool_calls = accepted_tool_calls.saturating_add(1);
                            let tool_result = match self.tools.execute(call.clone()) {
                                Ok(result) if result.call_id == call.call_id => result,
                                Ok(result) => {
                                    let runtime_error = AgentError::ToolRuntimeFailure {
                                        call_id: call.call_id.clone(),
                                        detail: format!(
                                            "tool returned result for unexpected call ID {}",
                                            result.call_id
                                        ),
                                    };
                                    let failure_result = ToolResult {
                                        call_id: call.call_id.clone(),
                                        status: ToolResultStatus::Failed,
                                        output: None,
                                        error: Some(runtime_error.to_string()),
                                    };
                                    append_event(
                                        &mut self.sink,
                                        &mut history,
                                        &mut next_sequence,
                                        &request,
                                        AgentEventKind::ToolFinished {
                                            result: failure_result,
                                        },
                                    )?;
                                    return record_outcome(
                                        &mut self.sink,
                                        &mut history,
                                        &mut next_sequence,
                                        &request,
                                        RunOutcome::Failed {
                                            error: runtime_error,
                                        },
                                    );
                                }
                                Err(error) => {
                                    let runtime_error = match error {
                                        AgentError::ToolRuntimeFailure { detail, .. } => {
                                            AgentError::ToolRuntimeFailure {
                                                call_id: call.call_id.clone(),
                                                detail,
                                            }
                                        }
                                        other => AgentError::ToolRuntimeFailure {
                                            call_id: call.call_id.clone(),
                                            detail: other.to_string(),
                                        },
                                    };
                                    let failure_result = ToolResult {
                                        call_id: call.call_id.clone(),
                                        status: ToolResultStatus::Failed,
                                        output: None,
                                        error: Some(runtime_error.to_string()),
                                    };
                                    append_event(
                                        &mut self.sink,
                                        &mut history,
                                        &mut next_sequence,
                                        &request,
                                        AgentEventKind::ToolFinished {
                                            result: failure_result,
                                        },
                                    )?;
                                    return record_outcome(
                                        &mut self.sink,
                                        &mut history,
                                        &mut next_sequence,
                                        &request,
                                        RunOutcome::Failed {
                                            error: runtime_error,
                                        },
                                    );
                                }
                            };
                            append_event(
                                &mut self.sink,
                                &mut history,
                                &mut next_sequence,
                                &request,
                                AgentEventKind::ToolFinished {
                                    result: tool_result,
                                },
                            )?;
                        }
                    }
                }
            }
        }
    }
}

impl<M, C, P, T, V, S> AgentRunner for DeterministicRunner<M, C, P, T, V, S>
where
    M: ModelProvider,
    C: ContextBuilder,
    P: PolicyEvaluator,
    T: ToolExecutor,
    V: Validator,
    S: EventSink,
{
    fn run(&mut self, request: AgentRequest) -> Result<RunOutcome, AgentError> {
        DeterministicRunner::run(self, request)
    }
}

fn validate_request(request: &AgentRequest) -> Result<(), AgentError> {
    if request.run_id.0.is_empty() {
        return Err(AgentError::InvalidRequest {
            detail: "run ID must not be empty".to_owned(),
        });
    }
    if request.session_id.0.is_empty() {
        return Err(AgentError::InvalidRequest {
            detail: "session ID must not be empty".to_owned(),
        });
    }
    if request.instruction.is_empty() {
        return Err(AgentError::InvalidRequest {
            detail: "instruction must not be empty".to_owned(),
        });
    }
    if request.initial_context.len() > request.limits.max_context_items as usize {
        return Err(AgentError::InvalidRequest {
            detail: "initial context exceeds max_context_items".to_owned(),
        });
    }
    Ok(())
}

fn validate_call(call: &ToolCall, seen_call_ids: &[String]) -> Option<String> {
    if call.call_id.is_empty() {
        return Some("tool call ID must not be empty".to_owned());
    }
    if call.tool_id.is_empty() {
        return Some("tool ID must not be empty".to_owned());
    }
    if seen_call_ids.iter().any(|seen| seen == &call.call_id) {
        return Some(format!("tool call ID {} was already used", call.call_id));
    }
    None
}

fn decision_matches_call(decision: &PolicyDecision, call_id: &str) -> bool {
    match decision {
        PolicyDecision::Allow {
            call_id: decision_id,
        }
        | PolicyDecision::Deny {
            call_id: decision_id,
            ..
        } => decision_id == call_id,
    }
}

fn validation_failure_detail(report: &ValidationReport) -> String {
    report
        .checks
        .iter()
        .find(|check| check.status == ValidationStatus::Failed)
        .map(|check| format!("{}: {}", check.name, check.detail))
        .unwrap_or_else(|| "validation report failed".to_owned())
}

fn append_event<S: EventSink>(
    sink: &mut S,
    history: &mut Vec<AgentEvent>,
    next_sequence: &mut u64,
    request: &AgentRequest,
    kind: AgentEventKind,
) -> Result<(), AgentError> {
    let sequence = Sequence(*next_sequence);
    let event = AgentEvent {
        envelope: EventEnvelope {
            schema_version: 1,
            event_id: EventId(format!("{}:{}", request.run_id, sequence.0)),
            sequence,
            run_id: request.run_id.clone(),
            session_id: request.session_id.clone(),
        },
        kind,
    };
    sink.append(event.clone())?;
    history.push(event);
    *next_sequence = next_sequence.saturating_add(1);
    Ok(())
}

fn record_outcome<S: EventSink>(
    sink: &mut S,
    history: &mut Vec<AgentEvent>,
    next_sequence: &mut u64,
    request: &AgentRequest,
    outcome: RunOutcome,
) -> Result<RunOutcome, AgentError> {
    let evidence = EvidenceRecord {
        kind: if matches!(&outcome, RunOutcome::Failed { .. }) {
            "run-failure"
        } else {
            "run-outcome"
        }
        .to_owned(),
        summary: "Outcome is supported by the recorded run events".to_owned(),
        supporting_events: history
            .iter()
            .map(|event| event.envelope.event_id.clone())
            .collect(),
        details: JsonValue::Null,
    };
    append_event(
        sink,
        history,
        next_sequence,
        request,
        AgentEventKind::EvidenceRecorded { record: evidence },
    )?;
    append_event(
        sink,
        history,
        next_sequence,
        request,
        AgentEventKind::OutcomeRecorded {
            outcome: outcome.clone(),
        },
    )?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestContext;

    impl ContextBuilder for TestContext {
        fn build(
            &self,
            request: &AgentRequest,
            history: &[AgentEvent],
            limits: &RunLimits,
        ) -> Result<ModelInput, AgentError> {
            let mut messages = vec![ModelMessage {
                role: ModelRole::User,
                content: request.instruction.clone(),
            }];
            for event in history {
                if let AgentEventKind::ToolFinished { result } = &event.kind {
                    messages.push(ModelMessage {
                        role: ModelRole::Tool,
                        content: format!("{}:{:?}", result.call_id, result.status),
                    });
                }
            }
            Ok(ModelInput {
                run_id: request.run_id.clone(),
                instruction: request.instruction.clone(),
                messages,
                available_tools: vec![ToolSpec {
                    tool_id: "echo".to_owned(),
                    description: "echo input".to_owned(),
                    input_schema: JsonValue::Object(BTreeMap::new()),
                }],
                remaining_steps: limits.max_steps.saturating_sub(
                    history
                        .iter()
                        .filter(|event| {
                            matches!(event.kind, AgentEventKind::ModelActionReceived { .. })
                        })
                        .count() as u32,
                ),
                remaining_tool_calls: limits.max_tool_calls.saturating_sub(
                    history
                        .iter()
                        .filter(|event| matches!(event.kind, AgentEventKind::ToolStarted { .. }))
                        .count() as u32,
                ),
            })
        }
    }

    struct ScriptedModel {
        actions: Vec<Result<ModelAction, AgentError>>,
        calls: usize,
    }

    impl ModelProvider for ScriptedModel {
        fn next_action(&mut self, _input: ModelInput) -> Result<ModelAction, AgentError> {
            self.calls += 1;
            if self.actions.is_empty() {
                return Err(AgentError::ModelFailure {
                    detail: "script exhausted".to_owned(),
                });
            }
            self.actions.remove(0)
        }
    }

    #[derive(Clone)]
    struct TestPolicy {
        allow: bool,
    }

    impl PolicyEvaluator for TestPolicy {
        fn decide(&self, call: &ToolCall, _state: &AgentState) -> PolicyDecision {
            if self.allow {
                PolicyDecision::Allow {
                    call_id: call.call_id.clone(),
                }
            } else {
                PolicyDecision::Deny {
                    call_id: call.call_id.clone(),
                    reason: "denied by test policy".to_owned(),
                }
            }
        }
    }

    struct TestTool {
        result: ToolResultStatus,
        calls: usize,
    }

    impl ToolExecutor for TestTool {
        fn execute(&mut self, call: ToolCall) -> Result<ToolResult, AgentError> {
            self.calls += 1;
            Ok(ToolResult {
                call_id: call.call_id,
                status: self.result.clone(),
                output: Some(call.arguments),
                error: None,
            })
        }
    }

    struct FailingTool;

    impl ToolExecutor for FailingTool {
        fn execute(&mut self, call: ToolCall) -> Result<ToolResult, AgentError> {
            Err(AgentError::ToolRuntimeFailure {
                call_id: call.call_id,
                detail: "fixture runtime failure".to_owned(),
            })
        }
    }

    #[derive(Clone)]
    struct TestValidator {
        status: ValidationStatus,
        calls: std::cell::Cell<usize>,
    }

    impl Validator for TestValidator {
        fn validate(
            &self,
            _request: &AgentRequest,
            _events: &[AgentEvent],
        ) -> Result<ValidationReport, AgentError> {
            self.calls.set(self.calls.get() + 1);
            Ok(ValidationReport {
                status: self.status.clone(),
                checks: vec![ValidationCheck {
                    name: "test".to_owned(),
                    status: self.status.clone(),
                    detail: "test validation".to_owned(),
                }],
            })
        }
    }

    fn request(limits: RunLimits) -> AgentRequest {
        AgentRequest {
            run_id: RunId("run-1".to_owned()),
            session_id: SessionId("session-1".to_owned()),
            instruction: "do the deterministic task".to_owned(),
            initial_context: Vec::new(),
            deterministic_seed: 7,
            limits,
        }
    }

    fn finish() -> ModelAction {
        ModelAction::Finish {
            output: "done".to_owned(),
        }
    }

    fn call(call_id: &str) -> ModelAction {
        ModelAction::CallTool(ToolCall {
            call_id: call_id.to_owned(),
            tool_id: "echo".to_owned(),
            arguments: JsonValue::string("hello"),
        })
    }

    fn runner(
        actions: Vec<Result<ModelAction, AgentError>>,
        allow: bool,
        tool_status: ToolResultStatus,
        validation_status: ValidationStatus,
    ) -> DeterministicRunner<
        ScriptedModel,
        TestContext,
        TestPolicy,
        TestTool,
        TestValidator,
        InMemoryEventSink,
    > {
        DeterministicRunner::new(
            ScriptedModel { actions, calls: 0 },
            TestContext,
            TestPolicy { allow },
            TestTool {
                result: tool_status,
                calls: 0,
            },
            TestValidator {
                status: validation_status,
                calls: std::cell::Cell::new(0),
            },
            InMemoryEventSink::new(),
        )
    }

    fn kinds(
        runner: &DeterministicRunner<
            ScriptedModel,
            TestContext,
            TestPolicy,
            TestTool,
            TestValidator,
            InMemoryEventSink,
        >,
    ) -> Vec<&'static str> {
        runner
            .sink()
            .events()
            .iter()
            .map(|event| match event.kind {
                AgentEventKind::RunStarted { .. } => "RunStarted",
                AgentEventKind::ModelInputBuilt { .. } => "ModelInputBuilt",
                AgentEventKind::ModelActionReceived { .. } => "ModelActionReceived",
                AgentEventKind::PolicyEvaluated { .. } => "PolicyEvaluated",
                AgentEventKind::ToolStarted { .. } => "ToolStarted",
                AgentEventKind::ToolFinished { .. } => "ToolFinished",
                AgentEventKind::ValidationCompleted { .. } => "ValidationCompleted",
                AgentEventKind::EvidenceRecorded { .. } => "EvidenceRecorded",
                AgentEventKind::OutcomeRecorded { .. } => "OutcomeRecorded",
            })
            .collect()
    }

    #[test]
    fn normal_finish_validates_before_completion() {
        let mut runner = runner(
            vec![Ok(finish())],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 2,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(outcome, Ok(RunOutcome::Completed { .. })));
        assert_eq!(
            kinds(&runner),
            vec![
                "RunStarted",
                "ModelInputBuilt",
                "ModelActionReceived",
                "ValidationCompleted",
                "EvidenceRecorded",
                "OutcomeRecorded"
            ]
        );
        assert_eq!(runner.sink().events()[0].envelope.sequence, Sequence(0));
        assert_eq!(runner.sink().events().len(), 6);
    }

    #[test]
    fn single_tool_success_has_ordered_lifecycle() {
        let mut runner = runner(
            vec![Ok(call("call-1")), Ok(finish())],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 3,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(outcome, Ok(RunOutcome::Completed { .. })));
        assert_eq!(
            kinds(&runner),
            vec![
                "RunStarted",
                "ModelInputBuilt",
                "ModelActionReceived",
                "PolicyEvaluated",
                "ToolStarted",
                "ToolFinished",
                "ModelInputBuilt",
                "ModelActionReceived",
                "ValidationCompleted",
                "EvidenceRecorded",
                "OutcomeRecorded"
            ]
        );
        assert_eq!(
            runner
                .sink()
                .events()
                .iter()
                .filter(|event| matches!(event.kind, AgentEventKind::OutcomeRecorded { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn denial_has_no_tool_events_and_terminates() {
        let mut runner = runner(
            vec![Ok(call("call-1")), Ok(finish())],
            false,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 2,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(
            outcome,
            Ok(RunOutcome::Terminated {
                reason: TerminationReason::PolicyDenied { .. }
            })
        ));
        assert_eq!(kinds(&runner).last(), Some(&"OutcomeRecorded"));
        assert!(!runner.sink().events().iter().any(|event| {
            matches!(
                event.kind,
                AgentEventKind::ToolStarted { .. } | AgentEventKind::ToolFinished { .. }
            )
        }));
    }

    #[test]
    fn invalid_action_is_structured_failure() {
        let mut runner = runner(
            vec![Ok(ModelAction::CallTool(ToolCall {
                call_id: String::new(),
                tool_id: "echo".to_owned(),
                arguments: JsonValue::Null,
            }))],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 2,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(
            outcome,
            Ok(RunOutcome::Failed {
                error: AgentError::InvalidModelAction { .. }
            })
        ));
        assert_eq!(kinds(&runner).last(), Some(&"OutcomeRecorded"));
    }

    #[test]
    fn model_error_terminates_without_being_swallowed() {
        let mut runner = runner(
            vec![Err(AgentError::ModelFailure {
                detail: "fixture failed".to_owned(),
            })],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 2,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert_eq!(
            outcome,
            Ok(RunOutcome::Failed {
                error: AgentError::ModelFailure {
                    detail: "fixture failed".to_owned()
                }
            })
        );
    }

    #[test]
    fn max_steps_terminates_before_another_model_action() {
        let mut runner = runner(
            vec![Ok(call("call-1")), Ok(finish())],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 1,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert_eq!(
            outcome,
            Ok(RunOutcome::Terminated {
                reason: TerminationReason::BudgetExhausted
            })
        );
        assert_eq!(
            runner
                .sink()
                .events()
                .iter()
                .filter(|event| matches!(event.kind, AgentEventKind::ModelActionReceived { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn structured_tool_failure_is_recorded_and_can_finish() {
        let mut runner = runner(
            vec![Ok(call("call-1")), Ok(finish())],
            true,
            ToolResultStatus::Failed,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 3,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(outcome, Ok(RunOutcome::Completed { .. })));
        let finished = runner.sink().events().iter().find_map(|event| {
            if let AgentEventKind::ToolFinished { result } = &event.kind {
                Some(result)
            } else {
                None
            }
        });
        assert!(matches!(
            finished,
            Some(ToolResult {
                status: ToolResultStatus::Failed,
                ..
            })
        ));
    }

    #[test]
    fn tool_runtime_failure_finishes_lifecycle_and_terminates_structurally() {
        let mut runner = DeterministicRunner::new(
            ScriptedModel {
                actions: vec![Ok(call("call-1")), Ok(finish())],
                calls: 0,
            },
            TestContext,
            TestPolicy { allow: true },
            FailingTool,
            TestValidator {
                status: ValidationStatus::Passed,
                calls: std::cell::Cell::new(0),
            },
            InMemoryEventSink::new(),
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 3,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert_eq!(
            outcome,
            Ok(RunOutcome::Failed {
                error: AgentError::ToolRuntimeFailure {
                    call_id: "call-1".to_owned(),
                    detail: "fixture runtime failure".to_owned(),
                }
            })
        );
        assert_eq!(
            runner
                .sink()
                .events()
                .iter()
                .filter(|event| matches!(event.kind, AgentEventKind::ToolFinished { .. }))
                .count(),
            1
        );
        assert!(
            !runner
                .sink()
                .events()
                .iter()
                .any(|event| { matches!(event.kind, AgentEventKind::ValidationCompleted { .. }) })
        );
    }

    #[test]
    fn failed_validation_is_recorded_before_failed_outcome() {
        let mut runner = runner(
            vec![Ok(finish())],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Failed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 2,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(
            outcome,
            Ok(RunOutcome::Failed {
                error: AgentError::ValidationFailure { .. }
            })
        ));
        assert!(matches!(
            runner.sink().events().get(3).map(|event| &event.kind),
            Some(AgentEventKind::ValidationCompleted { .. })
        ));
    }

    #[test]
    fn event_ids_and_sequences_are_deterministic() {
        let build = || {
            let mut runner = runner(
                vec![Ok(call("call-1")), Ok(finish())],
                true,
                ToolResultStatus::Succeeded,
                ValidationStatus::Passed,
            );
            let result = runner.run(request(RunLimits {
                max_steps: 3,
                max_tool_calls: 1,
                max_context_items: 2,
            }));
            (result, runner.into_sink().into_events())
        };
        let (first_result, first_events) = build();
        let (second_result, second_events) = build();
        assert_eq!(first_result, second_result);
        assert_eq!(first_events, second_events);
        for (index, event) in first_events.iter().enumerate() {
            assert_eq!(event.envelope.sequence, Sequence(index as u64));
            assert_eq!(event.envelope.event_id.0, format!("run-1:{index}"));
        }
    }

    struct FailingSink;

    impl EventSink for FailingSink {
        fn append(&mut self, _event: AgentEvent) -> Result<(), AgentError> {
            Err(AgentError::EventLogFailure {
                detail: "sink unavailable".to_owned(),
            })
        }
    }

    #[test]
    fn event_sink_failure_is_propagated() {
        let mut runner = DeterministicRunner::new(
            ScriptedModel {
                actions: vec![Ok(finish())],
                calls: 0,
            },
            TestContext,
            TestPolicy { allow: true },
            TestTool {
                result: ToolResultStatus::Succeeded,
                calls: 0,
            },
            TestValidator {
                status: ValidationStatus::Passed,
                calls: std::cell::Cell::new(0),
            },
            FailingSink,
        );
        let result = runner.run(request(RunLimits {
            max_steps: 1,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert_eq!(
            result,
            Err(AgentError::EventLogFailure {
                detail: "sink unavailable".to_owned()
            })
        );
    }

    #[test]
    fn zero_step_budget_still_records_a_terminal_outcome() {
        let mut runner = runner(
            vec![Ok(finish())],
            true,
            ToolResultStatus::Succeeded,
            ValidationStatus::Passed,
        );
        let outcome = runner.run(request(RunLimits {
            max_steps: 0,
            max_tool_calls: 1,
            max_context_items: 2,
        }));
        assert!(matches!(
            outcome,
            Ok(RunOutcome::Terminated {
                reason: TerminationReason::BudgetExhausted
            })
        ));
        assert_eq!(
            kinds(&runner),
            vec!["RunStarted", "EvidenceRecorded", "OutcomeRecorded"]
        );
    }
}

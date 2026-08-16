//! In-memory event storage and replay validation for P0.

use agent_core::{AgentError, AgentEvent, EventLog, EventSink, RunId, SessionId};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct InMemoryEventStore {
    runs: BTreeMap<(SessionId, RunId), Vec<AgentEvent>>,
    fail_after: Option<usize>,
}

impl InMemoryEventStore {
    pub fn with_failure_after(event_count: usize) -> Self {
        Self {
            runs: BTreeMap::new(),
            fail_after: Some(event_count),
        }
    }
    pub fn validate_run(events: &[AgentEvent]) -> Result<(), AgentError> {
        let Some(first) = events.first() else {
            return Ok(());
        };
        for (expected, event) in events.iter().enumerate() {
            if event.envelope.sequence != agent_core::Sequence(expected as u64) {
                return Err(AgentError::ReplayMismatch {
                    detail: "event sequence is not contiguous".into(),
                });
            }
            if event.envelope.run_id != first.envelope.run_id
                || event.envelope.session_id != first.envelope.session_id
            {
                return Err(AgentError::ReplayMismatch {
                    detail: "event identity does not match run".into(),
                });
            }
            if event.envelope.event_id
                != agent_core::EventId::for_sequence(&first.envelope.run_id, expected as u64)
            {
                return Err(AgentError::ReplayMismatch {
                    detail: "event ID is not deterministic".into(),
                });
            }
        }
        Ok(())
    }
}

impl EventSink for InMemoryEventStore {
    fn append(&mut self, event: AgentEvent) -> Result<(), AgentError> {
        let key = (
            event.envelope.session_id.clone(),
            event.envelope.run_id.clone(),
        );
        let run = self.runs.entry(key).or_default();
        if self.fail_after.is_some_and(|limit| run.len() >= limit) {
            return Err(AgentError::EventLogFailure {
                detail: "injected event-store failure".into(),
            });
        }
        if event.envelope.sequence != agent_core::Sequence(run.len() as u64) {
            return Err(AgentError::ReplayMismatch {
                detail: "append sequence is not contiguous".into(),
            });
        }
        run.push(event);
        Ok(())
    }
}

impl EventLog for InMemoryEventStore {
    fn read_run(
        &self,
        session_id: &SessionId,
        run_id: &RunId,
    ) -> Result<Vec<AgentEvent>, AgentError> {
        let events = self
            .runs
            .get(&(session_id.clone(), run_id.clone()))
            .cloned()
            .unwrap_or_default();
        Self::validate_run(&events)?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{AgentEventKind, EventEnvelope};

    fn event(sequence: u64) -> AgentEvent {
        let run = RunId("r".into());
        let session = SessionId("s".into());
        AgentEvent {
            envelope: EventEnvelope {
                schema_version: 1,
                event_id: agent_core::EventId::for_sequence(&run, sequence),
                sequence: agent_core::Sequence(sequence),
                run_id: run,
                session_id: session,
            },
            kind: AgentEventKind::EvidenceRecorded {
                record: agent_core::EvidenceRecord {
                    kind: "x".into(),
                    summary: "x".into(),
                    supporting_events: Vec::new(),
                    details: agent_core::JsonValue::Null,
                },
            },
        }
    }

    #[test]
    fn append_read_and_isolate_runs() {
        let mut store = InMemoryEventStore::default();
        store.append(event(0)).expect("append");
        let events = store
            .read_run(&SessionId("s".into()), &RunId("r".into()))
            .expect("read");
        assert_eq!(events.len(), 1);
        assert!(
            store
                .read_run(&SessionId("other".into()), &RunId("r".into()))
                .expect("read")
                .is_empty()
        );
    }

    #[test]
    fn failures_are_explicit() {
        let mut store = InMemoryEventStore::with_failure_after(0);
        assert!(matches!(
            store.append(event(0)),
            Err(AgentError::EventLogFailure { .. })
        ));
        assert!(matches!(
            InMemoryEventStore::validate_run(&[event(1)]),
            Err(AgentError::ReplayMismatch { .. })
        ));
    }
}

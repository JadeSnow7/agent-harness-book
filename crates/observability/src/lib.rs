//! Read-only projections over recorded P0 events.

use agent_core::{AgentEvent, AgentEventKind, EventId, EvidenceRecord, RunOutcome};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunSummary {
    pub event_count: usize,
    pub outcome: Option<RunOutcome>,
    pub evidence: Vec<EvidenceRecord>,
}

pub fn summarize(events: &[AgentEvent]) -> RunSummary {
    let outcome = events.iter().find_map(|event| match &event.kind {
        AgentEventKind::OutcomeRecorded { outcome } => Some(outcome.clone()),
        _ => None,
    });
    let evidence = events
        .iter()
        .filter_map(|event| match &event.kind {
            AgentEventKind::EvidenceRecorded { record } => Some(record.clone()),
            _ => None,
        })
        .collect();
    RunSummary {
        event_count: events.len(),
        outcome,
        evidence,
    }
}

pub fn evidence_event_ids(events: &[AgentEvent]) -> Vec<EventId> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            AgentEventKind::EvidenceRecorded { record } => Some(record.supporting_events.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::{AgentEventKind, EventEnvelope, EventId, JsonValue, RunId, SessionId};

    #[test]
    fn summary_only_projects_recorded_facts() {
        let run = RunId("r".into());
        let session = SessionId("s".into());
        let event = AgentEvent {
            envelope: EventEnvelope {
                schema_version: 1,
                event_id: EventId::for_sequence(&run, 0),
                sequence: agent_core::Sequence(0),
                run_id: run,
                session_id: session,
            },
            kind: AgentEventKind::EvidenceRecorded {
                record: EvidenceRecord {
                    kind: "check".into(),
                    summary: "ok".into(),
                    supporting_events: vec![EventId("prior".into())],
                    details: JsonValue::Null,
                },
            },
        };
        let summary = summarize(std::slice::from_ref(&event));
        assert_eq!(summary.event_count, 1);
        assert_eq!(evidence_event_ids(&[event]), vec![EventId("prior".into())]);
    }
}

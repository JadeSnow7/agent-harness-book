//! Deterministic allow/deny policy.

use agent_core::{AgentState, PolicyDecision, PolicyEvaluator, ToolCall};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleDecision {
    Allow,
    Deny { reason: String },
}

impl RuleDecision {
    pub fn deny(reason: impl Into<String>) -> Self {
        Self::Deny {
            reason: reason.into(),
        }
    }
}

/// Deterministic policy with an explicit default and per-tool overrides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicPolicy {
    default: RuleDecision,
    rules: BTreeMap<String, RuleDecision>,
}

pub type PolicyEngine = DeterministicPolicy;

impl DeterministicPolicy {
    pub fn new(default: RuleDecision) -> Self {
        Self {
            default,
            rules: BTreeMap::new(),
        }
    }

    pub fn deny_all() -> Self {
        Self::new(RuleDecision::deny("tool_not_allowlisted"))
    }

    pub fn allow_all() -> Self {
        Self::new(RuleDecision::Allow)
    }

    pub fn with_rule(mut self, tool_id: impl Into<String>, decision: RuleDecision) -> Self {
        self.set_rule(tool_id, decision);
        self
    }

    pub fn set_rule(&mut self, tool_id: impl Into<String>, decision: RuleDecision) {
        self.rules.insert(tool_id.into(), decision);
    }

    pub fn allow_tool(&mut self, tool_id: impl Into<String>) {
        self.set_rule(tool_id, RuleDecision::Allow);
    }

    pub fn deny_tool(&mut self, tool_id: impl Into<String>, reason: impl Into<String>) {
        self.set_rule(tool_id, RuleDecision::deny(reason));
    }
}

impl Default for DeterministicPolicy {
    fn default() -> Self {
        Self::deny_all()
    }
}

impl PolicyEvaluator for DeterministicPolicy {
    fn decide(&self, call: &ToolCall, _state: &AgentState) -> PolicyDecision {
        match self.rules.get(&call.tool_id).unwrap_or(&self.default) {
            RuleDecision::Allow => PolicyDecision::Allow {
                call_id: call.call_id.clone(),
            },
            RuleDecision::Deny { reason } => PolicyDecision::Deny {
                call_id: call.call_id.clone(),
                reason: reason.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AllowListPolicy {
    allowed_tools: BTreeSet<String>,
}

impl AllowListPolicy {
    pub fn new<I, S>(tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed_tools: tools.into_iter().map(Into::into).collect(),
        }
    }
}

impl PolicyEvaluator for AllowListPolicy {
    fn decide(&self, call: &ToolCall, _state: &AgentState) -> PolicyDecision {
        if self.allowed_tools.contains(&call.tool_id) {
            PolicyDecision::Allow {
                call_id: call.call_id.clone(),
            }
        } else {
            PolicyDecision::Deny {
                call_id: call.call_id.clone(),
                reason: format!("tool '{}' is not allowed", call.tool_id),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::JsonValue;

    fn call() -> ToolCall {
        ToolCall {
            call_id: "c".into(),
            tool_id: "echo".into(),
            arguments: JsonValue::Null,
        }
    }

    #[test]
    fn allow_list_is_deterministic() {
        assert!(matches!(
            AllowListPolicy::new(["echo"]).decide(&call(), &AgentState::Ready),
            PolicyDecision::Allow { .. }
        ));
        assert!(matches!(
            AllowListPolicy::default().decide(&call(), &AgentState::Ready),
            PolicyDecision::Deny { .. }
        ));
    }

    #[test]
    fn explicit_allow_and_deny_rules_are_deterministic() {
        let mut policy = DeterministicPolicy::deny_all();
        policy.allow_tool("echo");
        policy.deny_tool("delete", "destructive_operation_requires_approval");
        let state = AgentState::PolicyChecking {
            call_id: "c".into(),
        };
        assert_eq!(
            policy.decide(&call(), &state),
            PolicyDecision::Allow {
                call_id: "c".into()
            }
        );
        let denied_call = ToolCall {
            call_id: "delete-1".into(),
            tool_id: "delete".into(),
            arguments: JsonValue::Null,
        };
        assert_eq!(
            policy.decide(&denied_call, &state),
            PolicyDecision::Deny {
                call_id: "delete-1".into(),
                reason: "destructive_operation_requires_approval".into()
            }
        );
    }

    #[test]
    fn denial_does_not_have_an_executor_path() {
        let policy = DeterministicPolicy::deny_all();
        let decision = policy.decide(&call(), &AgentState::Ready);
        let mut executions = 0;
        if matches!(decision, PolicyDecision::Allow { .. }) {
            executions += 1;
        }
        assert_eq!(executions, 0);
        assert!(matches!(decision, PolicyDecision::Deny { .. }));
    }
}

//! M3: minimal allow-list policy.
//!
//! Independent of `policy_engine::AllowListPolicy` (that one implements
//! `agent_core::PolicyEvaluator`, tied to `AgentState`). M3's loop only
//! needs "is this tool ID allowed."

use std::collections::BTreeSet;

pub struct AllowListPolicy {
    allowed: BTreeSet<String>,
}

impl AllowListPolicy {
    pub fn new<I, S>(allowed: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: allowed.into_iter().map(Into::into).collect(),
        }
    }

    pub fn check(&self, tool_id: &str) -> bool {
        self.allowed.contains(tool_id)
    }
}

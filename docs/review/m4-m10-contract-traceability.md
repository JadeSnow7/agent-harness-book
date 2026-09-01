# M4–M10 Contract Traceability for GPT-5.6 Cyber

这份矩阵把本轮 Python 教学切片、仓库内 Rust P0 和只读 HUSH 对照分开。表中的 dynamic test 只证明当前命令在本机通过；static confirmation 只证明源码形状；design-only 不是运行时证明。

| ID | 章节 | Python 类型/入口 | 测试 | HUSH 对照 | 证明 | 剩余风险 |
|---|---|---|---|---|---|---|
| I01 | CH07 | Identity, EventLog.append | test_cross_run | contracts/tool-protocol/v1 README | dynamic test | 内存 identity，不是跨进程租约 |
| I02 | CH13 | Runner.seen_call_ids | test_runner_rejects_duplicate_call_ids_before_execution | tool-protocol call identity | dynamic test | 仅串行 fake provider |
| I03 | CH13/16 | ToolCall, ModelResponse | test_runner_effect_reserves_before_handler_and_validates_finish | EffectIntent boundary | dynamic test | provider 适配器未接入 |
| I04 | CH11/16 | Validator, finish, RunResult | test_runner_validation_failure_cannot_complete; test_finish_gate | runtime output/validation seam | dynamic test | validator 规则由调用者提供 |
| I05 | CH10 | ToolDefinition, ToolDescriptor | test_wire_round_trip | ToolDescriptor shape | dynamic test | 无跨语言 schema registry |
| I06 | CH10 | ToolDescriptor.kind | test_read_allow_and_unified_diff; test_ask_by_default | read/effect distinction | dynamic test | domain adapter 未实现 |
| I07 | CH10/16 | validate_schema before Runner policy | test_input_schema_precedes_policy_ledger_and_handler | input_schema before policy/ledger | dynamic test | handler 仍是进程内函数 |
| I08 | CH10 | authorize | test_ask_by_default; test_policy_deny_invalid | PolicyDecision | dynamic test | 无真实 capability sandbox |
| I09 | CH13 | Runner + IdempotencyLedger | test_runner_effect_reserves_before_handler_and_validates_finish | reserve-before-execute | dynamic test | durable ledger 未实现 |
| I10 | CH08 | IdempotencyLedger.reserve | test_concurrent_reservation_has_one_executor | reservation ledger | dynamic test | RLock 仅保护单进程 |
| I11 | CH08/10 | intent_digest(EffectIntent) | test_intent_digest_is_stable_and_sensitive_to_arguments | normalized intent digest stability and parameter sensitivity | dynamic test | normalized arguments 仍是 JSON 级 |
| I12 | CH08 | reserve identity/digest check | test_idempotency_identity_and_digest_mismatch | same-key identity mismatch and digest mismatch | dynamic test | 无数据库唯一约束 |
| I13 | CH08 | Outcome.AMBIGUOUS, RecoveryDecision | test_unknown_is_permanent; test_recovery_actions | unknown result fail closed | dynamic test | 未接真实 crash failpoint |
| I14 | CH08 | Outcome enum, LedgerRecord.terminal | test_append_only_terminal_and_reuse | Reserved/InProgress/Completed/Failed/Ambiguous | dynamic test | 只允许教学状态转换 |
| I15 | CH10/13 | ToolResult.error | test_schema_failures (structured schema_error ToolResult); examples/python/m3-agent-loop/test_agent_loop.py::AgentLoopTests::test_tool_failure_becomes_an_observation_and_loop_continues (failed ToolResult retained as an observation and forwarded so the loop continues) | structured tool result | dynamic test | 观测保留策略未配置 |
| I16 | CH07 | EventLog.replay | test_core.CoreTests.test_event_snapshot_roundtrip | replay() 返回事件并保留 identity | dynamic test | 仅证明内存 replay 调用，不是 durable recovery |
| I17 | CH12 | Evidence.supporting_events | test_missing_seq_rejected | Evidence provenance | dynamic test | artifact store 未实现 |
| I18 | CH12 | ReviewBundle.bind | test_review_bundle_changes_when_any_input_changes | ReviewBundle binds ChangeSet/Validation/Evidence | dynamic test | 未有生产审批服务 |
| I19 | CH12 | ReviewBundle.bundle_digest, Approval | test_stale_approval_is_denied | stale approval rejection | dynamic test | caller 必须提供正确摘要字段 |
| I20 | CH12 | Trace, Summary, EvidenceStore | test_trace_filters_run; test_summary_read_projection | read-only projection | dynamic test | 未有并发读模型 |
| I21 | CH14 | TaskSpec, SubRun, TaskResult | test_serial_executor_isolates_parent | parent/child SubRun identity | dynamic test | 同步 executor，无取消传播 |
| I22 | CH14 | SerialDelegateExecutor.allowed_capabilities | capability boundary (static) | capability grant boundary | static confirmation | 不是 OS sandbox |
| I23 | CH14 | aggregate | test_failed_child_rejected; test_stall_and_evidence_gate | structured child result | dynamic test | 时间预算仅字段/计数 |
| I24 | CH15 | CapabilityManifest, Extension.lifecycle | test_activate_revoke_timeout_schema | extension declaration/lifecycle seam | dynamic test | fake adapter，无签名/隔离 |
| I25 | CH15/16 | ExtensionGateway, HarnessProfile | extension gateway (static) | extension cannot bypass runtime/policy | static confirmation | 组合入口未连接真实插件 |

## 证据边界

- Python M4–M10：内存、串行、标准库、DeterministicFakeModel；已运行命令和结果以 implementations.md 与本轮报告为准。
- Rust P0：独立的确定性组合参考；它不能把 Python 各章或 HUSH Runtime 自动标成完成。
- HUSH Runtime：只读对照其 ToolDescriptor、schema-before-policy、reserve-before-execute、Ambiguous 和 ReviewBundle 方向；本轮没有修改、构建或运行 HUSH 仓库。
- 不包含真实 Provider、API key、网络、OS sandbox、durable database、分布式 exactly-once、生产插件隔离或 Forge Studio 集成。

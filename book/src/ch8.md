# 第 8 章：Retry、Resume 与 Rollback

**状态：设计骨架/尚未实现。** 本章定义 M5 recovery 的决策语义和故障注入要求，不实现隐藏重试或 durable storage。

一个工具超时后，最容易写出这样的代码：捕获异常，然后再调用一次。它可能重复付款、重复写文件或重复发送请求。恢复设计必须先回答“失败发生在动作前、动作中，还是动作后”。

## 8.1 学习目标

完成本章后，读者应能区分 retry、resume 和 compensate，识别未知副作用结果，并用 failpoint 验证恢复不会重复执行已确认的动作。

## 8.2 三个词的边界

- **Retry**：同一逻辑操作再次尝试，要求动作可重试或有幂等键。
- **Resume**：从已持久化的事件边界继续，不重新执行已经确认完成的步骤。
- **Rollback / Compensate**：撤销已经发生的影响；这不是把日志指针倒回去。

```text
unknown outcome ≠ safe retry
recorded ToolFinished(success) → resume after it
recorded mutation → compensate with a new audited action
```

## 8.3 用决策代替隐藏分支

```rust
enum RecoveryDecision {
    Retry { attempt: u32, idempotency_key: String },
    Resume { from_sequence: u64 },
    Compensate { action: ChangeSetId },
    Stop { reason: String },
}
```

教学实现可以先只支持 `Resume` 和 `Stop`。P0 明确不做隐藏重试；工具失败作为结构化结果交还模型。未来增加 retry 时，应新增 call id、policy decision 和事件，而不是覆盖原事件。

## 8.4 故障注入与验证

不要用 `sleep` 等待竞态。给测试工具一个 failpoint：在执行前失败、返回后但写事件前失败、写入后进程退出。分别检查：调用次数、事件序列、重启后的动作。跨平台进程崩溃测试需使用可控 failpoint，不能只依赖 Unix `kill`。

一个重要不变量是：`ToolStarted(call_id)` 至多对应一个成功的副作用事实；如果结果未知，系统应进入人工检查或安全停止，而不是乐观重放。

验证场景：在执行前、返回后写事件前和写入后分别注入故障，检查调用次数、事件序列和重启后的动作。未知结果必须进入人工检查或安全停止，不能乐观重放。

## 8.5 本章将得到什么

完成 M5 recovery 实现后，系统将拥有显式 `RecoveryDecision`、幂等键和可控故障注入；这仍不等于跨系统事务或 Exactly-once 外部副作用。

下一章先处理副作用本身：把直接修改改成可审查的 ChangeSet。

## 8.6 恢复不是一个“再试一次”按钮

本章的工程增量主要是决策语义和故障注入合同，而不是一个已经交付的 recovery subsystem。它把 retry、resume、compensate 和 stop 分开，收益是系统不必在未知副作用结果上乐观重试；恢复选择可以带上幂等键、起始 sequence 和审计记录，测试也能针对执行前、写事件前和写事件后的故障分别断言。

代价是恢复无法只由 Harness 自己猜。外部服务是否支持幂等，工具是否能查询结果，补偿动作是否真的等价，都需要工具契约和环境事实。若把“事件已记录”误当成“副作用已提交”，resume 仍会重复动作；若把 rollback 写成倒退日志，又会掩盖已经发生的现实影响。网络超时和模型非确定性会让未知结果成为常态，而不是罕见异常。

| 能力 | 当前状态 |
| --- | --- |
| P0 对工具失败、终止和事件顺序的确定性参考 | P0 参考实现 |
| `RecoveryDecision`、幂等键和 failpoint 驱动的 M5 recovery | 设计骨架/尚未实现 |
| durable recovery、外部 Exactly-once、跨系统补偿 | 尚未实现 |

当前成熟度是 **Experimental**：边界已经足够指导实现和评审，但没有代码证据支持恢复完成度。这里有意留下的技术债是先不隐藏重试，也不伪造外部事务。下一章转而处理更靠前的问题：如果一个动作本身会改文件，就应该先把“将要改变什么”变成可审查的数据，再讨论如何批准和执行。

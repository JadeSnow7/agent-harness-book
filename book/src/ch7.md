# 第 7 章：Session、Task、Run、Step

**状态：设计骨架/尚未实现。** 本章冻结 M5 的身份、事件和 replay 边界；当前 `InMemoryEventStore` 不是 durable recovery。

现在我们有循环和上下文，却仍然容易把四件事混为一谈：用户想完成的任务、一次会话、一次执行尝试，以及其中的一步。进程重启后，如果只有一串聊天文本，就无法回答“这次工具是否已经执行”。

## 7.1 学习目标

完成 M5 后，读者应能区分 Session、Task、Run、Step，解释事件为何是事实源，并设计不重新调用模型或工具的 replay 测试。

## 7.2 四个身份

```text
Session  = 一组相关交互和尝试
Task     = 用户要达成的目标
Run      = Task 的一次执行尝试
Step     = Run 中一次模型决策及其后果
```

同一个 Task 可以有多个 Run；同一个 Run 有多个 Step；Session 可以包含用户修订目标的上下文。P0 当前使用 `SessionId` 和 `RunId`，并为事件分配连续 sequence。这已经是身份边界的最小实现，但 `InMemoryEventStore` 仍是内存存储。

## 7.3 事件而不是可变日志

一次 step 可以产生多个事件：输入构造、模型动作、policy 决策、工具开始、工具结束。事件应追加而不是覆盖：

```rust
struct EventEnvelope {
    run_id: RunId,
    session_id: SessionId,
    sequence: u64,
}
```

状态由 Runner 根据事件顺序推导，Observability 只能投影，不能反过来修改状态。`RunOutcome` 是终态记录；它不能出现在另一个终态之后。

## 7.4 快照与恢复边界

长事件日志每次重放会变慢，因此可以周期性保存 snapshot。但 snapshot 是优化，不是事实源：它必须带上最后应用的 sequence 和 schema version。恢复时先读取快照，再应用其后的事件；发现序号不连续应拒绝，而不是猜测修复。

```text
snapshot(seq=12) + events(13..n) → state
```

当前 P0 只证明内存事件顺序和 replay 检查。不要把它写成 durable recovery。

## 7.5 代码增量与验证

测试创建两个不同 Run，确认事件不会串线；重放完整事件时不调用 model 和 tool；删除中间事件时得到 `ReplayMismatch`。可以用计数器 Fake 验证 replay 期间执行次数仍为零。真实数据库、崩溃注入和跨进程锁属于后续验证，不在本章伪造结果。

当前 P0 参考组合：

```text
Session
 └── Task
      └── Run
           └── Step → EventEnvelope → EventLog
```

验证场景：创建两个不同 Run，确认事件不会串线；重放完整事件时不调用 model 和 tool；删除中间事件时得到 `ReplayMismatch`。真实数据库、崩溃注入和跨进程锁不在本章实现范围。

## 7.6 本章将得到什么

完成 M5 实现后，系统将拥有可区分身份的事件边界和不执行副作用的 replay 入口；这仍不等于崩溃安全的持久化协议。

下一章把 retry、resume 和 rollback 拆开，说明它们对副作用作出的不同承诺。

## 7.7 事件边界带来可追溯性，也带来一致性责任

本章把“历史”提升为带有 `SessionId`、`RunId` 和 sequence 的事件事实。收益是多个任务和多次尝试不必再共享一串含义不明的聊天记录；Runner 可以按事件重建状态，Observability 也只能投影已记录的事实。更重要的是，replay 可以被定义为“不再次调用模型和工具”，而不是重新运行一次任务。

这套边界的代价是写入顺序、身份一致性和 schema 演进都变成了必须维护的契约。事件写了一半时进程崩溃怎么办，快照与事件的版本如何兼容，两个 Run 是否会串线，都是实现问题而非命名问题。内存 EventStore 还无法证明跨进程崩溃安全；如果把 replay 当成重新执行，副作用会被重复放大。

| 当前层 | 状态 |
| --- | --- |
| P0 `EventEnvelope`、sequence、`InMemoryEventStore` 与 replay 校验 | P0 参考实现，已测试 |
| 独立 M5 的 Session/Task/Run/Step API | 设计骨架/尚未实现 |
| durable storage、snapshot 原子性、跨进程恢复 | 尚未实现 |

当前成熟度是 **Prototype（内存范围内）**：事件身份和顺序已有可运行证据，但持久化和崩溃语义尚未成立。有意留下的技术债是把内存日志作为事实源，先验证隔离和 replay 不变量，不提前选择数据库。下一章因此不能简单加一个 `try/except`：要恢复，必须先区分动作尚未发生、已经确认发生和结果未知三种状态。

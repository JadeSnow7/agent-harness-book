# 第 5 章：第一个 Agent Loop

**状态：设计骨架/尚未实现。** 本章冻结 M3 的问题、代码增量和验证场景；当前 P0 的确定性 Runner 只作为参考切片。

第 4 章的 `one_step` 很诚实：第一次模型调用提出工具，Runtime 执行一次，第二次模型调用给出文本。它也留下了一个明显问题：如果第二次响应又提出工具，我们只能报错。模型—工具—模型还没有成为循环。

## 5.1 先把问题说具体

假设模型在第二次响应中再次请求 `read`。第 4 章的 `one_step` 会把它当成错误，因为它只预留了“第一次请求工具、第二次返回文本”这条路径。问题不是再写一个 `if`，而是下一轮必须重新完成同一组动作：构造输入、询问模型、检查动作、执行工具、把观察放回历史。

因此，本章的工程增量可以先写成一句话：

```text
Before: 最多完成一次工具调用。
After: 允许多轮工具调用，但在明确预算和停止原因下结束。
```

完成 M3 后，读者应能用脚本化模型验证这条因果链。当前仓库还没有独立的 M3 实现，因此下面先用已经可运行的 P0 参考切片观察它，再把通用 M3 抽象冻结下来。

## 5.2 从手动两次调用到循环

最直接的改法是把固定的两次调用改成 `loop`：

```text
while budget remains:
  input = build_context(history)
  action = model.next_action(input)
  if action is Finish: validate and stop
  if action is CallTool: policy → execute → append result
stop with BudgetExhausted
```

这里的关键不是 `while` 关键字，而是每次迭代都必须有明确的输入、动作、观察和下一步条件。工具失败也是观察，不应被异常直接吞掉。

```rust
loop {
    let input = context.build(&request, &events, &limits)?;
    let action = model.next_action(input)?;
    match action {
        ModelAction::CallTool(call) => {
            policy.check(&call)?;
            let result = tools.execute(call)?;
            events.push(Event::ToolFinished(result));
        }
        ModelAction::Finish { output } => return validate(output, events),
    }
}
```

这段代码仍是教学草稿。它先回答控制流问题，不提前解决持久化、并行工具或隐藏重试。

## 5.3 运行当前可验证的参考切片

虽然 M3 尚未作为独立 milestone 实现，仓库中的 P0 demo 已经把同一条循环路径接成一个确定性组合。它使用脚本化模型先调用 `echo`，再返回最终文本：

```text
ScriptedMockModel
  1. CallTool(echo, value=7)
  2. Finish("echo completed")
```

运行：

```bash
cargo run -p p0-demo
```

当前验证输出为：

```text
outcome=Completed { ... }
event_count=11
evidence_count=1
```

这里的 `11` 不是性能指标，而是这条确定性路径实际记录的事件数。若要看顺序，运行集成测试：

```bash
cargo test -p p0-demo --test p0_e2e normal_tool_path_records_policy_execution_validation_and_evidence
```

测试断言的关键顺序是：模型输入 → 模型动作 → Policy → 工具开始 → 工具结果 → 第二次模型输入 → 最终动作 → Validation → Evidence → Outcome。这个结果能证明 P0 的组合已经走过多步模型—工具交互；它不能证明独立的 M3 Python 教学实现已经存在。

## 5.4 预算和停止原因

没有预算的循环，在模型持续提出相同工具时永远不会结束。至少需要步数和工具调用数两个边界；还可以加入 token、时间和成本预算。停止原因不能只写在日志里，而应成为结果的一部分：`Completed`、`Failed`、`BudgetExhausted`、`PolicyDenied`、`Cancelled`。

P0 的状态机把 policy denial 作为终止结果，把工具失败作为结构化 `ToolResult`。这两个选择很适合教学，因为它们不会隐藏重试。后续更丰富的 recovery 会显式新增一次调用，而不是让 Runner 在内部偷偷重跑。

## 5.5 状态草图

```mermaid
stateDiagram-v2
    [*] --> Ready
    Ready --> CallingModel
    CallingModel --> ExecutingTool: CallTool
    CallingModel --> Validating: Finish
    ExecutingTool --> CallingModel: ToolResult
    CallingModel --> Terminated: budget/error
    Validating --> Completed: pass
    Validating --> Failed: fail
```

`Finish` 不是成功本身。它只表示模型声明“我想结束”，Harness 仍要经过验证。

## 5.6 M3 的验证合同与当前边界

真正实现 M3 时，最小测试应使用 `ScriptedMockModel` 写出三步：工具调用、再次工具调用、最终 `Finish`；然后再加入一个不可达的第四步，断言 `max_steps` 终止发生在下一次模型动作之前。测试至少要检查：

- 工具结果确实进入下一轮模型输入；
- 每次工具失败仍形成可观察结果；
- 预算耗尽后没有额外模型调用；
- `Finish` 仍然经过 Validation，而不是直接变成 `Completed`。

P0 的 `DeterministicRunner`、`RunLimits` 和 `max_steps` 测试提供了这些不变量的参考，但它把 Context、Policy、Runtime、Validation 和 Evidence 一起组合起来，不能被当作 M3 的独立代码增量。M3 当前仍是设计骨架；下一次实现应先补一个 Python-first 的最小 loop 和离线测试，再同步 Rust 对照与实现索引。

## 5.7 本章将得到什么

读者现在能够指出 `one_step` 为什么不能继续行动，运行 P0 参考切片观察一条真实的多步事件链，并写出 M3 独立实现必须满足的测试合同。独立 M3 尚未完成，因此本章不把设计目标写成已交付能力。

下一章处理循环最容易遇到的失败：历史越积越长，模型不再可靠地看到真正重要的信息。

## 5.8 从一步执行到循环，复杂度第一次明显上升

本章的变化不是把两次调用包进 `while`，而是让 Harness 开始维护一段会影响下一次决策的运行历史。P0 参考切片已经证明一条确定性的多步事件链可以闭合；独立的 M3 教学实现仍未交付，因此这里要把“组合证据”和“里程碑完成”分开。

收益很明确：模型的工具结果可以进入下一轮，工具失败可以成为观察，`max_steps` 和 `max_tool_calls` 可以在模型动作之前截断无限行动，`RunOutcome` 也不再只有成功/异常两种模糊结果。代价是状态机、预算计数和终止原因都成为公共语义；任何一次计数时机错误，都可能多调用一次工具或在验证前提前结束。AI 的错误也会被循环放大：错误工具选择不再只影响一轮，而可能重复污染后续上下文。

| 能力 | 当前状态 |
| --- | --- |
| P0 `DeterministicRunner` 的确定性多步组合 | P0 参考实现，已运行并验证 |
| 独立 Python-first M3 loop | 设计骨架/尚未实现 |
| 步数、工具调用数和终止结果的不变量 | P0 有参考测试；M3 尚未独立验证 |
| Context compaction、recovery、progress detection | 尚未实现 |

当前成熟度是 **Prototype**：P0 组合足以观察循环的事件顺序，但还不能作为通用 Agent Loop 的稳定实现。有意留下的技术债是先只做串行、有界、脚本化的循环，不同时加入并行工具、重试和持久化。下一章由一个实际限制逼出来：循环每多走一轮，历史就多一截；如果没有上下文预算，循环的能力增长会直接转化为输入失控。

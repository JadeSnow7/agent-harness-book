# 第 5 章：第一个 Agent Loop

**状态：已实现并验证。** 本章冻结的 M3 问题、代码增量和验证场景现在保留两条独立的 Python 教学实现（`loop.py` 与 `agent_loop.py`）以及一份 Rust 对照实现（`examples/rust/m3-agent-loop/`），测试矩阵见 [`evals/m3-test-matrix.md`](../../evals/m3-test-matrix.md)。当前 P0 的确定性 Runner 仍只是参考切片，不是 M3 本身——两者的边界见 §5.6。Python M4–M10 的累计包位于 `tutorial/python/agent_harness`，并在后续章节分别验证。

第 4 章的 `one_step` 很诚实：第一次模型调用提出工具，Runtime 执行一次，第二次模型调用给出文本。它也留下了一个明显问题：如果第二次响应又提出工具，我们只能报错。模型—工具—模型还没有成为循环。

## 5.1 先把问题说具体

假设模型在第二次响应中再次请求 `read`。第 4 章的 `one_step` 会把它当成错误，因为它只预留了“第一次请求工具、第二次返回文本”这条路径。问题不是再写一个 `if`，而是下一轮必须重新完成同一组动作：构造输入、询问模型、检查动作、执行工具、把观察放回历史。

因此，本章的工程增量可以先写成一句话：

```text
Before: 最多完成一次工具调用。
After: 允许多轮工具调用，但在明确预算和停止原因下结束。
```

完成 M3 后，读者应能用脚本化模型验证这条因果链。下面先用已经可运行的 P0 参考切片建立直觉，再把 M3 自己的最小实现（Python 与 Rust 各一份，见 §5.6）过一遍。

## 5.2 从手动两次调用到循环

最直接的改法是把固定的两次调用改成 `loop`：

```text
while budget remains:
  input = build_context(history)
  action = model.next_action(input)
  if action is Finish: validate and stop
  if action is CallTool: policy → execute → append result
stop with an explicit reason
```

这里的关键不是 `while` 关键字，而是每次迭代都必须有明确的输入、动作、观察和下一步条件。工具失败也是观察，不应被异常直接吞掉；模型给出的动作也不一定落在“恰好一个工具”或“一段最终文本”这两种预期形状里，循环必须能安全地拒绝它没有被授权自行处理的情况。

这不再是教学草稿——`examples/python/m3-agent-loop/agent_loop.py` 把这条控制流写成了可运行、可测试的 Python，复用第 4 章已经验证的 `ToolRegistry`、`bridge` 和 `Config`/`complete`，只新增循环本身：

```python
 {{#include ../../examples/python/m3-agent-loop/agent_loop.py:m3-agent-loop}}
 ```

目标侧的 Rust 对照实现仍把 Policy 和 Validation 写在循环骨架中：

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

在源侧 Python 原型中，每一轮只处理四种确定的局面，对应四个 `StopReason`：

- 模型没有请求工具、给出了非空最终文本 → `COMPLETED`；
- 达到 `max_steps` 或 `max_tool_calls` → `BUDGET_EXHAUSTED`，且这个判断必须发生在下一次模型调用之前，不能先多打一次电话再后悔；
- 模型一轮给出了不止一个工具候选 → `AMBIGUOUS_TOOL_REQUEST`。循环不会替读者挑一个执行，因为“选哪一个”本身就是一次未经确认的决策；
- 模型响应无法被安全识别——包括 Provider JSON 解码失败、或响应内容既不是工具候选也不是文本 → `UNRECOGNIZED_ACTION`。循环捕获这类异常并安全停止，而不是让整个进程崩溃退出。

源侧 `agent_loop.py` 的这一版只有四个 `StopReason` 变体，没有 `PolicyDenied` 和 `Cancelled`：前者依赖[第 10 章](ch10.md)才会引入的 Policy 引擎，后者依赖[第 7 章](ch7.md)、[第 13 章](ch13.md)才会引入的 Session/取消控制。目标侧 `loop.py` 和 Rust 对照实现则把这两个终态纳入本章的独立合同；读者应把它们看作两条并存、边界不同的教学实现，而不是同一个 API 的两套声明。

## 5.3 两轮闭环：真实测试，不是描述

用一个脚本化 Transport 验证最常见的路径：第一轮模型请求 `read`，Runtime 执行并把结果放回历史，第二轮模型给出最终文本。测试直接断言 `call_id` 全程一致、循环以 `COMPLETED` 停止、第二次请求里确实带着第一轮的观察：

```python
{{#include ../../examples/python/m3-agent-loop/test_agent_loop.py:m3-agent-loop-case}}
```

验证命令：

```bash
python3.11 -m unittest discover -s examples/python/m3-agent-loop -p 'test_*.py'
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

测试断言的关键顺序是：模型输入 → 模型动作 → Policy → 工具开始 → 工具结果 → 第二次模型输入 → 最终动作 → Validation → Evidence → Outcome。这个结果能证明 P0 的组合已经走过多步模型—工具交互；它对应的是 P0 自己的验证场景，不能替代上面的 M3 测试矩阵。仓库里较早的 P0 确定性组合切片走的是同一类多步事件链，但它组合的是 Context、Policy、Validation 和 Evidence 这一整套 P0 边界，不是 M3 的独立实现。

## 5.4 预算和停止原因

没有预算的循环，在模型持续提出相同工具时永远不会结束。`AgentLoopLimits` 定义了两个独立边界：`max_steps` 限制模型调用总轮数，`max_tool_calls` 限制实际执行的工具次数——分开检查是因为“轮数很多但工具用得很少”和“工具用得很多但轮数不多”是两种不同的失控方式，合并成一个数字会掩盖其中一种。

停止原因不能只写在日志里，而应成为结果的一部分。目标侧实现的结果包含 `Completed`、`Failed`、`BudgetExhausted`、`PolicyDenied`、`Cancelled` 五种终态；源侧 `AgentLoopResult.stop_reason` 则是四个 `StopReason` 变体之一，`error` 字段只在解码失败触发 `UNRECOGNIZED_ACTION` 时才非空，保留一条可以安全展示的异常信息，不回显请求头或凭据。测试用一个只有一个响应体的 Transport 验证“预算耗尽后不会多打一次模型调用”：耗尽后如果循环还想再问模型，脚本化 Transport 会因为没有更多响应体可弹出而直接报错，而不是安静地返回空结果——这比断言调用次数更难被绕过。

## 5.5 状态草图

```mermaid
{{#include assets/ch05/agent-loop-state.mmd}}
```

`Completed` 不是验证通过，它只表示循环收到了模型的非空最终文本。这段文本是否真的满足用户目标，验证仍然缺失，是[第 11 章](ch11.md) `Validator` 要处理的问题。

## 5.6 M3 的验证合同与当前边界

M3 的最小测试用 `ScriptedMockModel` 写出三步：工具调用、再次工具调用、最终 `Finish`；再加入一个不可达的第四步，断言 `max_steps` 终止发生在下一次模型动作之前。测试至少要检查：

- 两轮闭环成功，`call_id` 和历史正确传递；
- 步数预算耗尽，且没有多打一次模型调用；
- 工具调用预算耗尽，且没有多执行一次工具；
- 一轮多个工具候选，安全停止且不执行任何一个；
- 工具执行失败（比如目标不存在）仍然进入下一轮，循环不中断；
- Provider 响应解码失败被捕获，循环安全停止而不是让进程崩溃；
- 响应内容既非工具候选也非文本的防御分支。

这份合同现在由目标侧的 Python/Rust 实现满足：`examples/python/m3-agent-loop/test_loop.py`（`python3 examples/python/m3-agent-loop/test_loop.py -v`）和 `examples/rust/m3-agent-loop/`（`cargo test -p m3-agent-loop`）。它们额外覆盖了合同之外、但同样需要明确行为的边界：重复 `call_id`、未知工具、协作式 `Cancel`，以及 `Completed`/`Failed`/`BudgetExhausted`/`PolicyDenied`/`Cancelled` 五种终态互斥且都可达。

P0 的 `DeterministicRunner`、`RunLimits` 和 `max_steps` 测试仍然是这些不变量最早的参考，但它把 Context、Policy、Runtime、Validation 和 Evidence 一起组合起来，不是 M3 的独立代码增量——M3 两侧实现都是从这份合同重新写起：循环控制流本身不依赖 `DeterministicRunner`，只复用了各自语言里与 P0 编排层无关的基础类型（工具执行原语）。测试矩阵的逐条 Python/Rust 对应见 [`evals/m3-test-matrix.md`](../../evals/m3-test-matrix.md)。

源侧的 `examples/python/m3-agent-loop/test_agent_loop.py` 另有 7 个测试，专门验证 `agent_loop.py` 的四种 `StopReason`、`call_id` 传递、工具失败观察、预算边界和 Provider 解码失败。它还直接 mock `complete()` 来覆盖“响应既不是工具候选也不是文本”的防御分支；这是对该 Python 原型自身的测试，不应被解释为目标侧五终态实现的重复证据。两条 Python 实现都保留在合并后的目录中，分别使用各自的入口和测试命令。

## 5.7 本章将得到什么

读者现在能够指出 `one_step` 为什么不能继续行动，运行 P0 参考切片观察一条真实的多步事件链，并对照 §5.6 的测试合同运行 M3 自己的 Python 和 Rust 实现——两者都是可以直接执行、离线验证的代码，不是设计目标的复述。

下一章处理循环最容易遇到的失败：历史越积越长，模型不再可靠地看到真正重要的信息。

## 5.8 从一步执行到循环，复杂度第一次明显上升

本章的变化不是把两次调用包进 `while`，而是让 Harness 开始维护一段会影响下一次决策的运行历史。P0 参考切片证明了一条确定性的多步事件链可以闭合；目标侧 M3 实现交付了预算、Policy、Validation、取消和五种终态的完整不变量；源侧 Python 原型则把 Provider 解码和四种安全停止原因单独讲清楚。这些是三份不同的证据，不能相互替代。

收益很明确：模型的工具结果可以进入下一轮，工具失败可以成为观察，`max_steps` 和 `max_tool_calls` 可以在模型动作之前截断无限行动，停止原因不再只有成功/异常两种模糊结果，而是按实现分成可测试的显式变体。代价是状态机、预算计数和终止原因都成为公共语义；任何一次计数时机错误，都可能多调用一次工具或多打一次电话。AI 的错误也会被循环放大：错误工具选择不再只影响一轮，而可能重复污染后续上下文——这也是为什么“一轮多个候选”和“无法识别的响应”都选择立即安全停止，而不是让循环替读者做决定。

| 能力 | 当前状态 |
| --- | --- |
| P0 `DeterministicRunner` 的确定性多步组合 | P0 参考实现，已运行并验证 |
| 独立 M3 loop（目标侧 Python + Rust） | 已实现并验证，见 §5.6 |
| 源侧 Python M3 原型（`agent_loop.py`） | 已实现并验证，见 §5.6 |
| 步数、工具调用数和终止结果的不变量 | P0 有参考测试；M3 独立测试矩阵已覆盖并验证 |
| Python M4–M10 累计教学包 | 已实现并验证；内存/fake-provider 教学切片，见后续章节 |
| Context compaction、recovery、progress detection | 尚未实现 |

当前成熟度是 **Prototype**：P0 组合、目标侧 M3 实现和源侧 Python 原型都足以观察循环的事件顺序和终止不变量，但都还不能作为通用 Agent Loop 的稳定实现。Python M4–M10 是后续章节中的内存/fake-provider 教学包，也不表示 durable recovery、真实 Provider、OS sandbox 或 HUSH Runtime 集成。有意留下的技术债是先只做串行、有界、脚本化的循环，不同时加入并行工具、重试和持久化。下一章由一个实际限制逼出来：循环每多走一轮，历史就多一截；如果没有上下文预算，循环的能力增长会直接转化为输入失控。

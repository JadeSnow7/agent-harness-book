# 第 5 章：第一个 Agent Loop

**状态：M3 Python 原型已实现并验证；Rust 独立示例和累计 Rust 工程增量仍是设计骨架/尚未实现。** 本章的 `examples/python/m3-agent-loop/` 是可以离线跑通的真实代码；Python M4–M10 的累计包位于 tutorial/python/agent_harness，并在后续章节分别验证。

第 4 章的 `one_step` 很诚实：第一次模型调用提出工具，Runtime 执行一次，第二次模型调用给出文本。它也留下了一个明显问题：如果第二次响应又提出工具，我们只能报错。模型—工具—模型还没有成为循环。

## 5.1 先把问题说具体

假设模型在第二次响应中再次请求 `read`。第 4 章的 `one_step` 会把它当成错误，因为它只预留了“第一次请求工具、第二次返回文本”这条路径。问题不是再写一个 `if`，而是下一轮必须重新完成同一组动作：构造输入、询问模型、检查动作、执行工具、把观察放回历史。

因此，本章的工程增量可以先写成一句话：

```text
Before: 最多完成一次工具调用。
After: 允许多轮工具调用，但在明确预算和停止原因下结束。
```

## 5.2 从手动两次调用到循环

最直接的改法是把固定的两次调用改成 `loop`：

```text
while budget remains:
  input = build_context(history)
  action = model.next_action(input)
  if action is Finish: stop
  if action is CallTool: execute → append result
stop with an explicit reason
```

这里的关键不是 `while` 关键字，而是每次迭代都必须有明确的输入、动作、观察和下一步条件。工具失败也是观察，不应被异常直接吞掉；模型给出的动作也不一定落在“恰好一个工具”或“一段最终文本”这两种预期形状里，循环必须能安全地拒绝它没有被授权自行处理的情况。

这不再是教学草稿——`examples/python/m3-agent-loop/agent_loop.py` 把这条控制流写成了可运行、可测试的 Python，复用第 4 章已经验证的 `ToolRegistry`、`bridge` 和 `Config`/`complete`，只新增循环本身：

```python
{{#include ../../examples/python/m3-agent-loop/agent_loop.py:m3-agent-loop}}
```

每一轮只处理四种确定的局面，对应四个 `StopReason`：

- 模型没有请求工具、给出了非空最终文本 → `COMPLETED`；
- 达到 `max_steps` 或 `max_tool_calls` → `BUDGET_EXHAUSTED`，且这个判断必须发生在下一次模型调用之前，不能先多打一次电话再后悔；
- 模型一轮给出了不止一个工具候选 → `AMBIGUOUS_TOOL_REQUEST`。循环不会替读者挑一个执行，因为“选哪一个”本身就是一次未经确认的决策；
- 模型响应无法被安全识别——包括 Provider JSON 解码失败、或响应内容既不是工具候选也不是文本 → `UNRECOGNIZED_ACTION`。循环捕获这类异常并安全停止，而不是让整个进程崩溃退出。

`StopReason` 这一版只有这四个变体，没有 `PolicyDenied` 和 `Cancelled`：前者依赖[第 10 章](ch10.md)才会引入的 Policy 引擎，后者依赖[第 7 章](ch7.md)、[第 13 章](ch13.md)才会引入的 Session/取消控制。读者现在不需要理解、也不需要实现它们——它们不是本章遗漏，只是还没轮到。

## 5.3 两轮闭环：真实测试，不是描述

用一个脚本化 Transport 验证最常见的路径：第一轮模型请求 `read`，Runtime 执行并把结果放回历史，第二轮模型给出最终文本。测试直接断言 `call_id` 全程一致、循环以 `COMPLETED` 停止、第二次请求里确实带着第一轮的观察：

```python
{{#include ../../examples/python/m3-agent-loop/test_agent_loop.py:m3-agent-loop-case}}
```

验证命令：

```bash
python3.11 -m unittest discover -s examples/python/m3-agent-loop -p 'test_*.py'
```

仓库里较早的 P0 确定性组合切片（`cargo run -p p0-demo`）走的是同一类多步事件链，但它组合的是 Context、Policy、Validation 和 Evidence 这一整套 P0 边界，不是本章独立验证的教学实现；能用来提前感受“模型—工具—模型”反复出现是什么样子，不能替代上面这条命令作为 M3 的证据。

## 5.4 预算和停止原因

没有预算的循环，在模型持续提出相同工具时永远不会结束。`AgentLoopLimits` 定义了两个独立边界：`max_steps` 限制模型调用总轮数，`max_tool_calls` 限制实际执行的工具次数——分开检查是因为“轮数很多但工具用得很少”和“工具用得很多但轮数不多”是两种不同的失控方式，合并成一个数字会掩盖其中一种。

停止原因不能只写在日志里，而应成为结果的一部分：`AgentLoopResult.stop_reason` 是四个 `StopReason` 变体之一，`error` 字段只在解码失败触发 `UNRECOGNIZED_ACTION` 时才非空，保留一条可以安全展示的异常信息，不回显请求头或凭据。测试用一个只有一个响应体的 Transport 验证“预算耗尽后不会多打一次模型调用”：耗尽后如果循环还想再问模型，脚本化 Transport 会因为没有更多响应体可弹出而直接报错，而不是安静地返回空结果——这比断言调用次数更难被绕过。

## 5.5 状态草图

```mermaid
{{#include assets/ch05/agent-loop-state.mmd}}
```

`Completed` 不是验证通过，它只表示循环收到了模型的非空最终文本。这段文本是否真的满足用户目标，验证仍然缺失，是[第 11 章](ch11.md) `Validator` 要处理的问题。

## 5.6 M3 的验证合同与当前边界

`examples/python/m3-agent-loop/test_agent_loop.py` 目前有 7 个测试，覆盖：

- 两轮闭环成功，`call_id` 和历史正确传递；
- 步数预算耗尽，且没有多打一次模型调用；
- 工具调用预算耗尽，且没有多执行一次工具；
- 一轮多个工具候选，安全停止且不执行任何一个；
- 工具执行失败（比如目标不存在）仍然进入下一轮，循环不中断；
- Provider 响应解码失败被捕获，循环安全停止而不是让进程崩溃；
- 响应内容既非工具候选也非文本的防御分支。

最后一条需要额外说明：M1 的 `decode_response` 保证只要成功解码，响应必然满足“有工具候选或有非空文本”之一，这条分支用真实的脚本化 Transport 造不出来，测试直接 mock 了 `complete()` 的返回值来验证 `run_agent_loop` 自身的分支逻辑，不是伪装成一条端到端场景。这不是本章的漏洞，而是刻意保留的防御——`run_agent_loop` 不应该假设未来所有 Provider 适配器都维持同一条不变量。

Rust 累计工程尚未跟进这条增量；cargo test -p tutorial-agent-harness --offline 覆盖的仍是第 4 章的一步闭环，不包含循环。Python M3–M10 使用实现索引中的独立命令。

## 5.7 本章将得到什么

读者现在能够运行一个真实的、有预算的多轮工具循环，指出它在四种局面下分别做什么、为什么这么做，并且知道哪些看起来相关的能力（Policy、取消、验证）是有意留给后面章节的，不是被遗忘的坑。Rust 独立示例和累计工程的对应增量还没有开始，这是下一次任务要还的债，不在本章内混着说完。

下一章处理循环最容易遇到的失败：历史越积越长，模型不再可靠地看到真正重要的信息。

## 5.8 从一步执行到循环，复杂度第一次明显上升

本章的变化不是把两次调用包进 `while`，而是让 Harness 开始维护一段会影响下一次决策的运行历史，并且要为“这一轮该怎么办”给出四选一的明确答案。

收益很明确：模型的工具结果可以进入下一轮，工具失败可以成为观察，`max_steps` 和 `max_tool_calls` 可以在模型动作之前截断无限行动，停止原因不再只有成功/异常两种模糊结果，而是四个各自可测试的显式变体。代价是状态机、预算计数和终止原因都成为公共语义；任何一次计数时机错误，都可能多调用一次工具或多打一次电话。AI 的错误也会被循环放大：错误工具选择不再只影响一轮，而可能重复污染后续上下文——这也是为什么“一轮多个候选”和“无法识别的响应”都选择立即安全停止，而不是让循环替读者做决定。

| 能力 | 当前状态 |
| --- | --- |
| M3 Python 原型（`run_agent_loop`、四个 `StopReason`） | 已实现并验证 |
| P0 `DeterministicRunner` 的确定性多步组合 | P0 参考实现，已运行并验证，但不是本章证据 |
| Rust 独立示例（`examples/rust/m3-agent-loop`） | 设计骨架/尚未实现 |
| Rust 累计工程增量（`tutorial/agent-harness/`） | 设计骨架/尚未实现 |
| Context compaction、recovery、progress detection | 尚未实现 |

当前成熟度是 **Prototype**：Python 原型足以让读者跑通并测试一条真实的多轮循环，但还不是可以直接复用的稳定实现，Rust 两条轨道也还没跟上。有意留下的技术债是先只做串行、有界、脚本化的循环，不同时加入并行工具、重试和持久化。下一章由一个实际限制逼出来：循环每多走一轮，历史就多一截；如果没有上下文预算，循环的能力增长会直接转化为输入失控。

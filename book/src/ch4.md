# 第 4 章：从一次工具候选到一步闭环

**状态：已实现并验证。** 第 3 章的 `ToolUseBlock` 仍然只是数据。本章让 Runtime 校验并执行一个 `read`，再把结果按原 `call_id` 送回模型；4.5 节额外用 `write` 演示一次可见的副作用，并说明它为什么是教学捷径而非推荐设计。

本章刻意只完成固定两次模型调用的一步闭环：第一次提出一个工具候选，Runtime 执行一次，第二次返回最终文本。它没有循环、预算、重试或停止策略。

## 4.1 连续案例：读取 `hello.txt`

```text
模型：ToolUseBlock(id="call_42", name="read", input={path: "hello.txt"})
        ↓
Registry 查找并校验
        ↓
Workspace 内执行 read
        ↓
ToolResult(call_id="call_42", content="1: hello workspace")
        ↓
第二次模型调用：返回最终文本
```

模型输出调用并不会自动获得磁盘权限。Runtime 负责稳定执行和报告；Policy、审批和 OS 隔离属于后续边界。

## 4.2 Runtime 的最小契约

```text
ToolSpec   = { name, description, input_schema, strict }
ToolCall   = { call_id, name, arguments }
ToolResult = { call_id, name, status, output?, error? }
```

未知工具、参数错误和工具执行失败都收敛为结构化 `ToolResult`。上层因此可以观察失败，而不是因为某个工具异常直接丢失整个运行。

## 4.3 Registry：先校验，再执行

下面是现有 Python Runtime 的核心执行片段：

```python
{{#include ../../examples/python/m2-tool-runtime/registry.py:m2-registry-execute}}
```

Registry 不解析 Provider JSON，也不替 Policy 做批准决定。它只处理工具查找、参数校验、执行和失败收敛。

## 4.4 `read`：工作区内的有界观察

```python
{{#include ../../examples/python/m2-tool-runtime/tools/read.py:m2-read-tool}}
```

`read` 的关键边界是：路径必须解析到 Workspace 内，读取有字节和行数上限，输出带行号，失败变成结构化结果。路径检查不是生产级沙箱，也不能消除所有 TOCTOU、挂载点或进程权限风险。

## 4.5 `write`：一次可见的副作用（教学捷径，非推荐范式）

```python
{{#include ../../examples/python/m2-tool-runtime/tools/write.py:m2-write-tool}}
```

`write` 把模型给出的完整内容原子写入 Workspace 内的目标路径：先在同目录写临时文件，`fsync` 后 `os.replace`，失败时不留下半截文件。加入它是因为只读一次很难让读者确认 Runtime 真的在起作用；写入之后再读回同一个文件，才有一个可以肉眼检查的前后变化：

```python
{{#include ../../examples/python/m2-tool-runtime/test_runtime.py:m2-writing-case}}
```

**这不是推荐的生产设计。** 这里模型的写入意图被直接连到执行，中间没有任何审查：没有 diff、没有原因、没有人工或 Policy 把关，写完就是写完。真实 Harness 应该先把这个意图变成[第 9 章](ch9.md)的 `ChangeSet`——带原内容指纹、diff 和风险级别，经 Policy/审批后才 apply。本章不实现那套审查机制，只是先证明 Runtime 能够稳定执行一次副作用并把结果原样回报给模型；把"能执行"和"该不该被审查后再执行"分开，是刻意的教学顺序，不是疏漏。

## 4.6 M1 / M2 bridge

桥接必须是显式转换，而不是让 Runtime 开始解析 Provider JSON：

```text
ToolDefinition → ToolSpec
ToolUseBlock   → ToolCall
ToolResult     → ToolResultBlock → role=tool Message
```

最重要的不变量是 `call_id`：`ToolUseBlock.id` 进入 `ToolCall.call_id`，执行后再成为 `ToolResultBlock.tool_use_id`，最终由 M1 编码成 `function_call_output.call_id`。

## 4.7 固定一步闭环

真实的教学函数使用 Fake Transport 调用两次：

```python
{{#include ../../examples/python/m2-tool-runtime/one_step.py:m2-one-step}}
```

第一次没有工具、第一次有多个工具或第二次再次请求工具，都会立即失败；函数不会偷偷进入循环。第二次请求必须看见第一轮的 `ToolResult`，否则模型没有机会根据观察作出最终回答。

一个连续案例测试保留了相同的 `call_id`，并检查第二次请求中确实出现 `function_call_output`：

```python
{{#include ../../examples/python/m2-tool-runtime/test_runtime.py:m2-reading-case}}
```

## 4.8 局部 postcondition

一步闭环可以检查“工具成功”“输出包含某段文本”“文件存在”等局部条件，但这不是 M7 的任务级 Validation，也不是 Evidence。它只回答一次工具调用后，局部状态是否符合预期。

## 4.9 验证与下一步

```bash
python3.11 -m py_compile \
  examples/python/m2-tool-runtime/*.py \
  examples/python/m2-tool-runtime/tools/*.py
python3.11 -m unittest discover \
  -s examples/python/m2-tool-runtime \
  -p 'test_*.py'
```

Rust 对照实现、完整七工具目录、Workspace 原子写入、glob、bash 安全说明和完整失败矩阵见 [M2 Tool Runtime 实验](labs/m2-tool-runtime.md) 与[实现索引](implementations.md)。

M1 让系统听懂工具候选，M2 让系统稳定地做一次并返回观察。下一章 M3 才会加入有界循环、步数预算、停止原因和重复调用策略。

### 累计 Rust 工程对照

累计工程 `tutorial/agent-harness/` 把同一条边界落到 Rust，但只注册 `read` 一个工具：`ToolRegistry` 负责查找、参数校验和失败收敛，`Workspace` 只提供 `resolve` 和相对路径转换（不含 `write`/`edit` 需要的原子替换），`bridge` 模块是唯一同时认识 Runtime 类型和 M1 协议类型的地方，`one_step::run_one_tool_step` 实现固定两次调用的闭环。`call_id` 从 `ToolUseBlock` 经 `ToolCall`、`ToolResult` 到 `ToolResultBlock` 全程一致，可用离线测试验证：

```bash
cargo test -p tutorial-agent-harness --offline
```

下面两张图分别是这条闭环的时序，以及 `read` 从候选到结果的失败分类：

```mermaid
{{#include assets/ch04/one-step-closure.mmd}}
```

```mermaid
{{#include assets/ch04/tool-call-failure-paths.mmd}}
```

这仍然是固定一步、不是循环——第二次模型响应如果还想要工具，闭环会显式报错而不是继续。4.5 节演示的 `write` 来自这里的 Python 实现，累计 Rust 工程尚未并入它；ls/find/grep/edit/bash 五个工具和原子写入同样仍只留在 `examples/` 与 [M2 Tool Runtime 实验](labs/m2-tool-runtime.md)。实现与离线测试索引见 [实现索引](implementations.md)。

## 4.10 第一个闭环的工程账本

本章前，模型只能提出一个结构化候选；本章后，`ToolRegistry`、`Workspace` 和 `read` 让一次动作经过校验、执行并把原 `call_id` 的观察返回给模型。收益是系统第一次真正改变了“模型只能输出文本”的边界：工具是否存在、路径是否越过工作区、读取是否成功，都可以由 Runtime 产生可观察结果，而不是让模型猜测。

这个收益不是免费的。Registry 增加了注册和参数校验的耦合；Workspace 边界若只检查字符串路径，仍需面对 symlink、TOCTOU 和权限问题；工具失败、超时或副作用未知时，一步闭环没有恢复协议。AI 还可能选择不存在的工具、传入错误参数，或在拿到工具结果后仍然给出未经验证的结论。当前的局部 postcondition 只验证一次动作，不能替代任务级 Validator。`write` 引入的是一类新风险：模型的写入意图未经任何审查就被执行，这是本章刻意留下、留给[第 9 章](ch9.md) `ChangeSet` 去解决的技术债，不是被忽略的问题。

| 能力 | 当前状态 |
| --- | --- |
| Python/Rust M2 一步 Tool Runtime 与 `hello.txt` 案例 | 已实现并验证 |
| Rust（累计工程）M2 一步 Tool Runtime（仅 `read`） | 已实现并验证 |
| `write` 单次可见副作用（教学捷径，Python） | 已实现并验证，非推荐生产范式 |
| 七工具、Workspace 和失败矩阵 | 已实现并验证，仍是教学边界 |
| 多轮 Loop、预算、恢复、审批和 ChangeSet | Python M3–M10 已实现并验证；Rust/P0 对应能力按实现索引区分 |
| 生产级沙箱与外部副作用控制 | 尚未实现 |

当前成熟度是 **Prototype**：这条路径可运行、可测试、能展示真实的模型—工具—观察闭环，但只支持固定一步；`write` 能让读者看到一次真实的文件改变，却不能被误称为完整变更审查系统——审查在[第 9 章](ch9.md)才出现。有意留下的技术债是把执行次数限制在一步、把写入和审查分开，以便先看清 `call_id`、路径和失败回传。下一章的真实问题已经出现：第二次模型响应如果仍然提出工具，固定流程就无法继续；系统需要循环，但循环也必须带预算和停止原因。

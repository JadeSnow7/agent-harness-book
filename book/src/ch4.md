# 第 4 章：从一次工具候选到一步闭环

**状态：已实现并验证。** 第 3 章的 `ToolUseBlock` 仍然只是数据。本章让 Runtime 校验并执行一个 `read`，再把结果按原 `call_id` 送回模型。

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

## 4.5 M1 / M2 bridge

桥接必须是显式转换，而不是让 Runtime 开始解析 Provider JSON：

```text
ToolDefinition → ToolSpec
ToolUseBlock   → ToolCall
ToolResult     → ToolResultBlock → role=tool Message
```

最重要的不变量是 `call_id`：`ToolUseBlock.id` 进入 `ToolCall.call_id`，执行后再成为 `ToolResultBlock.tool_use_id`，最终由 M1 编码成 `function_call_output.call_id`。

## 4.6 固定一步闭环

真实的教学函数使用 Fake Transport 调用两次：

```python
{{#include ../../examples/python/m2-tool-runtime/one_step.py:m2-one-step}}
```

第一次没有工具、第一次有多个工具或第二次再次请求工具，都会立即失败；函数不会偷偷进入循环。第二次请求必须看见第一轮的 `ToolResult`，否则模型没有机会根据观察作出最终回答。

一个连续案例测试保留了相同的 `call_id`，并检查第二次请求中确实出现 `function_call_output`：

```python
{{#include ../../examples/python/m2-tool-runtime/test_runtime.py:m2-reading-case}}
```

## 4.7 局部 postcondition

一步闭环可以检查“工具成功”“输出包含某段文本”“文件存在”等局部条件，但这不是 M7 的任务级 Validation，也不是 Evidence。它只回答一次工具调用后，局部状态是否符合预期。

## 4.8 验证与下一步

```bash
python3 -m py_compile \
  examples/python/m2-tool-runtime/*.py \
  examples/python/m2-tool-runtime/tools/*.py
python3 -m unittest discover \
  -s examples/python/m2-tool-runtime \
  -p 'test_*.py'
```

Rust 对照实现、完整七工具目录、Workspace 原子写入、glob、bash 安全说明和完整失败矩阵见 [M2 Tool Runtime 实验](labs/m2-tool-runtime.md) 与[实现索引](implementations.md)。

M1 让系统听懂工具候选，M2 让系统稳定地做一次并返回观察。下一章 M3 才会加入有界循环、步数预算、停止原因和重复调用策略。

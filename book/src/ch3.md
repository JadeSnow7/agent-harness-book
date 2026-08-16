# 第 3 章：从 Provider JSON 到统一协议

**状态：已实现并验证。** 第 2 章的上层代码仍然知道 `input`、`output` 和 `output_text`。一旦接入工具或更换 Provider，这些字段就会扩散到 Context、Runtime 和测试里。

本章只做一件事：建立 Harness 内部的最小语言。模型可以返回文本，也可以提出一个 `read` 工具候选，但任何工具都还没有执行。

## 3.1 连续案例：把“读取文件”表示成数据

上一章的请求是：读取工作区 `hello.txt` 的第一行。现在我们不再要求模型输出一段不可解析的命令，而是给它一个工具声明：

```text
ToolDefinition = {
  name: "read",
  description: "Read a file inside the workspace",
  input_schema: { type: "object", required: ["path"] }
}
```

模型返回的内容仍然只是候选动作：

```text
ToolUseBlock { id: "call_42", name: "read", input: { path: "hello.txt" } }
```

`ToolUseBlock` 不代表动作已经获准或执行。它只是下一层 Runtime 可以检查的数据。

## 3.2 统一协议隔离 Provider

没有统一协议时：

```text
业务代码 → OpenAI JSON
业务代码 → 另一家 Provider JSON
测试代码 → 模仿每一家 JSON
```

加入协议后：

```text
业务代码 → ModelRequest / ModelResponse
                    ↓
             Provider Adapter
                    ↓
              Provider JSON
```

适配器可以变化，Tool Runtime 看到的 `ToolUseBlock` 不变。协议层不是为了制造“大一统框架”，而是让 Provider 差异停留在一个可测试的边界。

## 3.3 固定最小协议

```text
ContentBlock = TextBlock | ToolUseBlock | ToolResultBlock
Message      = { role, content[] }
ToolDefinition = { name, description, input_schema, strict }
ModelRequest  = { model, system?, messages[], tools[] }
ModelResponse = { id?, model?, message, status? }
```

工具结果必须保留原始 `call_id`：

```text
ToolUseBlock.id
      ↓
ToolCall.call_id
      ↓
ToolResultBlock.tool_use_id
      ↓
function_call_output.call_id
```

错误也按发生位置区分：配置错误、传输错误、Provider 错误、编码错误和解码错误。错误可以暴露类别与 HTTP 状态码，但不能回显 API Key、认证 Header 或完整响应正文。

## 3.4 Python 实现：先定义内部对象

下面的类型来自 `examples/python/m1-unified-protocol/protocol.py`。它们不发送网络，也不执行工具。

```python
{{#include ../../examples/python/m1-unified-protocol/protocol.py:m1-protocol-types}}
```

`ToolDefinition` 在请求离开进程前检查工具名、描述、schema 和 `strict`；`ModelRequest` 拒绝空消息和重复工具名。模型侧的 strict 约束不能替代 Runtime 的参数校验。

## 3.5 Python 实现：请求与响应适配

统一请求由 `build_model_request` 生成：

```python
{{#include ../../examples/python/m1-unified-protocol/chat_once.py:m1-build-request}}
```

真正的 Provider 细节由 `complete` 负责：

```python
{{#include ../../examples/python/m1-unified-protocol/chat_once.py:m1-complete}}
```

适配器的工作是把 Responses 的 `function_call` 解码成 `ToolUseBlock`，把后续的 `function_call_output` 编码成 `ToolResultBlock`。这组对象的关联以官方函数调用指南为准。[^function-calling] 本章不执行 `read`，也不把 reasoning 或未知输出项目伪装成统一内容块。

为了观察文本和工具候选，命令行只需要一个格式化函数：

```python
{{#include ../../examples/python/m1-unified-protocol/chat_once.py:m1-format-response}}
```

连续案例在协议测试中变成一个带 `call_id` 和路径参数的候选：

```python
{{#include ../../examples/python/m1-unified-protocol/test_protocol.py:m1-reading-case}}
```

## 3.6 离线测试形成什么证据

| 测试目标 | 可证明什么 |
|---|---|
| 空模型、空消息、非法或重复工具 | 歧义请求不会进入传输层 |
| 工具字段和注册顺序 | `ToolDefinition` 映射稳定 |
| 文本、拒答、function call | 主要输出可以解码 |
| 缺少 call id/name、非法参数、空输出 | 失败不会被当成成功文本 |
| tool result 编码 | 原始 `call_id` 没有丢失 |
| Fake Transport | URL、Header、payload 和 timeout 正确 |
| HTTP/JSON 安全错误 | 错误信息不泄露敏感内容 |

验证命令：

```bash
python3.11 -m py_compile examples/python/m1-unified-protocol/*.py
python3.11 -m unittest discover \
  -s examples/python/m1-unified-protocol \
  -p 'test_*.py'
```

Rust 对照实现和测试只通过[实现索引](implementations.md)引用。这样正文先讲协议语义，再让读者按需要切换语言，不让 ownership 或 trait 语法打断第一次理解。

## 3.7 本章小结

M1 让 Harness 拥有一套稳定语言：模型可以回复文本，也可以提出结构化候选动作。下一章会把 `ToolUseBlock` 桥接到 Runtime，执行恰好一个 `read`，再以相同 `call_id` 返回观察。

这仍不是 Agent Loop。循环、预算、重试和停止策略属于后续章节。

[^function-calling]: OpenAI, [Function calling guide](https://developers.openai.com/api/docs/guides/function-calling)，核验日期：2026-08-08。

## 3.8 统一协议的收益不是“类型更漂亮”

M0 之后，系统已经能得到 Provider 响应，但上层若继续依赖厂商字段，就会在文本、拒答、工具候选和工具结果之间形成多套分支。本章把这些内容收敛成 `Message`、`ContentBlock`、`ToolDefinition` 和统一错误，收益是 Agent Loop 可以消费同一套内部对象；Fake Model 和协议测试也能直接构造 `ToolUseBlock`，不必伪造真实网络响应。

这次收敛的代价是信息可能被抹平。Provider 的特殊字段、调用粒度和错误语义如果没有明确映射，就会在适配层悄悄丢失；协议一旦被上层大量依赖，后续修改会变成兼容性问题。AI 的非确定性还会把缺少 `call_id`、错误参数和混合内容块带到边界，若解析器把它们当普通文本，错误会继续向执行层放大。

| 协议能力 | 当前状态 |
| --- | --- |
| Python/Rust M1 消息、内容块和工具定义 | 已实现并验证 |
| 文本、拒答、function call、tool result 的离线映射 | 已实现并验证 |
| 多 Provider 的真实兼容矩阵与流式语义 | 设计骨架/尚未实现 |
| 工具候选的实际执行 | 尚未完成，留给 M2 |

当前成熟度可判为 **Usable（教学范围内）**：协议已经能支撑下一步工具闭环，测试覆盖了关键失败形状；但公共 API 仍按仓库决策属于实验性，不能因为“统一”就宣称稳定或生产就绪。有意留下的技术债是保持最小协议，不提前封装厂商全部能力。下一章必须把 `ToolUseBlock` 变成受控的 `ToolCall`，否则统一协议只是在内存里换了名字。

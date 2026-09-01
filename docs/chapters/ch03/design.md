# ch3 累计 Rust M1 设计

本切片在 `tutorial/agent-harness` 的 ch2 单次调用上引入 Provider-neutral 统一协议。数据流为：

```text
ModelRequest -> OpenAI Responses adapter -> Transport -> adapter -> ModelResponse
```

`ModelRequest`、`Message`、`ContentBlock` 和 `ToolDefinition` 是上层契约；OpenAI 的 `input_text`、`output_text`、`function_call` 与 `function_call_output` 只存在于 adapter 边界。`call_id` 从 `ToolUseBlock.id` 原样贯通到 `ToolResultBlock.tool_use_id` 和 provider 输出。

本章只把 `read hello.txt` 解码为候选数据，不执行工具。`is_error` 是内部元数据，当前没有宣称对应的 Provider 独立错误映射。reasoning 和未知 provider 项目会被跳过，但若没有任何可识别内容则解码失败；拒答文本只按可展示文本处理，不声称保留完整拒答语义。

离线 Fake Transport 验证 URL、headers、payload 和 timeout，并覆盖构造校验、编解码、参数形状、错误安全和候选 call id。非目标包括工具 Runtime、Workspace、循环、流式、重试、多 Provider 真实兼容和稳定公共 API。

# ch4 设计边界：从一次工具候选到一步闭环

本章在 `tutorial/agent-harness` 的 ch3 统一协议之上新增 Runtime 层，只注册 `read` 一个工具。

## 保留的边界

- `ToolCall`/`ToolResult`/`ToolStatus`/`ToolError` 是 Runtime 内部的最小结构化类型，独立于 `protocol::ToolUseBlock`/`ToolResultBlock`；两者只通过 `bridge` 模块的显式转换函数往来，不合并成同一组类型。
- `ToolRegistry.execute` 把"未注册工具""参数校验失败""执行失败"统一收敛成结构化 `ToolResult`，不把工具业务失败抬升为未捕获异常或 panic。
- `Workspace` 只提供 `read` 真正需要的能力：`resolve` 把用户路径解析并限制在固定 root 内，拒绝 `..`、绝对路径和经尚不存在父目录发生的越界；`relative_to_root` 把结果格式化为相对路径。不包含 `write`/`edit` 需要的原子替换。
- 本章只注册并使用 `read`；`ls`/`find`/`grep`/`edit`/`bash` 仍只存在于 `examples/` 和 [M2 Tool Runtime 实验](../../book/src/labs/m2-tool-runtime.md)，不并入累计工程。`write` 在 ch4.md 4.5 节作为教学捷径单独演示（模型写入意图未经审查直接执行，用于让读者看到一次可肉眼检查的副作用），出自 `examples/python/m2-tool-runtime` 的既有实现，同样不并入 `tutorial/agent-harness`。
- `one_step::run_one_tool_step` 固定两次模型调用：第一次必须恰好请求一个工具，Runtime 执行后把观察写回，第二次必须给出最终文本、不得再请求工具；这不是 Agent Loop，没有预算、重试或多轮工具调用。
- `call_id` 从 `ToolUseBlock.id` 原样贯通到 `ToolCall.call_id`、`ToolResult.call_id` 和 `ToolResultBlock.tool_use_id`，最终由 M1 编码成 `function_call_output.call_id`；由离线集成测试直接断言编码后的请求体。

## 与 ch3 遗留问题的处理

`tutorial/agent-harness/src/openai_responses.rs` 中原有一个未被测试或调用的 `tool_result_message` 函数，形状与本章"结果编码为工具消息"的职责重叠。本章复用并改写了它：新增 `is_error: bool` 参数，由 `bridge::result_to_message` 直接调用，不再制造第二套等价逻辑。改动前确认该函数在仓库内除自身定义外没有任何调用点，因此是安全的签名调整。

本章不引入 Agent Loop、Context 预算、Session、Policy、Sandbox、Validation 或 Observability；候选动作能否执行仍由 Runtime 边界决定，不由模型自称完成。

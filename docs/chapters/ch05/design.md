# ch5 Python M3 设计边界：从一步闭环到 Agent Loop

本切片在 `examples/python/m2-tool-runtime` 已验证的 `ToolRegistry`/`Workspace`/`bridge`/`chat_once` 之上新增 `examples/python/m3-agent-loop/agent_loop.py`，把第 4 章固定两次调用的一步闭环换成一个有预算、有显式停止原因的循环，只注册 `read` 一个工具，不修改 M0/M1/M2 任何文件。

`run_agent_loop` 每轮只处理四种局面，收敛成 `StopReason` 的四个变体：零工具候选且有非空最终文本 → `COMPLETED`；达到 `max_steps`/`max_tool_calls` → `BUDGET_EXHAUSTED`，且判断发生在下一次模型调用之前；一轮出现多个工具候选 → `AMBIGUOUS_TOOL_REQUEST`，循环不自动挑选、不执行任何一个；`complete()` 抛出 `ProtocolError`（Provider 响应解码失败等）→ `UNRECOGNIZED_ACTION`，循环捕获它而不是让进程崩溃。这一版 `StopReason` 不包含 `PolicyDenied`（依赖 ch10 的 Policy 引擎）和 `Cancelled`（依赖 ch7/ch13 的 Session/取消控制），正文显式说明了原因，不是遗漏。

工具执行失败仍由 M2 的 `ToolRegistry.execute` 收敛成结构化 `ToolResult`，作为下一轮观察正常进入循环，不中断循环。测试用脚本化 Transport 覆盖两轮成功闭环、步数预算、工具调用预算、多候选安全停止、工具失败后继续、Provider 解码失败安全停止；另有一条防御分支（响应既非工具候选也非文本）在 M1 当前 `decode_response` 的不变量下经真实链路不可达，测试直接 mock `complete()` 的返回值覆盖这条分支，不冒充端到端场景。

本章不引入 Context 预算、Session、Policy、Sandbox 或 Validation；`Completed` 只表示循环收到了非空最终文本，不代表这段文本已经过验证，验证仍是 ch11 的问题。Rust 独立示例和累计工程（`tutorial/agent-harness/`）的对应增量本次未做，留给后续任务。

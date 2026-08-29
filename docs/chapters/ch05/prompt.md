# ch5 需求 Prompt：Python M3 Agent Loop 原型

- **章节：** ch05
- **Prompt 原文状态：** Approved（本文件是获批后的原文归档，实施完成后不根据最终代码反向改写；实施结果、审计结论见本目录 `design.md` 和会话记录，不并入本文件）
- **需求确认状态：** Confirmed
- **实施批准状态：** Approved
- **实施状态：** Completed，已通过独立执行后审计（AUDIT PASSED，无阻断性问题；一条非阻断流程提醒见下）
- **上一章起点坐标：** 工作树坐标，非 commit SHA。分支 `main`，与 `origin/main` 同步；工作树中 `README.md`、`book/src/ch4.md`、`book/src/implementations.md`、`book/src/labs/m2-tool-runtime.md`、`examples/python/m2-tool-runtime/test_runtime.py`、`examples/python/m2-tool-runtime/tools/write.py` 有与本任务无关的未提交改动；`docs/chapters/`、`docs/decisions/`、`docs/prompts/`、`docs/workflow-runs/`、`CLAUDE.md`、`docs/report.md` 整体未跟踪。均已在实施中原样保留。
- **适用工作流：** `docs/prompts/workflow/v2/`（v1 七角色 + Artifact Recorder；`v2` 是当前接受版本，见 `docs/decisions/reader-ai-coding-workflow-v2.md`）
- **范围决定：** 本次任务只做 Python 原型 + ch5 正文改写；Rust 独立示例（`examples/rust/m3-agent-loop`）和累计工程增量（`tutorial/agent-harness/`）留给后续任务，不在本次范围内。

<!-- BEGIN APPROVED PROMPT -->
任务名称：
实现 M3 Agent Loop 的 Python 教学原型，并把 ch5 正文从设计骨架改写为已验证状态

任务性质：
这是《从 0 搭建 AI Agent》M3 里程碑的第一个真实代码增量，紧接在已验证的 M0（examples/python/m0-model-call）、
M1（examples/python/m1-unified-protocol）、M2（examples/python/m2-tool-runtime）之后。当前 M3 在仓库里
完全不存在——没有 Python、独立 Rust 或累计 Rust 工程的任何实现，book/src/ch5.md 目前是设计骨架，且其中
5.2 节直接贴了一段真实 Rust 代码，与全书"正文 Python-first"的约定冲突。本任务只做 Python 原型和 ch5 正文，
不做 Rust 独立示例、不做累计 Rust 工程增量，这两项是有意留到后续任务的技术债，不得在本任务里顺带实现。

用户已确认的业务逻辑：
1. AgentLoop 每轮：构造请求（复用 registry 的工具声明）→ 调用模型 → 检查响应里的工具候选数量：
   - 恰好 0 个且最终文本非空 → 停止，StopReason.COMPLETED；
   - 恰好 1 个 → 执行该工具（复用 M2 的 ToolRegistry.execute，不新写执行逻辑），把观察写回历史，进入下一轮；
   - 大于 1 个 → 不自动挑选、不执行任何一个，立即停止，StopReason.AMBIGUOUS_TOOL_REQUEST（用户明确要求：多工具
     候选需要人工复核，循环没有权限替读者做这个选择）；
   - 0 个且最终文本为空 → 立即停止，StopReason.UNRECOGNIZED_ACTION（既不是工具请求也不是有效结束，循环不猜测
     模型意图）。
2. 预算：至少支持 max_steps（模型调用总轮数上限）和 max_tool_calls（工具执行次数上限）两个独立边界；达到上限
   后必须在触发下一次模型调用之前停止，返回 StopReason.BUDGET_EXHAUSTED，不得多打一次模型调用。
3. 工具执行失败（未知工具、参数错误、执行异常）必须像 M2 的 ToolRegistry.execute 一样收敛成结构化 ToolResult，
   作为下一轮的观察正常进入循环，不能被异常直接中断整个 AgentLoop。
4. StopReason 这一版只包含 COMPLETED、BUDGET_EXHAUSTED、AMBIGUOUS_TOOL_REQUEST、UNRECOGNIZED_ACTION 四个变体，
   明确不包含 PolicyDenied（依赖 ch10 才有的 Policy 引擎）和 Cancelled（依赖 ch7/ch13 才有的 Session/取消
   控制）——ch5 正文必须用一句话说明这两个变体为什么现在不需要、以后哪一章会引入，不能让读者以为是遗漏。
5. 只注册 read 一个工具，复用 M2 已有的 ToolRegistry/Workspace/ReadTool，不新增工具、不改动 M2 的任何文件。

实施中确认的补充业务逻辑（发现 M1 decode_response 的不变量后新增，用户已确认）：
由于 M1 的 decode_response 保证任何成功解码的 ModelResponse 必然满足"有工具候选或有非空文本"之一，字面意义的
"零工具候选且文本为空"在真实 complete() 链路下不可达。UNRECOGNIZED_ACTION 同时用于捕获 complete() 抛出的
ProtocolError（Provider 响应解码失败、API 错误、传输错误），循环必须捕获它并安全停止，不能让进程崩溃退出。
原有的"响应内容既非工具候选也非文本"分支作为防御性分支保留（应对未来可能不遵守同一不变量的其他 decoder），
但需要如实说明它在当前链路下不可达，测试用直接构造/mock ModelResponse 的方式覆盖，不冒充端到端场景。

起点与目标代码坐标：
- 起点：examples/python/m0-model-call、m1-unified-protocol、m2-tool-runtime 均已实现并验证（离线测试全部通过）；
  examples/python 目录下没有 m3-agent-loop；book/src/ch5.md 是当前设计骨架（含待替换的 Rust 代码块）；
  book/src/implementations.md 没有 M3 相关条目。
- 目标：新增 examples/python/m3-agent-loop/（agent_loop.py + test_agent_loop.py），改写 book/src/ch5.md，
  新增 book/src/assets/ch05/agent-loop-state.mmd，并在 book/src/implementations.md 追加一行 M3 Python 条目。

允许新增和修改的文件：
- 新增：examples/python/m3-agent-loop/agent_loop.py
- 新增：examples/python/m3-agent-loop/test_agent_loop.py
- 新增：book/src/assets/ch05/agent-loop-state.mmd
- 修改：book/src/ch5.md（整章改写允许，但保留章节问题驱动结构和已有正确内容，不整篇推倒重写文风）
- 修改：book/src/implementations.md（只在 M2 小节之后追加一个新的"M3：Agent Loop"小节，不改动 M0/M1/M2/P0
  现有内容一个字）

默认禁止范围：
- examples/python/m0-model-call/、m1-unified-protocol/、m2-tool-runtime/ 下任何文件（只允许通过 sys.path
  插入相邻目录、`from xxx import yyy` 的方式只读复用，禁止修改）
- tutorial/agent-harness/、examples/rust/ 下任何文件
- README.md、book/src/ch4.md、book/src/labs/m2-tool-runtime.md
- docs/、CLAUDE.md、AGENTS.md、OUTLINE.md、Cargo.toml、Cargo.lock、crates/
- 不得执行 git add、git commit、push、tag、切换分支或修改 remote

验证命令：
```
python3.11 -m unittest discover -s examples/python/m3-agent-loop -p 'test_*.py'
```

十一项完成报告：
实现或修改摘要；新增文件；修改文件；公共 API 变化；新依赖及用途；执行的验证命令；验证结果；未运行的检查及
原因；已知限制；遗留问题；是否触及后续 milestone（ch6 及以后）。

停止条件：
- 发现 git 状态或 implementations.md/ch5.md 当前内容与本 Prompt 描述的起点不一致；
- 需要修改本 Prompt 禁止范围内的文件才能完成任务；
- 需要真实网络或真实 API Key 才能通过测试；
- 发现业务逻辑歧义（比如上方四条业务逻辑之外的新场景）。
停止时不得自行决定，如实报告并等待新的确认。

禁止提交、push 和未经批准的外部操作：
不得执行 git add、git commit、push、tag、发布、切换分支或修改 remote；不得运行 mdbook build 之外声称"文档
已构建"；不得伪造测试通过或运行结果。
<!-- END APPROVED PROMPT -->

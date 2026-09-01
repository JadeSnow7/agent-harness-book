# 全书结构与 Chapter Contracts（v0.2 editorial contract）

本轮精修以当前 `book/src`、M0/M1/M2 示例和 P0 确定性参考切片为事实基础。章节顺序负责教学递进，M0–M10 负责实现增量，P0 负责验证跨模块组合；三者不是同一条完成状态线。

## 读者与阅读方式

目标读者是已经了解 Agent 和基本计算机概念、但几乎没有独立完成大型项目经验的学生，以及希望跨界学习 Agent 的工作者。正文不承担完整的编程入门，但会提供 Python、终端、Git、JSON 和 HTTP 的预备路线与自测。

每章固定采用：

```text
问题与直觉 → Python 风格伪代码 → 可测试的 Python 片段
→ 失败路径与验证 → 实现索引 → 下一问题
```

Python 是正文的教学语言；已验证的 Python、Rust 实现通过实现索引引用。TypeScript 只有在实现、离线测试和类型检查都存在后才加入索引，本轮不创建空链接。

## 实现状态标签

- **已实现并验证**：本轮命令实际运行通过，且源码和测试可追溯。
- **P0 参考实现**：确定性、内存内、无外部副作用的组合切片，不等于生产级 subsystem。
- **设计骨架/尚未实现**：用于冻结问题、接口方向和验证场景，不得写成已有能力。

本轮状态快照：Python M4–M10 已在 tutorial/python/agent_harness 建立累计实现，并在各章 examples/python/m4–m10 中完成离线测试；独立 Rust M3 已实现并验证，Rust M4–M10、累计 Rust M3–M10、真实 Provider、durable recovery、OS sandbox 和生产扩展仍未实现。P0 仍是独立 Rust 参考组合。

## Part I — 从模型开始

- ch0 绪论：从模型到 Agent 系统
- ch1 什么是 Harness
- ch2 第一次模型调用
- ch3 从 Provider JSON 到统一协议
- ch4 从一次工具候选到一步闭环

## Part II — 让系统循环行动

- ch5 第一个 Agent Loop（M3）
- ch6 Context Engineering：模型本轮应该看到什么（M4）
- ch7 Session、Task、Run、Step：把执行变成可恢复对象（M5）
- ch8 Retry、Resume 与 Rollback：失败之后怎么办（M5）

## Part III — 让行动受到约束

- ch9 Side Effect、ChangeSet 与 Mutation（M6）
- ch10 Policy 与 Sandbox：允许什么真正执行（M6）
- ch11 Validation：模型说完成不等于完成（M7）
- ch12 Evidence 与 Observability：让结果可审计（M7/M8）

## Part IV — 让循环可靠地收敛

- ch13 Stop Conditions、Progress 与 Loop Engineering（M9）
- ch14 Planning 与 Sub-Agent：受控委派（M9）

## Part V — 扩展 Harness

- ch15 MCP、Skills、Hooks 与 Plugin（M10 前的扩展边界）
- ch16 完整 Harness：从通用运行时到 Forge Studio 案例（M10）

## Chapter Contracts

| 章节 | Problem | Reader Starts With | New Abstraction | Code Delta | Validation | Next Problem |
|---|---|---|---|---|---|---|
| ch0 | 新术语让 Agent 工程显得无从下手 | 基本 Agent 直觉 | Model/Harness/Environment 地图 | 全书路线与读者契约 | 能说清阅读顺序和边界 | Harness 到底负责什么 |
| ch1 | 模型能力被误当成系统能力 | ch0 的三层边界 | Harness 职责表 | 责任边界图与失败分类 | 能区分模型、策略、工具和环境失败 | 如何完成第一次模型调用 |
| ch2 | 单次调用不能访问环境 | Harness 职责地图 | Config、Transport、Response parsing | 一次安全的非流式请求 | Fake Transport 与错误路径 | Provider 字段如何隔离 |
| ch3 | Provider JSON 扩散到上层 | M0 请求/响应 | Message、ContentBlock、ToolDefinition | Provider adapter 与统一错误 | 文本、拒答、function call、tool result | 候选动作如何执行 |
| ch4 | 工具候选不是执行结果 | `ToolUseBlock` 与 `hello.txt` | ToolCall、Registry、一步闭环 | `read`、Workspace 边界、postcondition | call_id、路径、失败和二次调用 | 一步闭环如何继续 |
| ch5 | 一步闭环不能自动继续 | M2 `one_step`、`ToolUseBlock` | `AgentLoop`、`StopReason` | while 循环加预算与终止 | scripted model 两轮、预算/未知动作 | 上下文会膨胀 |
| ch6 | 每轮输入不受控且会超窗 | 有消息和工具结果 | `ContextBuilder`、`ContextBudget` | 排序、裁剪、来源 | 顺序、上限、必需项测试 | 内存状态无法恢复 |
| ch7 | 长任务缺少身份和边界 | 有界 loop 和事件草稿 | `Session/Task/Run/Step` | 事件 envelope、快照边界 | 重放不调用模型/工具 | 中断后如何继续 |
| ch8 | retry/resume/rollback 被混为一谈 | 事件日志与副作用概念 | `RecoveryDecision`、幂等键 | 显式恢复决策 | failpoint、重复调用计数 | 修改应先审查 |
| ch9 | 直接写入难 review/rollback | Tool Runtime 与 policy | `ChangeSet`、`Mutation` | 读写分离、变更提案 | diff、拒绝、应用一次 | 谁能批准执行 |
| ch10 | 工具能力不等于权限 | ChangeSet 与 P0 allowlist | `PolicyDecision`、sandbox boundary | allow/deny/ask 外置 | deny 无执行、路径边界 | 结果是否真实完成 |
| ch11 | final answer 只是声明 | tool result、policy | `Validator`、`ValidationReport` | 验证阶段与失败状态 | 测试/结构断言 | 如何证明验证发生 |
| ch12 | 验证结果难追溯 | validation report 与事件 | `Evidence`、`Trace` | 事件投影和证据引用 | supporting events/replay | 循环何时停止 |
| ch13 | while true 不知道何时收敛 | state、validation、evidence | `Progress`、`StopPolicy` | 停滞/预算/升级 | 重复动作和进展 fixture | 复杂任务如何拆分 |
| ch14 | 委派容易变成无边界聊天 | loop 与 capability | `SubRun`、delegation contract | 隔离上下文、预算、聚合 | 子运行失败/超预算 | 外部能力如何接入 |
| ch15 | 扩展机制语义混乱且有风险 | Tool/Context/Hook 边界 | capability manifest 与 lifecycle extension | manifest 与生命周期插槽 | schema、能力差异、脱敏 | 如何组合完整 harness |
| ch16 | 模块组合容易被误读为完成 | 全部前置抽象 | Harness profile 与领域 adapter | 组合而非新增核心依赖 | P0 对照、文档构建、逐章测试 | 后续逐章实现与精修 |

本表是教学契约，不声称所有 Code Delta 已经实现。当前可引用的真实类型包括 `DeterministicRunner`、`ScriptedMockModel`、`SimpleContextBuilder`、`AllowListPolicy`、`ToolRegistry`、`RequiredOutputValidator`、`InMemoryEventStore`、`Evidence` 和 `RunOutcome`。

## 章末工程分析层

Chapter Contract 约束本章新增什么；章末工程分析层解释这次增量在系统里留下了什么。两者不能互相替代。正文完成技术讲解和验证后，各章应根据自己的问题选择三到五个分析小节，不强行复制固定模板，但至少回答：

1. 本章之前系统处于什么状态，本章改变了哪条可观察路径；
2. 当前 checkout 中哪些能力已实现并验证，哪些只是 P0 参考实现，哪些仍是设计骨架；
3. 新抽象换来了什么具体收益，又增加了什么实现、运行或 AI 特有风险；
4. 当前能力属于 Experimental、Prototype、Usable、Stable 中哪一阶段，为什么还不能越级称为 Production-ready；
5. 哪些问题是有意留下的技术债，以及它如何迫使下一章出现。

状态判断以源码、测试和实际运行结果为准。P0 只是一条确定性、内存内、无外部副作用的组合切片；它可以为后续章节提供事件、状态和验证的不变量，但不会自动把对应的 M3–M10 教学实现标成完成。

全书的因果链固定为：

```text
提出问题 → 建立模型 → 最小实现 → 验证
        → 工程回顾与状态快照 → 收益、代价与风险
        → 有意留下的技术债 → 暴露下一问题
```

章节可以改变叙述重点：模型调用章节重在边界和失败路径，工具章节重在副作用，状态章节重在事实源和恢复，扩展章节重在能力声明与供应链风险。这样保持共同的方法论，同时避免把正文写成重复的设计卡片。

# 第 1 章：什么是 Harness

上一章把 Agent System 写成 `Model + Harness + Environment`。这个公式里，模型往往最受关注，Harness 却决定了模型能看见什么、能做什么，以及系统凭什么相信任务已经完成。下文先用常见产品对照 Harness 职责，再说明本书的最小选择。

## 1.1 Harness 的工程定义

Harness 是围绕模型构建的运行支撑层。它不负责替模型"思考"，而是把一次不确定的模型输出接入一个有状态、有权限、有验证的执行过程。

```text
                        ┌──────────────┐
                        │     User     │
                        └──────┬───────┘
                               │ Goal
                               ▼
┌──────────────────────────────────────────────────────┐
│                    Agent Harness                     │
│                                                      │
│  Context Builder   Tool Runtime   State / Memory     │
│  Permission        Policy         Validation         │
│  Observability     Recovery       Loop Controller    │
│                                                      │
└───────────────┬──────────────────────────┬───────────┘
                │ Model Input              │ Actions
                ▼                          ▼
        ┌──────────────┐           ┌──────────────┐
        │    Model     │           │ Environment  │
        └──────────────┘           └──────────────┘
```

提示词模板可以是 Harness 的一部分，但 Harness 远不只是一组提示词。它还必须处理程序世界里模型无法自行保证的事情：数据结构、I/O、权限、错误、预算、证据和终止。

## 1.2 Model、Harness 与 Environment 的边界

- **Model** 接收输入，返回文本或结构化动作候选。它不直接拥有宿主机权限。
- **Harness** 组织上下文、解释模型动作、实施策略、记录状态并驱动有界循环。
- **Environment** 是动作真正产生影响的地方，例如文件系统、终端、浏览器、数据库或远程 API。

一次工具调用因此不是"模型执行了命令"，而是：模型提出调用，Harness 检查调用，Environment 执行调用，Harness 再把结果作为观察交还给模型。

```text
ModelAction
    → PolicyDecision
    → ToolExecution
    → Observation
    → StateUpdate
```

这种拆分让失败可以被定位。模型选择错误、策略拒绝、工具运行失败和验证不通过是四类不同问题，不应该被压成一句"Agent 出错了"。

## 1.3 Harness 的核心职责


| 职责                  | 要回答的问题      | 最小输出             |
| ------------------- | ----------- | ---------------- |
| Context Builder     | 本轮模型应该看到什么  | 有序、受限的模型输入       |
| Tool Runtime        | 参数是否合法，怎样执行 | 结构化工具结果          |
| State / Session     | 任务进行到了哪里    | 可恢复或可重放的状态记录     |
| Permission / Policy | 这个动作是否允许执行  | Allow、Deny 或 Ask |
| Validation          | 环境是否真的满足目标  | 检查结果与失败原因        |
| Observability       | 系统为何做出这一步   | 事件、Trace 和证据     |
| Recovery            | 失败后怎样继续     | 重试、恢复、降级或终止决策    |
| Loop Controller     | 是否继续下一轮     | 明确的预算与停止原因       |


这些职责不意味着第一版就要实现八个复杂框架。先把输入、输出和所有者写清楚，通常比建立庞大的抽象层更重要。

## 1.4 为什么提示词不能代替系统边界

系统提示词可以告诉模型"不要删除文件"，却不能从操作系统层面阻止删除。它可以要求模型"先运行测试"，却不能证明测试进程真的启动、退出码为零，或测试覆盖了目标行为。

因此，安全与可靠性约束必须在模型之外执行：

- 路径规则和沙箱限制进程实际能够访问的资源；
- Policy 在工具执行前做出允许、拒绝或请求审批的决定；
- 超时和预算防止循环无限运行；
- Validator 根据环境状态判断任务是否完成；
- Event Log 保存可审计的行动与结果。

提示词仍然重要，但它适合表达目标、偏好和工作方法，不适合充当不可绕过的安全边界。

## 1.5 结构化 Prompt 是输入契约，不是安全边界

ch0 的 Prompt 生成器把工程事实整理成可审阅的输入契约。它的价值不在于让模型"更听话"，而在于让下一次实施有明确的起点、范围和证据要求。可以这样把 Prompt 的部分映射回 Harness：

| Prompt 部分 | Harness 中对应的责任 |
| --- | --- |
| 角色、背景、输入字段 | Context Builder：准备有序、受限的上下文 |
| 任务、工作流、阶段顺序 | Loop Controller：编排当前阶段与下一步 |
| 文件白名单、禁止事项 | Policy 的意图输入；实际 Allow、Deny 或 Ask 由独立的 Policy、工具权限或沙箱强制执行 |
| YAML 输出契约与状态 | Protocol / State：提供可解析的任务状态表示，持久化与恢复仍由系统负责 |
| 测试、验证和证据要求 | Validation / Observability：检查环境并留下依据 |
| 停止条件、人工批准 | Human Gate：在不可安全继续时暂停 |

这份 Prompt 仍有四个明确限制：模型可能不遵守文字约束；只读审计可能遗漏信息；路径范围必须由工具权限、沙箱和 Policy 实际落实；"测试通过"必须由真实命令、退出码和环境证据支持。换句话说，Prompt 描述责任，Harness 执行责任，Environment 才提供最终事实。

读者可以参考[附录 C 的可复制模板](appendix-ai-coding-prompt.md)，先生成草案，再交给后续角色和人工门禁审阅。项目变化后保留旧 Prompt，并以新证据生成 revision；这样 Prompt 是可追溯的工程输入，而不是一段被事后润色的模型对话。

## 1.6 常见 Harness 设计分析

不同产品对工具、权限、会话和扩展的组合不同，产品能力也会持续变化。阅读对比时只关注维度，不把它当作排名或永久事实。同一模型接入不同 Harness，可见边界与失败模式也会显著不同。

先用五个维度建立观察框架：


| 维度  | 需要观察的问题                         |
| --- | ------------------------------- |
| 上下文 | 是否读取仓库指令、当前文件和历史状态              |
| 执行  | 工具是否有参数约束、审批和沙箱边界               |
| 状态  | 是否能恢复会话、重放事件和查看过程               |
| 扩展  | MCP、Skills、Hooks、Plugins 位于哪个边界 |
| 反馈  | 是否有测试、诊断、验证和人工升级                |


同一公式 `Agent = Model + Harness`，不同 Harness 会塑造不同的智能体行为：编码向产品默认假设 Environment 是仓库与终端，优化读写文件、跑测试、看 diff；常驻型产品则假设 Environment 是消息通道、日程与长期记忆，优化跨会话 recall、定时任务与从 Telegram 发来的打断。模型权重相同，行为轮廓也可以完全不同。

下文对照八款产品：六款**编码向** Harness（Codex、Claude Code、OpenCode、Pi、Cursor、DeepSeek Harness）与两款**常驻、多通道** Harness（OpenClaw、Hermes）。描述基于截至 2026 年中的公开文档与可观察行为；细节以各产品当前文档为准，DeepSeek Harness 的核验日期见文末脚注。

八者差异在于 Harness 把哪些职责做成一等模块：


| 产品          | 入口形态                 | 任务域     | Harness 重心（一句话）              |
| ----------- | -------------------- | ------- | ---------------------------- |
| Codex       | CLI / 云端 agent       | 软件工程    | 强类型运行时 + OS 沙箱与审批分离          |
| Claude Code | 终端 agent             | 软件工程    | 生命周期 hooks + 默认只读与权限升级      |
| OpenCode    | TUI / Desktop / IDE 扩展 | 软件工程    | 开源多模型 + LSP 反馈 + build/plan 模式 |
| Pi          | 终端 TUI / SDK / RPC    | 软件工程    | 极小核心 + 扩展组合 + JSONL 树形会话    |
| Cursor      | IDE 集成               | 软件工程    | 编辑器上下文 + Rules/Skills + 工具审批  |
| DeepSeek Harness | CLI（NPX）/ Web UI | 软件工程/通用智能体任务 | Cordis 插件内核："一切皆插件" + host-level sandbox 默认拒绝 |
| OpenClaw    | 多通道聊天 / Gateway      | 常驻运营    | 通道 + 记忆 + 调度 + 可插拔 agent runtime |
| Hermes      | 消息 Gateway / TUI / daemon | 常驻运营    | 跨会话记忆 + 技能学习环 + cron 投递      |


每款产品的详细机制、时序图和取舍分析见[附录 D：Harness 产品案例横向对比](appendix-harness-comparison.md)；这里直接给出跨产品共同出现的几种设计张力。

### 横向归纳：Harness 设计张力

八款产品并不指向同一最优解，而是占住几种不同的张力：

**编码 vs 常驻运营。** Codex、Claude Code、OpenCode、Pi、Cursor、DeepSeek Harness 默认 Environment 是仓库与开发工具链；OpenClaw 与 Hermes 默认 Environment 是消息通道、日程与跨会话用户状态。同一模型在前者里像「工程师」，在后者里像「值班操作员」——差别主要在 Harness 装配的上下文与反馈，而非参数规模。

**集成深度 vs 可移植性。** Cursor 把 Harness 绑在 IDE 内，换来打开文件、诊断和 diff 的原生上下文；Pi、OpenCode 和 DeepSeek Harness 更偏终端、Web UI 或 SDK，便于嵌入脚本、CI 或自建界面；OpenClaw 与 Hermes 则把可移植性放在 Gateway 与自托管 daemon 上。没有绝对高下，只有任务是否依赖编辑器态或是否要求 7×24 在线。

**默认安全 vs 组合自由。** Claude Code 和 Codex 内置较明确的 permission 与 sandbox 默认值；Pi 把能力交给 Extensions 组合，灵活但边界由使用者承担。OpenCode 用 plan/build 模式在二者之间切分只读与写入阶段。DeepSeek Harness 用 host-level sandbox（默认拒绝网络、IPC 和文件监听）加插件化组合，试图两者兼顾——但插件本身的可信来源仍是一个新的开放问题。Hermes 提供多种 execution backend，隔离强度随 backend 变化，不能默认等于 OS 级沙箱。

**内核一体化 vs 一切皆插件。** Claude Code、Codex、Cursor 把 Context Builder、Policy、Tool Runtime 等模块编译进一体化运行时，扩展点相对有限；DeepSeek Harness 反过来，把模型适配、工具、会话、沙箱和调度全部实现成 Cordis 内核之上的插件，连核心能力都通过同一套注册机制加载。可插拔程度换来组合自由，也把"插件是否可信"从边缘问题变成了核心问题——这正是本书 ch15 要处理的能力声明和生命周期扩展边界。

**厂商绑定 vs 模型中立。** Codex、Claude Code、Cursor 各自绑定模型与账号生态；OpenCode、Pi、OpenClaw 与 Hermes 更强调多 provider 或可插拔 runtime；DeepSeek Harness 目前以 DeepSeek 自家模型为主，但其插件化内核在架构上允许替换 Model 适配器。Harness 设计因此也包含「模型路由放在哪一层、runtime 切换是否 fail closed」的问题。

读产品时应问「哪一层是 Harness 负责的」，而不是「哪个模型更强」。八者都证明：Agent 行为轮廓，大量来自运行时如何把上下文、工具、权限、记忆与反馈接进循环——**换 Harness，比换模型更常改变智能体「像什么」**。

上述产品可以很重、很开放，或 IDE 原生；本书的目标不同——先做一个**可测试、可解释、默认确定性**的最小闭环，而不是复刻任何一家。下一节说明这一选择的具体原则。

## 1.7 本书选择的最小 Harness

本书选择以下原则：

1. **小而完整**：先形成一个能结束、能失败、能被测试的闭环。
2. **边界显式**：模型、策略、工具、验证和状态各自返回结构化结果。
3. **默认确定性**：默认测试不访问真实网络，不依赖 API Key。
4. **安全在模型之外实施**：不把提示词当作权限控制。
5. **正文追溯到源码**：核心 Python 片段来自可测试示例，Rust 实现通过索引引用。
6. **保持通用**：通用 Harness 不依赖 Forge Studio 或其他产品领域类型。

最小闭环可以表示为：

```text
Goal
  → Build Context
  → Model Action
  → Policy Check
  → Tool Execution
  → Observation
  → Validation
  → Continue or Stop
```

"最小"意味着暂不加入用不到的抽象；"完整"意味着不能省略失败路径和停止条件。本书也不复刻任何现有产品，不把 Forge Studio 的领域模型放进通用核心。

## 1.8 本章小结

大语言模型提供生成和推理能力，Harness 决定这种能力如何进入真实环境。下一章先退回到最小起点：不调用工具、不驱动循环，只完成一次真实但有安全边界的模型请求，并观察它为什么还不是 Agent。

## 1.9 边界画出来以后，系统并没有自动变强

本章开始时，"模型会回答"很容易被误读成"系统能完成任务"；现在至少可以把模型、Harness 和 Environment 的责任分开。收益是故障可以被定位：解析失败属于协议边界，工具失败属于 Runtime，越权属于 Policy，结果不满足目标属于 Validation。后面每加入一个模块，读者都能问它究竟接住了哪一种失败。

当前状态仍应保持克制：M0–M2 的实现和测试已经存在，但本章介绍的完整职责表不是一个已经交付的生产 Harness。P0 组合拥有确定性 Runner、Policy、Tool、Validation 和事件，但它是内存内参考实现。整体成熟度属于 **Experimental**：边界语言可用于教学和设计评审，跨模块的生产运行条件尚未具备。

边界也带来了代价。抽象层越多，数据转换和责任错位的机会越多；如果把"模型提出了动作"误当成"Policy 已批准"，系统就会在文字上有安全边界、在执行上没有安全边界。AI 的非确定性还会让同一职责边界收到不同形状的候选，因此每个边界都必须有结构化失败和离线测试，不能只靠示例中的成功路径。

本章有意留下的技术债，是暂时不实现完整权限、状态和验证框架。现在先把职责说清楚，下一章再用一次最小模型调用证明：即使 Transport 和解析都正确，模型仍然看不到环境，也不能凭空完成 `hello.txt` 的读取。

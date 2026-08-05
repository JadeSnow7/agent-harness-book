

# 绪论

你可能听过这样一种说法：在 AI 时代，只要你学东西够慢，那就不用学了，自 2022 年以来，业界的研究重点从 prompt engineering（提示词工程）到 context engineering（上下文工程），再到 harness engineering（约束工程）以及最近兴起的 loop engineering（循环工程），关于 Agent 的技术不断演进，可能你刚弄懂一项技术，它已经是过去式了。

很多人在这股 AI 浪潮中都深感迷茫和焦虑，担心自己随时会被 AI 取代。然而，有挑战必有机遇，趁 AI 方兴未艾，积极学习新技术，拥抱变革，才是破局之道。

这也是笔者写这个系列的初衷：记录自己的学习过程，以“干中学”的方式，从 0 开始搭建一个自己的 Agent 框架，并在这个过程中学习 Agent 的前沿技术。同时，也希望自己的学习经验能帮助到同样在学习 Agent 的你。

## 一次 Agent Run 里发生了什么

一次 Agent Run 的骨架大致长这样：

```text
构造上下文 → 调用模型 → 判定工具权限 → 执行工具
        → 验证结果 → 记录事件与证据 → 决定是否继续
```

在这个流程里，模型只是其中的一个环节。它拿到什么、能做什么、做完之后发生了什么，都由模型外部的一圈代码决定。这一圈代码，就是 harness。理解了这一点，引言的核心论点就清楚了——**可靠性是在模型外围被设计出来的，而不是在模型内部被祈祷出来的。**

## 关注点逐层外移：四段演进

如果把 Agent 工程化的重心画成一条时间线，会看到一条清晰的线索：

```text
Prompt engineering → Context engineering → Harness engineering → Loop engineering
```

这条线索最容易被误读。很多人把它当成一条升级路径，仿佛新阶段出现、旧阶段就过时了。恰恰相反：**这四个阶段不是四代技术互相替代，而是 Agent 工程关注点的逐层外移。** 新的阶段并未消灭旧的阶段，而是把旧的阶段包进更大的系统边界。具体来说，每一层优化的对象分别是：

- **Prompt engineering：优化一次模型调用。** 面对"怎样用输入文本激发模型能力"这个问题，Few-shot、CoT、Self-Consistency、结构化提示这些技术先后成为主流。在那个阶段，措辞的效果立竿见影。
- **Context engineering：优化模型在当前步骤能看到什么。** 当应用开始承载真实任务，可用的信息量远超任何窗口，问题变成"有限窗口里应该放什么、删什么、何时检索"。RAG、长上下文、摘要压缩、分层记忆、上下文隔离随之而来。
- **Harness engineering：优化模型如何获得状态、工具、权限、反馈和验证。** 问题的焦点从"模型能想到什么"扩展到"模型能不能安全、可信地做到"。工具注册、状态管理、沙箱、审批、会话、验证、追踪，构成一个可靠的运行时。
- **Loop engineering：优化整个系统如何反复行动、纠错、收敛和停止。** Agent 一旦连续行动，可靠性就取决于循环本身——是否在推进、是否陷入停滞、何时重试、何时升级、何时必须停止。

每一层都源于上一层的边界，又完整地保留了上一层：context engineering 没有让 prompt 失效，harness engineering 没有让上下文管理失效，loop engineering 也没有让工具与验证失效。**演进是加宽，不是替换。**

## 时间线上的印证

报告给出了一条建议采用的时间划分，它恰好把上面的抽象落到了具体的年份：


| 阶段                  | 主要活跃期                       | 核心问题                     | 标志性突破                                    | 代表项目或实践                                          |
| ------------------- | --------------------------- | ------------------------ | ---------------------------------------- | ------------------------------------------------ |
| Prompt engineering  | 2020—2024 为主流焦点，持续至今        | 怎样用输入文本激发模型能力            | Few-shot、CoT、Self-Consistency、结构化提示      | GPT-3 prompting、CoT、DSPy、PromptSource            |
| Context engineering | 2023—2025 快速形成，持续至今         | 有限窗口内应该放什么、删什么、何时检索      | RAG、长上下文、摘要压缩、分层记忆、上下文隔离                 | LangChain/LangGraph、MemGPT、LlamaIndex            |
| Harness engineering | 2024 年萌芽，2025—2026 成为明确工程范式 | 怎样把模型包进一个可靠运行时           | Tool registry、状态管理、沙箱、审批、会话、验证、追踪        | Codex、Claude Code、Pi、OpenHands                   |
| Loop engineering    | 2026 年形成行业术语，学术根源始于 2022    | 怎样设计能够持续行动、反馈、纠错并稳定停止的闭环 | ReAct、Reflexion、Self-Refine、测试驱动循环、多代理审查 | Codex/Claude Code 自动循环、Symphony、各类 reviewer loop |


表格里藏着几个值得细看的转折点。GPT-3 的 few-shot prompting 把"通过输入而非微调改变行为"推向主流，CoT 与 Self-Consistency 随后扩展了单次推理的能力。但 RAG、MemGPT 和 Lost in the Middle 等工作很快证明了一件事：**扩大上下文并不等于正确利用上下文。** "只要信息在窗口里，模型就会好好用"——这个直觉是错的，于是上下文的选择、压缩与记忆管理成为一门需要专门手艺的工程。2025 年后，context engineering 被明确描述为对有限上下文资源的系统化管理；2026 年，harness engineering 进一步把关注点扩展到工具、运行环境、验证、权限、状态和可观测性。无论是工程实践还是同期论文，都指向同一个结论：**Agent 成功与否，往往取决于整个"模型—harness—环境"系统，而不只是模型本身。**

## 技术进化树

如果把四层分别展开，报告整理出一棵技术进化树。它像一棵树的分层高度：下层是上层的地基，上层把下层包进更大的边界。

```text
Foundation Model
│
├── Prompt Engineering
│   ├── 指令与角色
│   ├── Few-shot
│   ├── Chain-of-Thought
│   └── Structured Output
│
├── Context Engineering
│   ├── 当前对话选择
│   ├── RAG / 搜索
│   ├── 摘要与压缩
│   ├── 长短期记忆
│   └── 上下文隔离
│
├── Harness Engineering
│   ├── Model Adapter
│   ├── Tool Registry / Executor
│   ├── Session / State
│   ├── Sandbox / Permission
│   ├── Validation / Guardrail
│   └── Trace / Observability
│
└── Loop Engineering
    ├── Plan → Act → Observe
    ├── Verify → Diagnose → Revise
    ├── Progress / Convergence Detection
    ├── Reviewer / Sub-agent
    ├── Failure Recovery
    └── Stop / Escalate / Human Approval
```

值得注意的还有横向贯穿这四层的能力：**评测、安全、可观测性、成本控制和人类监督。** 它们不专属于任何一层，而是每一层都要回答的横切问题——这也是为什么我们几乎能在任意一层看到"评测集""审批"或"trace"的影子。读这本书时请记住这张图：后续每一章，都是在为这棵树上的某一根枝干写下具体的实现。

## 值得记住的论文与文章

报告的论文列表是理解这条演进的捷径，也解释了每个阶段从何而来：


| 资源                                                                    | 作者与年份           | 价值                  |
| --------------------------------------------------------------------- | --------------- | ------------------- |
| Language Models are Few-Shot Learners                                 | Brown 等，2020    | Prompt 范式起点之一       |
| Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks      | Lewis 等，2020    | 外部知识进入上下文的基础        |
| Chain-of-Thought Prompting Elicits Reasoning in Large Language Models | Wei 等，2022      | 多步推理提示              |
| ReAct: Synergizing Reasoning and Acting in Language Models            | Yao 等，2022/2023 | 推理与环境行动交替           |
| Lost in the Middle                                                    | Liu 等，2023      | 长上下文并不保证有效利用        |
| MemGPT                                                                | Packer 等，2023   | 分层、虚拟化上下文管理         |
| Reflexion                                                             | Shinn 等，2023    | 语言反馈与情节记忆           |
| Self-Refine                                                           | Madaan 等，2023   | 生成—反馈—改写循环          |
| Plan-and-Solve Prompting                                              | Wang 等，2023     | 先规划、后执行             |
| Effective Context Engineering for AI Agents                           | Anthropic，2025  | 工业界上下文工程总结          |
| Harness Engineering: Leveraging Codex in an Agent-first World         | Lopopolo，2026   | Agent 友好仓库和验证环境     |
| Harness Engineering for Coding Agent Users                            | Böckeler 等，2026 | 面向使用者的 harness 心智模型 |
| Agentic Harness Engineering                                           | Lin 等，2026      | 基于可观测证据自动演化 harness |
| What Is Loop Engineering?                                             | IBM，2026        | 当前行业定义              |


不必逐篇精读。Few-Shot Learners 与 CoT 定义了"通过输入激发能力"，RAG 与 Lost in the Middle 定义了"外部知识与上下文利用"，ReAct、Reflexion、Self-Refine 定义了"行动—反馈—改写"的循环雏形，而近两年的工业界文章——Anthropic 的上下文工程总结、多篇 harness engineering 文章、IBM 对 loop engineering 的行业定义——把焦点一路从模型内部带到模型外围。每篇背后，都对应着本书后面会亲手实现的一个机制。

## 一个需要谨慎对待的术语

四层演进里，前三个词——prompt engineering、context engineering、harness engineering——在业界已经有相对稳定的所指。唯独 **loop engineering** 需要特别说明：**它是 2026 年才迅速流行的行业术语，至今尚未形成稳定的学术定义。** 它当下的学术根基，实际上来自更早的研究——ReAct、Reflexion、Self-Refine、测试反馈循环，以及更古老的控制论式 Agent 研究。本书在讨论这个方向时会保持同样的谨慎：我们描述问题、给出工程手段，但不把这个仍在演进的方向包装成已经成熟的理论。

## 接下来

这条演进线，同时也是本书的路线图。从大模型与 Agent 的边界开始，书里会一步步把 context、harness、loop 各层亲手搭出来——直到你能重新诊断引言开头那个场景的真正病因：信息没进来、工具没做对、循环没收敛。到那时，你会和许多实践者一样，把"改 prompt"放到最后，而不是最先。

# 绪论

你可能听过这样一种说法：在 AI 时代，只要你学东西够慢，那就不用学了，自生成式 AI 进入大众视野以来，业界的关注点从 prompt engineering（提示词工程）到 context engineering（上下文工程），再到 harness engineering（约束工程）以及最近被讨论越来越多的 loop engineering（循环工程），关于 Agent 的技术不断演进，可能你刚弄懂一项技术，业界早就开始攻关新的技术了。

| 阶段                  | 主要活跃期                       | 核心问题                     | 标志性突破                                    | 代表项目或实践                                          |
| ------------------- | --------------------------- | ------------------------ | ---------------------------------------- | ------------------------------------------------ |
| Prompt engineering  | 2020—2024 为主流焦点，持续至今        | 怎样用输入文本激发模型能力            | Few-shot、CoT、Self-Consistency、结构化提示      | GPT-3 prompting、CoT、DSPy、PromptSource            |
| Context engineering | 2023—2025 快速形成，持续至今         | 有限窗口内应该放什么、删什么、何时检索      | RAG、长上下文、摘要压缩、分层记忆、上下文隔离                 | LangChain/LangGraph、MemGPT、LlamaIndex            |
| Harness engineering | 2024 年萌芽，2025—2026 成为明确工程范式 | 怎样把模型包进一个可靠运行时           | Tool registry、状态管理、沙箱、审批、会话、验证、追踪        | Codex、Claude Code、Pi、OpenHands                   |
| Loop engineering    | 2026 年形成行业术语，学术根源始于 2022    | 怎样设计能够持续行动、反馈、纠错并稳定停止的闭环 | ReAct、Reflexion、Self-Refine、测试驱动循环、多代理审查 | Codex/Claude Code 自动循环、Symphony、各类 reviewer loop |

很多人在这股 AI 浪潮中都深感迷茫和焦虑，担心自己随时会被 AI 取代。然而，有挑战必有机遇，趁 AI 方兴未艾，积极学习新技术，拥抱变革，才是破局之道。

这也是笔者写这个系列的初衷：记录自己的学习过程，以“干中学”的方式，从 0 开始搭建一个自己的 Agent 框架————更准确地说，一个最小但完整的 Agent Harness，并在这个过程中学习 Agent 的前沿技术。同时，也希望自己的学习经验能帮助到同样在学习 Agent 的你。

## Agent = LLM + Harness

相信不少人在求职面试时都被问过一个问题————什么是 Agent？大部分人的第一反应是先愣一下，然后回答代理，或者智能体，“什么是 Agent”看似简单，却很难得到统一答案。在业界给出权威统一的定义之前，我们暂且不管各种争论，你只需要记住一点：

**大模型不是 Agent**

当然，更加细致严谨的解释或许是：\(\text{LLM Agent}
=
\text{Model}
+
\text{Harness}
+
\text{Environment}
+
\text{Feedback Loop}\)，不过在本书讨论范围内，我们可以简单将 Agent 理解为一个大模型及支撑其工作的一套 harness，即：

\(\text{Agent}
=
\text{Model}
+
\text{Harness}\)

## 本书规划

本系列计划采取循序渐进的方式，搭建起一个可独立运行并可观测可验证的 Agent Harness 框架。

第一部分将简单介绍什么是 harness 和我们需要使用的技术。
第二部分将实现一个最小的 harness 运行时。
第三部分将通过提示词、上下文、循环验证等技术让 harness 变得可信。
第四部分将引入动态路由、子代理、长期记忆等高级功能。

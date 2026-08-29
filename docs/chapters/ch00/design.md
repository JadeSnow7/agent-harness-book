# ch0 设计方案：全书架构地图与 AI Coding 工作流

## 本章要解决的问题

读者在看到模型调用和工具代码之前，容易把“模型给出的文本”误认为“系统已经完成的动作”。本章先建立一张可复查的地图：Model 提出候选，Harness 组织上下文、策略、运行时和记录，Runtime 才在受控边界内接触 Environment；环境返回 Observation，Validation 把状态转换为 Evidence，人再依据目标、风险和预算决定继续或停止。

本章还要解释全书不是若干孤立示例，而是从同一个小任务逐步增加协议、工具闭环、循环、状态、策略、验证和扩展能力的工程路线。同时，本书本身采用同样的控制方式推进：Prompt 先被批准，增量再被实施，结果由外部检查和人工 Review 验收。

## 读者进入本章时具备的知识

读者可以有 Agent 和模型的直觉，但不假设其熟悉网络、操作系统、权限、事件记录或数据结构。正文只在第一次需要时给出足以支撑当前论证的短说明；HTTP、进程模型、序列化、事件溯源和具体实现边界留给后续章节。

## 本章交付的认知增量

读者离开本章时应能：

1. 区分 Model、Harness 和 Environment 的位置，而不把模型文本当成环境事实；
2. 说出一次反馈闭环中的 Input、Candidate Action、Observation、Event 和 Evidence；
3. 根据 Chapter Contract 理解 M0–M2、P0 和 M3–M10 的不同状态；
4. 解释为什么本章只建立地图和验收语言，不新增运行时代码；
5. 复述 AI Coding 的两个人工门禁，以及“已经完成”为什么不是证据。

## 两张图各自承担的认知任务

- `agent-harness-map.mmd` 是**系统架构图**。它回答“谁向谁提供什么、动作在哪里被允许、事实在哪里产生、状态和证据如何回流”，并用虚线分层标明当前仓库事实与未来路线，避免把目标架构读成已完成产品。
- `ai-coding-workflow.mmd` 是**协作流程图**。它回答“本书的一次增量如何从目标走到验收”，突出实施前批准 Prompt 和实施后审阅代码、证据及章节结果两个不可省略的人工门禁。AI 的完成声明不会直接通向任务完成。

## 本章不新增运行时代码的理由

ch0 的任务是固定共同语言、路线和验收边界。如果此时新增一段没有真实职责的 Python/Rust 代码，读者会把地图误读为能力，把占位实现误读为已经完成的 Runtime。本章保留仓库中已有的 M0–M2 和 P0 作为可追溯证据，但不重新实现它们；真实代码增量从 ch2 起按 Chapter Contract 发生。

## 与 ch1、ch2 的边界

ch0 只做总览，不详细裁定 Model 与 Harness 的职责边界，也不写 ch1 的边界伪代码。ch1 将把“谁负责什么、失败由谁判定”拆成可操作的边界；ch2 才开始真实的模型调用、Transport 和离线验证。后续章节再逐步引入统一协议、工具 Runtime、循环、状态、Policy、Validation 与 Evidence。

## 当前仓库事实状态

- `examples/python/m0-model-call/`、`m1-unified-protocol/`、`m2-tool-runtime/` 以及对应 Rust 示例是现有 M0–M2 证据链，不是本轮新实现。
- `crates/`、`fixtures/`、`docs/adr/`、`docs/specs/` 和 `docs/architecture/p0-component-boundaries.md` 保留了实现、合同和 P0 组合参考；P0 是确定性组合切片，不等同于 M3–M10 已逐章完成。
- M3–M10 的后续章节仍按仓库的 Chapter Contract 逐步实现；本章只提供路线说明和状态区分。

## 验收标准

- ch0 正文引用两份 Mermaid 源文件，且引用位置分别位于系统架构和 AI Coding 流程叙事中；
- 两图包含要求的节点、数据流、人工门禁和状态区分，不能暗示所有组件已生产可用；
- mdBook 0.5.4 使用固定版本的 `mdbook-mermaid` 0.17.0 构建到临时目录并生成 ch0 页面，内部链接和 include 无错误；
- 浏览器实际显示两张图，浅色与深色主题均可读，控制台没有 Mermaid 错误，页面不访问 Mermaid CDN；
- 变更文件严格位于本轮允许范围，`git diff --check` 通过；
- 正文明确写出没有新增 Agent Runtime 能力、ch1 边界伪代码尚未实现、ch2 及之后逐章真实实现尚未开始。

## Mermaid 渲染方案

本书使用 `mdbook-mermaid` 0.17.0 处理 Mermaid 代码块，并通过 `book.toml` 的 `additional-js` 加载仓库内的 `mermaid.min.js` 与 `mermaid-init.js`。`links` 预处理器先展开 `{{#include ...}}`，Mermaid 预处理器再把代码块转换为浏览器可渲染的节点。这样构建后的页面不依赖在线 CDN，也不需要为绘图引入 Node、Chromium 或 Agent Runtime 依赖。

`mdbook-mermaid` 是文档构建期工具；其安装器写入的初始化脚本保留 MPL-2.0 声明，随书保存的 Mermaid 11.6.0 JavaScript 保留 MIT 许可证声明。预处理器版本需要在本地说明、CI 和 Pages 工作流中保持一致。启用预处理器也会渲染旧章节已有的 Mermaid 块；本轮只检查这些图是否报错，不借机改写旧章节。

## 已知限制

图是认知地图，不是完整 API 规范、部署拓扑或生产安全保证。当前仓库中的 M0–M2 与 P0 证据不能证明真实网络、持久恢复、Sandbox、Recovery、Validation 或 Observability 已达到生产级；文档构建通过也只证明文档资产可构建，不证明运行时代码正确。后续章节的实现状态仍必须以源码、测试和实际验证为准。

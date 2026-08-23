# 逐章 AI Coding 工作流协议 v1

本目录保存《从 0 构建 Harness》的跨平台 AI Coding 工作流。它帮助读者从上一章工程坐标出发，经人工确认后完成本章增量，并留下可审计的实现、验证和 Git 证据。

决策依据见 [`reader-ai-coding-workflow-v1.md`](../../../decisions/reader-ai-coding-workflow-v1.md)。

## 资产索引

- [`controller.md`](controller.md)：总控 Prompt，负责能力检测、状态管理、派发和人工门禁；
- [`manual-handoff.md`](manual-handoff.md)：平台不支持多 Agent 时的手动会话协议；
- [`task-package.template.yaml`](task-package.template.yaml)：角色之间传递的任务包模板；
- [`agent-result.template.yaml`](agent-result.template.yaml)：角色返回结果的统一模板；
- [`roles/task-analysis.md`](roles/task-analysis.md)：任务与需求分析；
- [`roles/planning.md`](roles/planning.md)：计划和编码 Prompt；
- [`roles/implementation.md`](roles/implementation.md)：批准后的编码与验证；
- [`roles/post-implementation-audit.md`](roles/post-implementation-audit.md)：执行后独立审计；
- [`roles/summary.md`](roles/summary.md)：十一项完成报告；
- [`roles/commit.md`](roles/commit.md)：批准后的本地 Git 提交；
- [`roles/commit-audit.md`](roles/commit-audit.md)：提交内容与遗留工作区审核。

## 使用顺序

```text
能力检测
→ 任务与需求分析
→ 用户确认业务理解
→ 计划与完整编码 Prompt
→ 用户批准实施
→ 编码与验证
→ 执行后审计
→ 用户确认审计结论
→ 汇总报告
→ 用户批准提交
→ 本地提交
→ 提交审核
→ 用户最终验收
```

任何审计产生异议时，都不能直接进入修复。总控应回到需求分析或计划阶段，生成新的实施 Prompt，并重新等待用户批准。

## 自动与手动模式

总控在开始时声明：

```yaml
multi_agent_support: supported | unsupported | uncertain
execution_mode: automatic_multi_agent | manual_session_handoff
```

`supported` 可以自动调度隔离子 Agent。`unsupported` 或 `uncertain` 必须使用手动交接包，由读者在新会话中执行下一个角色。两种模式使用相同的任务包、权限和输出协议。

## 权限摘要

| 角色 | 工作区权限 | Git 权限 |
| --- | --- | --- |
| 总控 | 只读 | 只读 |
| 任务、需求分析 | 只读 | 只读 |
| 计划 | 只读 | 只读 |
| 编码 | 仅批准路径 | 只读，不提交 |
| 执行后审计 | 只读 | 只读 |
| 汇总 | 只读 | 只读 |
| 提交 | 不编辑业务文件 | 白名单暂存和本地提交 |
| 提交审核 | 只读 | 只读 |

## 模型选择

Sol 与 Luna 是能力画像，不是供应商依赖：

- Sol 类用于需求、计划、审计和综合判断；
- Luna 类用于边界明确的编码和机械 Git 操作。

总控每次派发都要说明实际模型、选择理由、任务复杂度和是否需要拆分。平台没有对应名称时，选择能力相近的模型即可。

## 与章节工程的关系

本目录只定义工作流，不包含任何章节已经实现的声明。章节独有的设计、Python 原型、架构图、读者 Rust Prompt、milestone 和执行报告应放在 `docs/chapters/chXX/`；累计 Rust 工程将在后续批准的 ch2 任务中独立建立。

本工作流使用的 Coding 子 Agent 不等于 ch14 将实现的 Harness `SubRun/Sub-Agent`。

## 版本与修订

- 主版本：v1；
- 破坏角色、状态、门禁或必填字段时建立 v2；
- 非破坏性修订记录在下表；
- 已批准的章节 Prompt 原文不得事后改写。

| Revision | Date | Change |
| --- | --- | --- |
| 001 | 2026-08-17 | 建立跨平台总控、七类角色、手动交接和结构化任务协议。 |

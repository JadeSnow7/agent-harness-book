# Reader AI Coding Workflow v2

This breaking v2 decision preserves the complete v1 protocol below and adds a
minimum-write Artifact Recorder. The durable sequence is `raw export -> record
-> verify -> state transition`; payload bytes are exact and metadata is a
canonical JSON-as-YAML manifest without host paths. No overwrite is allowed;
corrections create revisions. Missing, unsafe, sensitive, or unverifiable
inputs fail closed. Controller verification is deterministic integrity checking,
not semantic audit. There are no signatures, trusted external time source, or
complete secret detector.

- **Status:** Accepted
- **Decision date:** 2026-08-17
- **Scope:** 本书逐章教学工程的 AI Coding 工作流
- **Protocol assets:** `docs/prompts/workflow/v2/`（在 v1 协议资产基础上新增 Artifact Recorder；v1 资产保留只读，见 `docs/prompts/workflow/v1/`）

> **状态追记（2026-08-19）：** 下方"背景"与"后续事项"写于本决策通过之时，此后累计 Rust 教学工程 `tutorial/agent-harness/` 已经建立，覆盖 M0（ch2）与 M1（ch3）；ch1 也已补齐 Model/Harness 边界正文。因此"教学工程将在后续经批准的 ch2 任务中建立""为 ch1 建立任务包""在 ch2 创建独立的累计 Rust 教学工程"均已完成，不再是待办事项。本追记按附注方式说明现状，不改写原决策正文；具体源码与测试状态以 [实现索引](../../book/src/implementations.md) 和仓库当前测试为准。

## 背景

本书不仅解释 Harness，也要让读者能够把每章提供的 Prompt 交给 AI Coding 工具，从上一章工程坐标出发，生成本章的最小 Rust 增量。只保存一段编码指令不足以支撑这个目标：需求理解、计划、实现、审计、报告和 Git 坐标之间必须存在可复查的交接协议，任何改变工程状态的动作也必须保留人工门禁。

这套协议服务于独立的累计教学工程。当前 `crates/`、`examples/`、P0 和 M0–M2 仍是仓库已有证据链，不自动成为新教学工程的逐章完成状态。教学工程将在后续经单独批准的 ch2 任务中建立；本决策不创建它。

## 决策

采用 `docs/prompts/workflow/v2/` 中的跨平台工作流协议，在 v1 七种逻辑角色基础上新增 Artifact Recorder。一次章节任务由总控维护状态，并按顺序调度八种逻辑角色：

1. 任务、需求分析；
2. 计划；
3. 编码；
4. 执行后审计；
5. 汇总；
6. 提交；
7. 提交审核；
8. 归档记录（Artifact Recorder）。

前七种角色保留 v1 的权限边界和人工门禁；Artifact Recorder 是机械持久化角色，不总结、不审计、不修复、不修改 v1/章节/业务代码/Git/网络，只把其余角色的原始导出按 `raw export -> record -> verify -> state transition` 写成精确字节的 payload/manifest 对，细节见 [`docs/prompts/workflow/v2/roles/artifact-recorder.md`](../prompts/workflow/v2/roles/artifact-recorder.md)。

总控可以只读获取工作区状态、任务坐标和各角色报告，但不参与具体编码、测试、修复或 Git 提交。角色可以使用同一种底层模型，但上下文、权限和交接结果必须按协议隔离。

## 跨平台与多 Agent

核心 Prompt 不写死某个平台的子 Agent API、工具名或调用语法。启动时，总控必须把多 Agent 能力判断为：

- `supported`：能够创建隔离子 Agent 或子会话、传入受控任务包并收集结果；
- `unsupported`：平台明确不支持；
- `uncertain`：无法从当前能力证明支持。

只有 `supported` 可以进入自动调度。`unsupported` 和 `uncertain` 都使用手动会话交接：总控生成完整交接包，读者在新会话中执行目标角色，再把结构化结果带回总控。降级不授权总控亲自编码。

## 人工门禁

以下阶段必须等待用户明确确认：

1. 需求分析后确认业务理解；
2. 计划和完整编码 Prompt 生成后批准实施；
3. 执行后审计完成后确认审计结论；
4. 汇总报告完成后批准 Git 提交；
5. 提交审核完成后确认最终验收。

沉默、模糊回复、历史任务中的批准或子 Agent 的结论都不构成本轮批准。审计发现业务逻辑争议或实现缺陷时，任务回到需求分析或计划阶段；修复属于新的实施，必须重新批准。

## 报告与提交分离

汇总默认只生成完成报告，不改变 Git 状态。提交角色只有在用户明确“批准提交”后才能按显式文件白名单暂存并创建本地提交。默认禁止 `git add .`、push、tag、发布、切换分支以及修改 remote。

提交审核只读。发现错误时报告真实状态，不自动 amend、reset、revert 或清理工作区。

## 模型能力画像

工作流使用可跨平台映射的能力画像，而不是强制供应商或模型名称：

- **Sol 类：** 偏重复杂推理、边界判断、计划和独立审计；
- **Luna 类：** 偏重在明确范围内高效完成编码或机械操作。

默认分配如下：

| 角色 | 默认画像 |
| --- | --- |
| 任务、需求分析 | Sol 类 |
| 计划 | Sol 类 |
| 编码 | Luna 类 |
| 执行后审计 | Sol 类 |
| 汇总 | Sol 类 |
| 提交 | Luna 类 |
| 提交审核 | Sol 类 |
| 归档记录（Artifact Recorder） | Luna 类 |

平台没有同名模型时，总控选择能力相近的模型并说明依据。复杂编码任务可以拆分为多个编码子任务，但必须声明文件所有权、集成责任、重叠区域和失败回收方式。

## Python 原型与业务逻辑

Python 原型是正文教学资产，不要求全文复制进编码 Prompt。章节任务包只需引用原型位置及其提供的业务契约。

若某章不适合 Python 原型，Prompt 必须给出建议的输入、输出、主流程、状态变化、错误路径和有意限制，并将 `user_confirmation` 保持为 `pending`。用户确认前，建议逻辑不能成为实施依据。

## Shell、Git 与章节坐标

工作流可以引导读者使用 Shell、Cargo 和 Git，但不得假设读者已经熟悉这些工具。ch2 正文应简要解释当前目录、命令参数、退出码、Git 工作区、暂存区、提交，以及 commit 与 push 的区别。

章节验收后可以记录真实 Git commit SHA 作为下一章起点。没有实际提交时不得填写或猜测 SHA；此时只能报告当前工作树坐标不是不可变检查点。

## 两种“子 Agent”的边界

本工作流中的子 Agent 是开发本书和教学工程时使用的外部 AI Coding 角色。ch14 的 `SubRun/Sub-Agent` 是读者在 Rust Harness 中实现的运行时能力。使用前者不证明后者已经实现，也不得提前修改 ch14 的完成状态。

## 安全与证据

任务包和手动交接不得包含 API Key、Authorization Header、`.env` 内容或无关用户文件。默认验证不访问真实网络，不依赖真实凭据。所有角色必须区分实际执行结果、推断、未验证事项和后续建议。

## 版本策略

`v2` 是当前协议主版本；`v1` 保留只读，作为 v2 未修改部分的历史依据。破坏角色权限、人工门禁、状态语义或任务包必填字段的变更必须创建新主版本目录。非破坏性文字修订可以更新对应版本，但必须在该版本 README 的修订记录中说明。

已经获批的章节 Prompt 原文不得根据最终代码反向改写。实施结果、审计结果和验收状态必须记录在 Prompt 原文之外。

## 影响

收益：

- 读者可以在支持和不支持多 Agent 的平台上执行同一逻辑流程；
- 每次工程修改都有起点、批准、差异、验证和审计证据；
- 汇总、提交和最终验收不会被混成一句“完成”；
- 后续章节可以共享协议，而不必复制整套治理规则。

代价：

- 人工门禁增加交互次数；
- 手动会话模式需要读者复制交接包；
- 多角色输出会占用更多上下文；
- 协议只能约束协作流程，不能替代 OS 沙箱、仓库权限或运行时验证。

## 后续事项

- 单独审阅并同步 ch0 的工作流认知图；
- 为 ch1 建立 Model/Harness 边界任务包；
- 经批准后在 ch2 创建独立的累计 Rust 教学工程；
- 在 ch2 正文补充 Shell 与 Git 的最小使用过程和原理；
- 为每章保存完整读者 Prompt，并在书中展示核心段落和完整文件链接。

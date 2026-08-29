# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Mission

《从 0 搭建 AI Agent》的配套代码，以及 Forge Studio 无关的通用 Agent Harness 原型。代码必须可运行、可测试、可解释；通用 Harness 不得依赖 Forge Studio。

修改代码前必须先读 `AGENTS.md`——它是治理规则（事实源优先级、范围纪律、测试/安全/文档规则、Git 限制、完成报告）的单一入口，本节不重复。

## Commands

需要 Rust 1.85+（edition 2024）。全部 crate 共享根 `Cargo.toml` 的 `[workspace.dependencies]`。

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p p0-demo        # 本地 P0 组合示例
```

单个 crate / 单个测试：

```bash
cargo test -p <crate>                      # e.g. -p p0-demo, -p agent-core
cargo test -p tutorial-agent-harness --offline
cargo test -p <crate> <test_name_filter>   # 标准 cargo test name filter
```

Python 教学示例（默认离线，不读真实 API Key；`python3.11` 可换成任意 3.10+ 解释器名）：

```bash
python3.11 -m unittest discover -s examples/python/m0-model-call -p 'test_*.py'
python3.11 -m unittest discover -s examples/python/m1-unified-protocol -p 'test_*.py'
python3.11 -m unittest discover -s examples/python/m2-tool-runtime -p 'test_*.py'
```

书籍构建（mdbook 0.5.4 + mdbook-mermaid 0.17.0，CI 锁定的固定版本）：

```bash
cargo install mdbook --locked --version 0.5.4
cargo install mdbook-mermaid --locked --version 0.17.0
mdbook build book
```

Mermaid 用仓库内保存的本地 JS（`book/mermaid.min.js`、`book/mermaid-init.js`）渲染，不依赖在线 CDN；`mdbook-mermaid` 只是构建期预处理器，不是 Agent Runtime 依赖。

CI 完整 gate 见 `.github/workflows/ci.yml`（fmt + check + test + clippy + mdbook build + Python offline tests + MSRV 1.85 + 三平台 matrix）；`pages.yml` 把书部署到 GitHub Pages。

## 三条实现线（最重要的心智模型）

不要混淆三条"完成状态"不同的线：

- **M0–M10**：教学实现增量，每章一个里程碑。当前 M0–M2 已实现并验证。
- **P0**：一条确定性、内存内、无副作用的垂直切片，用于验证跨模块组合。它是参考实现，**不是**真实 Provider、durable recovery、生产 sandbox 或 Forge Studio 集成。
- **书籍章节 ch0–ch16**：教学阅读顺序，本身不声明实现状态。

实现状态与可运行的验证命令以 `book/src/implementations.md` 为索引；章节状态如何阅读见 `book/src/reading-guide.md`。

## Workspace crates（P0 组件边界）

`crates/` 下 8 个 crate 组成 P0 参考实现。核心依赖规则：**所有 crate 依赖 `agent-core`，`agent-core` 不依赖任何实现**；应用/示例层负责把每个边界的实现组合进 runner。`agent-core` 独占状态机推进；其他组件只返回值，不改变 run 状态。

| crate | 负责 |
|---|---|
| `agent-core` | `RunId`/`SessionId`/`EventId`、`AgentState`、`ModelInput`/`ModelAction`、`DeterministicRunner`、终止与预算、边界 trait |
| `context-engine` | 纯函数式构建/裁剪 `ModelInput`（`SimpleContextBuilder`） |
| `model-adapters` | 模型边界实现（`ScriptedMockModel`，确定性 mock provider） |
| `tool-runtime` | `ToolRegistry` + 无副作用工具（`EchoTool`） |
| `policy-engine` | 对候选 `ToolCall` 的确定性 allow/deny（`AllowListPolicy`） |
| `validators` | run 完成后的结构化验证（`RequiredOutputValidator`） |
| `session-store` | 内存 `EventLog`，append/read/replay 顺序检查（`InMemoryEventStore`） |
| `observability` | 只读投影事件 → Summary/Evidence（`summarize`） |

状态机规则、Tool lifecycle 边界、replay 不变量与显式终止条件见 `docs/architecture/p0-component-boundaries.md`；可执行契约见 `docs/specs/p0-deterministic-harness-contract.md`。

P0 组合示例在 `examples/rust/p0-demo`。本地运行期望输出 `outcome=Completed`、`event_count=11`、`evidence_count=1`。

## 教学示例（三个平行轨道）

同一里程碑通常有三个实现：

- `examples/python/<milestone>/`：正文教学语言，先展示控制流与失败路径。
- `examples/rust/<milestone>/`：更显式的类型/错误/资源边界对照。
- `tutorial/agent-harness/`：**累计 Rust 教学工程**，逐章累积（ch2→M0、ch3→M1、ch4→M2…），作为单一 crate 演进；测试必须离线可跑（`cargo test -p tutorial-agent-harness --offline`）。

主线案例是"读取工作区 `hello.txt` 第一行"。累计工程目前只注册 `read` 工具（ch4/M2）；其余 ls/find/grep/write/edit/bash 六个工具与完整失败矩阵仍只在 `examples/`，见 `book/src/labs/m2-tool-runtime.md`。

## Book（mdBook）与 docs

- `book/src/` 是正文（中文），`SUMMARY.md` 定义章节结构；附录含实现索引。
- **写 `book/src/` 正文前必须先读 `docs/writing/style-guide.md`**：作者声音、写作优先级、审阅规则以它为唯一事实源；不要把作者正文改写成通用 AI 技术文章或产品文案。
- `docs/decisions/`：已接受决策记录，是事实源优先级第 3 位（见 AGENTS.md）。
- `docs/prompts/workflow/v1|v2`：本书的 AI Coding 工作流资产（controller、roles、templates）；当前接受版本是 v2（新增 Artifact Recorder）。`docs/workflow-runs/` 存放工作流运行记录。**范围边界**：此工作流仅适用于 `book/src` 章节正文、`docs/chapters/*` 和 `tutorial/agent-harness` 的逐章教学增量；`crates/`、`examples/`、P0 组合示例、CI 配置等仓库其余工作沿用本文件与 `AGENTS.md` 的常规规则，不需要 8 角色门禁流程。
- 修改公开 API 或示例时，必须同步 `book/src/implementations.md` 与 README，保持代码、示例、书籍不脱节。

## 仓库特有硬约束

- P0 明确不含：真实 Provider、网络/浏览器工具、MCP、GUI、分布式执行、生产级沙箱、持久化数据库、并行 tool call、隐藏重试、Forge Studio/Godot 领域模型。后续能力应先经过新的决策记录。
- 不要把 P0 参考实现或"设计骨架"章节描述成已实现的生产能力（AGENTS.md 有同样的要求，这里指向最容易混淆的一处）。
- 正文与文档以中文为主，代码标识符与注释可用英文。

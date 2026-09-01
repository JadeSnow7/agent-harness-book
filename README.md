# agent-harness-book

《从 0 搭建 AI Agent》的配套代码，以及 Forge Studio 无关的通用 Agent Harness 原型。

当前仓库包含一个 P0 确定性垂直切片：

```text
request → context → deterministic mock model → policy → echo tool
        → validation → session events → evidence → outcome
```

P0 使用 Rust 2024、MSRV 1.85，并刻意不访问真实模型、网络、文件系统或 Forge Studio。公共 API 仍是实验性的。

## Run

需要 Rust 1.85 或更高版本：

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p p0-demo
```

Python 示例要求 Python 3.10 或更高版本；本机验证使用 `python3.11`。如果你的系统把其他 3.10+ 解释器命名为 `python3`，可以替换下面命令中的解释器名称：

```bash
python3.11 -m unittest discover -s examples/python/m0-model-call -p 'test_*.py'
python3.11 -m unittest discover -s examples/python/m1-unified-protocol -p 'test_*.py'
python3.11 -m unittest discover -s examples/python/m2-tool-runtime -p 'test_*.py'
```

构建书籍需要固定版本的 mdBook 与 Mermaid 预处理器：

```bash
cargo install mdbook --locked --version 0.5.4
cargo install mdbook-mermaid --locked --version 0.17.0
mdbook build book
```

Mermaid 使用随书保存的本地 JavaScript 渲染，不依赖在线 CDN。`mdbook-mermaid` 只参与文档构建，不是 Agent Runtime 依赖。

本地 P0 示例的预期结果包含 `outcome=Completed`、`event_count=11` 和 `evidence_count=1`。

M4–M10 also have one cumulative Python package and one independent example per milestone. The package is imported with PYTHONPATH=tutorial/python and remains stdlib-only, offline, in-memory, and deterministic.

## 当前教学状态与阅读入口

- M0–M2 是仓库中已有并可运行的 Python 教学示例；[阅读指南](book/src/reading-guide.md) 说明了各类实现状态如何阅读。
- 累计 Rust 教学工程 `tutorial/agent-harness/` 已从 ch2 起建立，当前覆盖 M0（ch2）、M1（ch3）与 M2（ch4，仅 `read` 工具），`cargo test -p tutorial-agent-harness --offline` 可离线验证；ls/find/grep/write/edit/bash 六个工具与完整失败矩阵仍只在 `examples/python/m2-tool-runtime` 中验证，见[实现索引](book/src/implementations.md)。
- P0 是确定性的 Rust 组合参考，用于验证跨模块组合；它与已实现并验证的 Python M3–M10 教学切片是两条不同的证据线。
- [ch0](book/src/ch0.md) 的架构地图与 AI Coding 工作流资产已经建立；工作流协议见 [`docs/prompts/workflow/v1/README.md`](docs/prompts/workflow/v1/README.md)，当前接受版本是 v2（新增 Artifact Recorder），决策记录见 [`docs/decisions/reader-ai-coding-workflow-v2.md`](docs/decisions/reader-ai-coding-workflow-v2.md)。
- Python M3–M10 已有源码、离线测试和章节正文；Rust M3–M10、真实 Provider、持久恢复和生产安全能力仍不在本轮范围内。

## Workspace

- `crates/agent-core`：ID、事件、状态机、边界 trait 和确定性 runner。
- `crates/model-adapters`：脚本化 Mock Model。
- `crates/context-engine`：有序、限额的上下文构建。
- `crates/tool-runtime`：Tool Registry 和无副作用 `echo`。
- `crates/policy-engine`：确定性 allow/deny。
- `crates/validators`：运行结果和最小 schema 的结构化验证。
- `crates/session-store`：内存 EventSink/EventLog 和 replay 顺序检查。
- `crates/observability`：从真实事件投影 Summary/Evidence。
- `examples/rust/p0-demo`：可运行的端到端组合与集成测试。

架构、契约、评测夹具和限制见 [`docs/architecture/p0-component-boundaries.md`](docs/architecture/p0-component-boundaries.md)、[`docs/specs/p0-deterministic-harness-contract.md`](docs/specs/p0-deterministic-harness-contract.md) 和 [`fixtures/p0-scenarios.json`](fixtures/p0-scenarios.json)。

## Scope

P0 不包含真实 Provider、网络/浏览器工具、MCP、GUI、分布式执行、生产级沙箱、持久化数据库、并行 tool call、隐藏重试或 Forge Studio/Godot 领域模型。后续能力应先经过新的决策记录。

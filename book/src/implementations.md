# 实现索引与验证命令

本页是正文与仓库代码之间的导航。实现索引只列出当前存在且可以通过离线测试验证的版本，不创建尚不存在的 TypeScript 入口。

## M0：一次模型调用

| 语言 | 源码 | 测试 | 状态 |
|---|---|---|---|
| Python | `examples/python/m0-model-call/chat_once.py` | `python3.11 -m unittest discover -s examples/python/m0-model-call -p 'test_*.py'` | 已实现并验证 |
| Rust | `examples/rust/m0-model-call/src/main.rs` | `cargo test -p m0-model-call` | 已实现并验证 |
| Rust（累计工程） | `tutorial/agent-harness/` | `cargo test -p tutorial-agent-harness --offline` | 已实现并验证 |

核心边界是配置、请求构造、HTTP 传输和响应解析；默认测试不联网、不读取真实 API Key。

## M1：统一协议

| 语言 | 源码 | 测试 | 状态 |
|---|---|---|---|
| Python | `examples/python/m1-unified-protocol/` | `python3.11 -m unittest discover -s examples/python/m1-unified-protocol -p 'test_*.py'` | 已实现并验证 |
| Rust | `examples/rust/m1-unified-protocol/` | `cargo test -p m1-unified-protocol` | 已实现并验证 |
| Rust（累计工程） | `tutorial/agent-harness/` | `cargo test -p tutorial-agent-harness --offline` | 已实现并验证 |

重点检查 `Message`、`ContentBlock`、`ToolDefinition`、`call_id`、function call 映射和安全错误。

## M2：一步 Tool Runtime

| 语言 | 源码 | 测试 | 状态 |
|---|---|---|---|
| Python | `examples/python/m2-tool-runtime/` | `python3.11 -m unittest discover -s examples/python/m2-tool-runtime -p 'test_*.py'` | 已实现并验证 |
| Rust | `examples/rust/m2-tool-runtime/` | `cargo test -p m2-tool-runtime` | 已实现并验证 |
| Rust（累计工程） | `tutorial/agent-harness/` | `cargo test -p tutorial-agent-harness --offline` | 已实现并验证 |

累计工程只注册 `read`；ls/find/grep/write/edit/bash 与原子写入仍只存在于 examples/。主线案例使用 `read hello.txt`。七工具、Workspace 和完整失败矩阵见 [M2 Tool Runtime 实验](labs/m2-tool-runtime.md)。

## M3：最小 Agent Loop

| 语言 | 源码 | 测试 | 状态 |
|---|---|---|---|
| Python | `examples/python/m3-agent-loop/` | `python3 examples/python/m3-agent-loop/test_loop.py -v` | 已实现并验证 |
| Rust | `examples/rust/m3-agent-loop/` | `cargo test -p m3-agent-loop` | 已实现并验证 |

M3 独立于 P0 的 `DeterministicRunner` 组合（见 [ch5 §5.6](ch5.md#56-m3-的验证合同与当前边界)）：两侧各自实现了模型—工具—模型的循环、预算边界、Policy、Validation，以及 `Completed`/`Failed`/`BudgetExhausted`/`PolicyDenied`/`Cancelled` 五种互斥终态。逐条对应关系见 [`evals/m3-test-matrix.md`](../../evals/m3-test-matrix.md)；两侧都复用了各自语言里已有的、不属于 P0 编排层的基础类型（Python 复用 M2 的 `ToolRegistry` 等，Rust 复用 `agent_core` 的数据类型和 `tool_runtime::ToolRegistry`），循环控制流本身是全新实现。累计工程 `tutorial/agent-harness/` 暂不新增 ch5 增量——该工程的 M2（ch4）目前只注册了 `read`，尚未完整并入七工具，在 M2 完整并入之前提前叠加 M3 会打乱累计工程自身的推进顺序。

## P0：确定性组合切片

P0 位于 `examples/rust/p0-demo`，组合 `DeterministicRunner`、`ScriptedMockModel`、`SimpleContextBuilder`、`AllowListPolicy`、`ToolRegistry`、`RequiredOutputValidator`、`InMemoryEventStore` 和 `Evidence`。它是 P0 参考实现，不是真实 Provider、durable recovery、生产级 sandbox 或 Forge Studio 集成。

验证命令：

```bash
cargo test -p p0-demo
cargo run -p p0-demo
```

## 尚未提供的实现

M4–M10 的章节目前是设计骨架；TypeScript 实现也尚未加入仓库。本页不会为缺失实现创建空链接。后续只有在源码、离线测试和必要的类型检查都通过后，才新增对应语言入口。

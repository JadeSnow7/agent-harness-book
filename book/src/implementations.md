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

累计工程只注册 `read`；ls/find/grep/edit/bash 与原子写入仍只存在于 examples/。主线案例使用 `read hello.txt`；`write` 在 ch4.md 4.5 节作为教学捷径单独演示，出自这里的 Python 实现，同样尚未并入累计工程。七工具、Workspace 和完整失败矩阵见 [M2 Tool Runtime 实验](labs/m2-tool-runtime.md)。

## M3：最小 Agent Loop

| 语言 | 源码 | 测试 | 状态 |
|---|---|---|---|
| Python（目标侧实现） | `examples/python/m3-agent-loop/loop.py` | `python3 examples/python/m3-agent-loop/test_loop.py -v` | 已实现并验证 |
| Python（源侧原型） | `examples/python/m3-agent-loop/agent_loop.py` | `python3.11 -m unittest discover -s examples/python/m3-agent-loop -p 'test_agent_loop.py'` | 已实现并验证 |
| Rust | `examples/rust/m3-agent-loop/` | `cargo test -p m3-agent-loop` | 已实现并验证 |

目标侧的 `loop.py` 与 Rust 实现分别覆盖模型—工具—模型循环、预算边界、Policy、Validation，以及 `Completed`/`Failed`/`BudgetExhausted`/`PolicyDenied`/`Cancelled` 五种互斥终态。源侧 `agent_loop.py` 则在预算内反复调用模型、执行恰好一个工具候选，直到四个 `StopReason`（`COMPLETED`、`BUDGET_EXHAUSTED`、`AMBIGUOUS_TOOL_REQUEST`、`UNRECOGNIZED_ACTION`）之一显式停止；它只注册 `read`，复用 M2 已验证的 `ToolRegistry`/`bridge`。两条 Python 实现来自独立历史，入口和测试命令不可互换。

## M4–M10：累计 Python Harness

| 里程碑 | 语言 | 源码 | 测试命令 | 状态 | 边界 |
|---|---|---|---|---|---|
| M4 / CH06 | Python 3.11 | tutorial/python/agent_harness/context.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m4-context-engineering -p 'test_*.py' | 已实现并验证 | 内存 UTF-8 预算；不是 tokenizer/检索服务 |
| M5 / CH07–08 | Python 3.11 | tutorial/python/agent_harness/identity.py, events.py, snapshots.py, idempotency.py, recovery.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m5-session-recovery -p 'test_*.py' | 已实现并验证 | append-only 内存教学账本；不是 durable exactly-once |
| M6 / CH09–10 | Python 3.11 | tutorial/python/agent_harness/tool_definition.py, tool_descriptor.py, effects.py, policy.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m6-effects-policy -p 'test_*.py' | 已实现并验证 | 字符串 ChangeSet 与内存 handler；不是 OS sandbox |
| M7 / CH11 | Python 3.11 | tutorial/python/agent_harness/validation.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m7-validation -p 'test_*.py' | 已实现并验证 | 调用者提供的确定性检查；不是领域验收平台 |
| M8 / CH12 | Python 3.11 | tutorial/python/agent_harness/evidence.py, events.py, _base.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m8-evidence-observability -p 'test_*.py' | 已实现并验证 | 内存 evidence/只读投影；不是 tracing 或 secrets manager |
| M9 / CH13–14 | Python 3.11 | tutorial/python/agent_harness/stop_policy.py, runner.py, delegation.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m9-loop-delegation -p 'test_*.py' | 已实现并验证 | 串行 fake provider 与 child function；不是外部 Agent scheduler |
| M10 / CH15–16 | Python 3.11 | tutorial/python/agent_harness/extensions.py, runner.py, provider.py | PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python python3.11 -m unittest discover -s examples/python/m10-extension-composition -p 'test_*.py' | 已实现并验证 | fake extension composition；不是插件隔离或 HUSH 集成 |

累计包核心回归：

~~~bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python \
  python3.11 -m unittest discover -s tutorial/python -p 'test_*.py'
~~~

M3 独立于 P0 的 `DeterministicRunner` 组合（见 [ch5 §5.6](ch5.md#56-m3-的验证合同与当前边界)）。目标侧 Python/Rust 两份实现覆盖五种互斥终态；源侧 Python 原型单独覆盖四种 `StopReason`。逐条对应关系见 [`evals/m3-test-matrix.md`](../../evals/m3-test-matrix.md)；各实现复用了各自语言里已有的、不属于 P0 编排层的基础类型，循环控制流本身是全新实现。累计 Rust 工程 `tutorial/agent-harness/` 暂不新增 ch5 增量——该工程的 M2（ch4）目前只注册了 `read`，尚未完整并入七工具，在 M2 完整并入之前提前叠加 M3 会打乱累计工程自身的推进顺序。

## P0：确定性组合切片

P0 位于 `examples/rust/p0-demo`，组合 `DeterministicRunner`、`ScriptedMockModel`、`SimpleContextBuilder`、`AllowListPolicy`、`ToolRegistry`、`RequiredOutputValidator`、`InMemoryEventStore` 和 `Evidence`。它是 P0 参考实现，不是真实 Provider、durable recovery、生产级 sandbox 或 Forge Studio 集成。

验证命令：

```bash
cargo test -p p0-demo
cargo run -p p0-demo
```

## 尚未提供的实现

Rust M4–M10、Rust 累计工程对应增量和 TypeScript 实现仍未加入仓库。本页不为缺失实现创建空链接；Python M4–M10 入口只表示本地教学原型已实现并验证，不表示生产能力或 HUSH Runtime 集成。

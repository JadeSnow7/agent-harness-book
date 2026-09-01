# ch4 需求 Prompt 草案：从一次工具候选到一步闭环

- **章节：** ch04
- **Prompt 原文状态：** 草案原文（BEGIN/END 标记之间）保留不变，不根据最终代码反向改写；以下状态字段为回填更新（2026-08-21）
- **需求确认状态：** Confirmed（回填依据：`tutorial/agent-harness/` 已注册 `read` 工具并通过 `cargo test -p tutorial-agent-harness --offline`，与本草案范围一致）
- **实施批准状态：** Approved（回填依据同上）
- **实施状态：** Completed（依据：git tag `ch04` 指向 commit `477b368`「feat: add read tool to cumulative Rust tutorial project (M2)」，`book/src/ch4.md` 已标注"已实现并验证"）
- **回填说明：** 本章实际实施发生在 `docs/chapters/`、`docs/workflow-runs/` 等工作流留痕资产建立之前，未保存独立的需求确认、计划批准和执行后审计记录文件。以上状态字段仅依据当前可验证的源码、测试和 git tag 回填，不代表 v2 协议要求的角色交接证据链在当时被完整保存；这与 ch05（`docs/chapters/ch05/prompt.md`）留有完整同步记录的情况不同。
- **上一章不可变坐标：** 无可用 commit SHA。截至本草案撰写时，`tutorial/agent-harness/` 已经在工作树中完成 ch2（M0）与 ch3（M1 统一协议），`book/src/ch3.md` 已标注“状态：已实现并验证”，但仓库最近一次 Git 提交发生在这些改动之前——`protocol.rs`、`openai_responses.rs`、`tests/unified_protocol.rs` 以及正文改动都还是未提交的工作树内容。需求分析必须重新核对当前 `git status`，把“ch2+ch3 已在工作树完成”和“存在不可变的 ch3 commit”当成两件不同的事，不得互相替代。
- **适用工作流：** `docs/prompts/workflow/v2/`（v1 七角色 + Artifact Recorder；`v2` 是当前接受版本，见 `docs/decisions/reader-ai-coding-workflow-v2.md`）

这份文件保存 ch4 的需求 Prompt 草案。它用于启动任务与需求分析，而不是授权编码。
需求确认后，计划角色还必须生成精确文件白名单和完整编码 Prompt；只有用户明确回复
“批准实施”后，编码角色才可以修改工程。Prompt 一旦获批，不得根据最终代码反向改写；
实施、审计、提交和验收证据应保存到独立结果文件。

<!-- BEGIN DRAFT PROMPT -->
任务名称：
把一次工具候选变成一步闭环观察：M2 Tool Runtime 并入累计 Rust 教学工程

任务性质：
这是《从 0 搭建 AI Agent》逐章累计教学工程的第三个真实代码增量，对应 M2 Tool
Runtime，紧接在已完成的 ch2（M0 模型调用）与 ch3（M1 统一协议）之后。任务需要先
完成只读需求分析，再经过计划、实施、审计、汇总、提交、提交审核和归档记录；当前
Prompt 只授权任务与需求分析，不授权修改代码、正文、配置或 Git 状态。

`examples/python/m2-tool-runtime/` 与 `examples/rust/m2-tool-runtime/` 已经实现并验证
了完整的七工具 Runtime（`read`、`ls`、`find`、`grep`、`write`、`edit`、`bash`），但那是
独立的 M2 证据链，不是本章新实现的一部分，也不得被直接复制进累计工程。本章只把
`read` 这一个工具、`ToolCall`/`ToolResult`/`ToolRegistry`/`Workspace` 边界和固定两次
调用的“一步闭环”接到 `tutorial/agent-harness/` 已有的 M0/M1 基础上。

## 一、任务目标

需求分析应核对并向用户确认：ch4 是否以如下目标进入计划阶段。

1. 在 `tutorial/agent-harness/` 现有 `config`/`transport`/`protocol`/`openai_responses`/
   `model_call` 基础上，新增 Runtime 层：结构化 `ToolCall`/`ToolResult`/`ToolStatus`/
   `ToolError`、`ToolRegistry`、教学级 `Workspace` 边界，以及唯一注册的 `read` 工具。
2. 用显式桥接函数把 `protocol::ToolUseBlock` 转成 Runtime `ToolCall`、把 `ToolResult`
   转回 `protocol::ToolResultBlock`/`Message`，`call_id` 全程原样贯通，不与协议类型
   合并成同一套结构。
3. 实现固定两次调用的一步闭环：第一次模型调用必须恰好提出一个工具候选；Runtime
   执行该候选并产生 `Observation`；第二次模型调用携带该观察，必须给出最终文本、
   不得再次请求工具。任何一步不满足都应作为明确失败返回，而不是静默重试或循环。
4. 让 `read` 的失败路径完整：未知工具、参数不合法、路径越界（`..`、绝对路径、经由
   不存在父目录发生的逃逸）、目标不存在或不是文件、读取时的字节/行数截断都必须有
   可复现的离线测试。
5. 默认验收完全离线：一步闭环的两次模型调用都通过 Fake Transport 驱动，不访问真实
   网络、不读取真实 API Key；同时保留一个显式、手动触发的真实一步闭环入口。
6. 让读者理解：工具候选被执行，不等于任务被验证完成；`ToolResult.succeeded` 只说明
   这一次调用有没有抛出异常，不等于上层目标达成；本章仍然不是 Agent Loop——第二次
   模型调用如果还想要工具，必须被拒绝并报告为 M3 之后的问题。
7. 把 Rust 累计工程、Python M2 原型、ch4 正文和实现索引重新同步，同时在正文中如实
   说明：读者现在拥有的是“一步”，不是“循环”。

## 二、本章结束时的可观察结果

若后续计划获得批准并完成实施，读者应能从仓库根目录完成以下动作：

1. 在离线、无真实凭据的环境中编译并运行累计工程新增的 Runtime 测试；
2. 看到一次完整闭环：模型提出 `read` 候选 → Runtime 校验并执行 → Workspace 拒绝越
   界路径或返回文件内容 → 结果带着原始 `call_id` 回到第二次模型调用 → 得到最终文本；
3. 看到未知工具、非法参数、路径越界、目标不存在等场景分别产生结构化失败，而不是
   进程崩溃或被静默吞掉；
4. 看到第一次模型调用如果没有请求恰好一个工具、或第二次模型调用又请求了工具，闭环
   会显式报错，而不是继续循环；
5. 在自愿提供环境变量和真实 Provider 的前提下，手动触发一次真实的一步闭环，并理解
   这不属于默认验收证据；
6. 说出 `ToolCall`、`ToolResult`、`Workspace` 边界、一步闭环和 Agent Loop 之间的差异，
   以及本章有意留下的限制；
7. 明白 `read` 成功只证明“这一次系统调用没有出错”，不能证明模型的最终回答确实满足
   了用户目标——验证仍然缺失，属于后续章节的问题。

## 三、必须先核对的事实基线

任务与需求分析角色必须只读检查当前 checkout，不得直接相信本 Prompt 中可能随时间
变化的状态。至少核对：

- 当前工作目录和仓库根目录；
- `git status --short --branch`，并明确区分“ch2/ch3 已经完成”和“存在不可变的上一章
  commit”这两件事——当前两者都不成立，只有前者成立；
- `tutorial/agent-harness/` 现有 `Cargo.toml`、`src/`、`tests/` 的真实结构（`config.rs`、
  `model_call.rs`、`openai_responses.rs`、`protocol.rs`、`transport.rs`、`lib.rs`、
  `main.rs`，以及 `tests/model_call.rs`、`tests/unified_protocol.rs`）；
- `tutorial/agent-harness/src/openai_responses.rs` 中已存在但当前没有测试或调用点的
  `tool_result_message` 函数，及其与本章“结果编码为工具消息”职责的重叠；
- `AGENTS.md` 和任务范围内的已接受决策；
- `docs/decisions/reader-ai-coding-workflow-v2.md`；
- `docs/decisions/repository-baseline.md` 中的依赖、安全和验证边界；
- `book/src/ch3.md`、`book/src/ch4.md`（如已存在草稿）、`book/src/implementations.md`
  和 [M2 Tool Runtime 实验](../../book/src/labs/m2-tool-runtime.md)；
- `examples/python/m2-tool-runtime/`（`tool_types.py`、`registry.py`、`workspace.py`、
  `bridge.py`、`one_step.py`、`tools/read.py`）的业务逻辑和离线测试；
- `examples/rust/m2-tool-runtime/`（`tool_types.rs`、`registry.rs`、`workspace.rs`、
  `bridge.rs`、`one_step.rs`、`tools/read.rs`）的当前结构、能力和测试缺口，注意它依赖
  独立的 `examples/rust/m1-unified-protocol` crate，累计工程必须改为依赖
  `tutorial/agent-harness` 自己的 `protocol` 模块，不能引入对 `examples/` 的编译期依赖；
- ch0 已保存的 Prompt、设计和工作流资产是否只是工作树坐标，还是已有不可变提交坐标。

已知但必须重新验证的基线假设：

- `tutorial/agent-harness/` 已完成 M0 与 M1，`chat_once`/`complete`/`build_model_request`/
  `format_response` 和统一协议类型都已经存在并通过测试；
- 现有 Python/Rust M2 示例已经把 Runtime 类型、Registry、Workspace 和一步闭环跑通，
  并覆盖了七个工具的失败矩阵，但那是独立证据链，仅供业务参考；
- 当前根 workspace 仍使用 Rust Edition 2024、MSRV 1.85、`reqwest`、`serde_json`；
- 当前工作区可能包含用户未提交修改（包括整个 ch2+ch3 增量），任何实现都必须保护
  这些修改；
- 当前没有可作为 ch4 起点的、经确认的上一章 commit SHA。

如果实际 checkout 与上述假设不同，必须以当前事实为准，列出冲突并停止在需求确认
门禁；不得静默调整任务目标。

## 四、待用户确认的业务逻辑

以下内容来自 `examples/python/m2-tool-runtime/`、`examples/rust/m2-tool-runtime/` 和
[M2 Tool Runtime 实验](../../book/src/labs/m2-tool-runtime.md)，是需求草案，不是已经
批准的实施依据。任务与需求分析角色必须逐项核对，并将 `user_confirmation` 保持为
`pending`。

### 输入

- 一个已经在 ch3 建立的 `ModelRequest`（`messages`、可选 `system`）；
- 一份工具注册表，本章只注册 `read`；
- 一个教学级 `Workspace` 根目录（测试使用临时目录 + 固定 fixture，例如 `hello.txt`）；
- 一个真实或伪造的 HTTP Transport（复用 ch2/ch3 已有的 `Transport` trait）。

本章不接收会话历史之外的额外状态、不接收预算或停止策略参数、不接收除 `read` 以外
的工具声明。

### 输出

- 成功时：一个 `OneStepResult`（或等价结构），保留第一次响应、`ToolResult`、追加后的
  请求和最终响应，供测试断言 `call_id` 和顺序；
- 失败时：可分类、可安全展示的 Runtime/闭环错误，不暴露真实文件系统绝对路径之外的
  敏感信息、不暴露 API Key 或 Authorization Header；
- `read` 成功时返回 `{ path, content, line_count, truncated }`，`content` 按
  `"{行号}: {原文}"` 逐行拼接，行号从 `offset`（默认 1）开始；
- `read` 失败时返回结构化 `ToolResult { status: Failed, error }`，错误文本足够定位问题
  但不回显整份文件内容作为“证据”。

### 主流程

```text
读取已确认的 ModelRequest（消息 + 可选 system）
→ 从 ToolRegistry 生成工具声明并附加到请求
→ 第一次调用 → 必须恰好包含一个 ToolUseBlock
→ ToolUseBlock → ToolCall（保留 call_id）
→ ToolRegistry.execute：
    未注册 → Failed(unknown tool)
    参数不合法 → Failed(invalid arguments)
    Workspace 解析失败（越界/不存在/不是文件） → Failed
    读取成功 → Succeeded(content, line_count, truncated)
→ ToolResult → ToolResultBlock（tool_use_id = call_id）
→ 追加 assistant 消息与 tool 消息，发起第二次调用
→ 第二次响应必须没有工具候选、且有非空文本
→ 返回闭环结果
```

### 失败分类

至少区分：

- 第一次响应没有工具候选，或包含不止一个工具候选；
- 请求的工具未在 Registry 中注册；
- 工具参数校验失败（`path` 缺失/非字符串、`offset`/`limit` 非正整数）；
- 路径越界：`..` 分量、绝对路径逃逸、经由尚不存在父目录发生的间接逃逸；
- 目标路径不存在，或存在但不是文件；
- 第二次响应仍然包含工具候选；
- 第二次响应没有可展示的最终文本。

错误信息不得包含：

- API Key、Authorization Header；
- 与任务无关的环境变量；
- 完整原始 Provider 响应正文；
- 测试中用于证明不泄露的 sentinel secret；
- 工作区根目录之外的真实绝对路径细节（教学边界，不承诺生产级信息隔离，但不应
  主动把越界尝试的完整宿主机路径回显给调用者）。

### Workspace 解析规则

- 拒绝任何包含 `..` 分量的路径；
- 相对路径相对 Workspace root 解析；绝对路径必须仍落在 root 内，否则失败；
- 目标尚不存在时，向上找到最近的已存在祖先目录并 canonicalize，防止通过尚未创建
  的路径或符号链接绕过边界；
- `read` 要求目标必须存在且是普通文件；
- 本章 `Workspace` 候选范围只需要 `resolve` 和“转换为相对 root 的路径”两个能力；是否
  提前引入 `write`/`edit` 需要的原子替换，必须在计划阶段单独确认，不能默认加入。

### 状态变化

本章仍不维护跨请求的 Session 或 Agent 状态。一次一步闭环只产生两次进程内模型调用
和一次工具执行；两次调用之间的临时状态（追加的消息列表）只存在于当前函数调用栈
内，不持久化、不支持恢复、不支持重放。

### 有意限制

- 只支持一步、固定两次调用的闭环，不支持多轮工具调用或循环；
- 只注册 `read`，不引入 `ls`/`find`/`grep`/`write`/`edit`/`bash`；
- 不实现预算、重试、超时升级或停止策略；
- 不实现会话、恢复、Policy、Sandbox、Validation 或 Evidence；
- `Workspace` 是教学级路径边界，不是 OS 级 sandbox，不阻止真正拥有该进程权限的
  代码越权；
- 不把工具执行成功当作任务级验证通过的证据。

## 五、累计 Rust 工程的候选边界

以下是进入需求确认的候选方案，不是已批准的文件级设计。

### 模块划分

候选新增模块（供计划角色裁剪，不是强制清单）：

- `tool_types`：`ToolStatus`、`ToolSpec`、`ToolCall`、`ToolResult`、`ToolError`、
  `Tool` trait；
- `workspace`：教学级路径边界，范围见第四节；
- `registry`：`ToolRegistry`（`register`/`specs`/`execute`），以及 `require_object`/
  `require_string` 一类的最小参数校验辅助；
- `tools`（或 `tools::read`）：唯一的 `ReadTool` 实现；
- `bridge`：`spec_to_tool_definition`、`tool_use_to_call`、`result_to_tool_result_block`、
  `result_to_message`——这里必须先核对并解决第三节提到的 `tool_result_message` 重叠，
  不能制造两套等价逻辑；
- `one_step`（或等价命名）：`request_with_registry_tools`、`run_one_tool_step`。

计划角色应避免为追求目录数量而机械拆分文件；如果某些职责合并到更少文件里仍然
清楚，应优先选择更少的新增文件。

任务分析必须请用户确认：

1. 是否按上述候选划分模块，或合并为更少文件；
2. `Workspace` 是否只实现 `read` 所需的最小能力（不含 `atomic_write`）；
3. `tool_result_message`（已存在于 `openai_responses.rs`）与本章 `bridge` 模块的关系：
   复用、迁移改名，还是判定为死代码后移除；
4. 测试用的 Workspace fixture（例如 `hello.txt`）放在
   `tutorial/agent-harness/tests/fixtures/` 还是其他候选路径，且不得复用或修改
   `examples/` 下的 fixtures。

### 与 M1 的边界

`ToolCall`/`ToolResult` 是 Runtime 内部类型，`ToolUseBlock`/`ToolResultBlock` 是协议层
类型；两者不得合并。`bridge` 模块是唯一允许知道两侧类型的地方。本章不得反过来让
`protocol` 模块认识 Runtime 类型，保持 ch3 建立的分层方向不变。

## 六、Python 原型的角色

业务参考：

- `examples/python/m2-tool-runtime/tool_types.py`
- `examples/python/m2-tool-runtime/registry.py`
- `examples/python/m2-tool-runtime/workspace.py`
- `examples/python/m2-tool-runtime/bridge.py`
- `examples/python/m2-tool-runtime/one_step.py`
- `examples/python/m2-tool-runtime/tools/read.py`
- 对应的 `test_runtime.py`

Python 原型提供：

- 结构化 `ToolResult`（含 `succeeded`/`as_text`）；
- `ToolRegistry.execute` 的三层失败收敛（未知工具/参数校验/执行异常）；
- `Workspace` 的路径边界和越界检测；
- `read` 的有界读取（`DEFAULT_MAX_LINES`、`DEFAULT_MAX_BYTES`、`offset`/`limit`、
  行号前缀）；
- `run_one_tool_step` 的固定两次调用闭环及其三种显式失败（首次候选数量不对、二次
  仍要工具、二次无文本）。

Rust 实现不得机械逐行翻译 Python；`examples/rust/m2-tool-runtime/` 已经提供了一份
Rust 同构参考，可以作为业务对照，但累计工程必须改写其 `protocol`/`m1-unified-protocol`
依赖，接到 `tutorial/agent-harness` 自己的类型上，而不是引入对 `examples/` crate 的
编译期依赖。计划必须说明 Rust 在所有权、错误类型、trait 对象（`Box<dyn Tool>`）、
可变借用（`&mut self` 的 `execute`）等方面的选择。

## 七、依赖与公共 API 候选约束

需求分析和计划阶段应优先验证能否只复用工作区已有依赖（`reqwest`、`serde_json`）。

默认不为本章引入文件系统 mock 库、随机数/临时目录以外的新依赖、完整 Agent SDK 或
沙箱库。如计划认为必须新增依赖（例如更完善的临时目录管理），必须说明当前 ch4 的
直接用途、离线可用性和 MSRV 影响，并单独请求用户确认。

本章不承诺稳定公共 API。Runtime 类型、`ToolRegistry`、`Workspace` 和 `read` 的可见性
应以 CLI/集成测试实际需要为边界，不为 ch5 的 Agent Loop 预留占位 trait、抽象循环
接口或未经验证的扩展点。

## 八、默认离线测试契约

后续计划至少应覆盖以下正常、错误和边界行为：

1. 未注册工具触发 `unknown tool` 失败，且不进入参数校验或执行；
2. `path` 缺失或非字符串时在执行前失败；
3. `offset`/`limit` 为非正整数、非整数或布尔值时失败；
4. 相对路径正确解析到 Workspace root 内的文件；
5. 含 `..` 的路径、绝对路径逃逸、经由不存在父目录的间接逃逸均被拒绝；
6. 目标不存在或不是文件时明确失败；
7. 正常读取返回带行号的内容、正确的 `line_count`，且 `offset`/`limit` 生效；
8. 超过 `DEFAULT_MAX_BYTES`/`DEFAULT_MAX_LINES` 时设置 `truncated: true`；
9. `call_id` 从 `ToolUseBlock` 经 `ToolCall`、`ToolResult` 到 `ToolResultBlock` 全程一致；
10. 第一次响应没有工具候选、或有多个工具候选时，闭环返回明确错误而不是猜测执行
    第一个；
11. 第二次响应仍包含工具候选时，闭环返回明确错误（不是 Agent Loop）；
12. 第二次响应没有非空文本时，闭环返回明确错误；
13. 失败信息不包含 sentinel secret、真实密钥或与任务无关的敏感路径细节；
14. 默认测试在没有真实环境变量和网络的条件下确定性通过，使用与 ch2/ch3 一致的
    Fake Transport。

测试不得通过删除、跳过或降低断言获得绿色结果；不得使用真实 API Key、真实
Provider、`sleep` 或不可控 timing race；不得访问工作区外的真实文件系统路径。

## 九、ch4 正文与实现索引同步

如果需求和计划随后获得批准，`book/src/ch4.md`（新建或续写）应在保留作者声音和
Python-first 教学顺序的前提下，与新的 Runtime 增量同步，遵循
`docs/writing/style-guide.md` 的章节推进骨架和状态词汇（`已实现并验证`、
`P0 参考实现`、`设计骨架/尚未实现`），并包含 OUTLINE.md 要求的章末工程分析层
（本章之前系统处于什么状态、已实现与骨架的划分、新增抽象的收益与风险、当前成熟度
阶段、有意留下的技术债）。

正文至少应让读者理解：

- 工具候选（`ToolUseBlock`）和工具执行结果（`ToolResult`/`Observation`）是两件事，
  中间隔着 Registry 和 Workspace 的判断；
- 一步闭环固定两次调用，不是循环，不能处理需要第三次工具调用的任务；
- `Workspace` 边界防止路径越界，但不是操作系统级沙箱；
- 本章之后，工具候选终于能落地执行，但“执行成功”依然不等于“任务被验证完成”。

正文和实现索引必须明确区分：

- 旧 Python/Rust M2 示例（七工具、完整失败矩阵）：已有并验证的参考证据，见
  [M2 Tool Runtime 实验](../../book/src/labs/m2-tool-runtime.md)；
- 新累计 Rust 工程 ch4 增量（仅 `read`）：只有代码、测试、文档和审计实际完成后才能
  标记为“已实现并验证”；
- P0：确定性组合参考，不等于新工程已经完成 M3 及之后的能力。

## 十、计划阶段的候选文件范围

以下范围只用于任务分析和计划，不构成当前修改授权。计划角色必须根据确认后的模块
划分给出精确到文件的白名单。

候选新增：

- `tutorial/agent-harness/src/tool_types.rs`
- `tutorial/agent-harness/src/workspace.rs`
- `tutorial/agent-harness/src/registry.rs`
- `tutorial/agent-harness/src/tools/`（含 `read.rs` 与必要的 `mod.rs`）
- `tutorial/agent-harness/src/bridge.rs`
- `tutorial/agent-harness/src/one_step.rs`（或计划确认后的等价命名）
- `tutorial/agent-harness/tests/` 下新增的 Runtime/一步闭环离线集成测试
- `tutorial/agent-harness/tests/fixtures/`（例如 `hello.txt`，仅供本章测试使用）
- `docs/chapters/ch04/design.md`（已作为候选草案存在，计划阶段可修订为最终版本）
- `docs/chapters/ch04/` 下独立保存的任务包、实施结果、审计或完成报告
- `book/src/assets/ch04/`（如计划需要正文示意图资产）

候选修改：

- `tutorial/agent-harness/src/lib.rs`，仅用于导出计划确认的新模块和类型；
- `tutorial/agent-harness/src/openai_responses.rs`，仅限于解决第三/五节提到的
  `tool_result_message` 重叠问题（复用、迁移或移除），不得顺带改动 ch3 已验证的
  编解码逻辑；
- 根 `Cargo.toml`/`Cargo.lock`，仅在计划证明确有必要新增依赖时；
- `book/src/ch4.md`，仅同步真实实现、命令和限制；
- `book/src/implementations.md`，仅增加经验证的新累计工程 M2 入口并保留旧实现状态；
- `book/src/SUMMARY.md`，仅在 ch4 正文首次成文时补充链接（如尚未存在）。

默认禁止修改：

- `examples/python/m0-model-call/`、`examples/python/m1-unified-protocol/`、
  `examples/python/m2-tool-runtime/`；
- `examples/rust/m0-model-call/`、`examples/rust/m1-unified-protocol/`、
  `examples/rust/m2-tool-runtime/`、`examples/rust/p0-demo/`；
- `crates/`；
- `fixtures/`（P0 使用的顶层 fixtures，与本章新增的 `tutorial/agent-harness/tests/fixtures/`
  是不同目录）；
- `tutorial/agent-harness/src/config.rs`、`model_call.rs`、`openai_responses.rs` 中已由
  ch2/ch3 验证的部分（除第三/五节明确允许的 `tool_result_message` 处理外）、
  `protocol.rs`、`transport.rs`；
- `book/src/ch0.md`、`ch1.md`、`ch2.md`、`ch3.md` 和 ch5–ch16；
- `.github/`、remote、branch、tag 和发布配置；
- 用户已有、与 ch4 无关的未提交文件（包括但不限于任何与本任务描述范围无关的
  改动）。

已批准的 `docs/chapters/ch04/prompt.md` 原文不属于编码角色的修改范围。若需求审阅要求
调整本草案，应在批准前完成；批准后产生的新争议必须创建版本化修订 Prompt，不能反向
改写原文。

## 十一、建议验证与证据

计划角色应根据最终包名和路径给出可直接运行的命令。默认至少考虑：

```text
cargo fmt --all -- --check
cargo check -p tutorial-agent-harness --offline
cargo test -p tutorial-agent-harness --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
python3.11 -m unittest discover -s examples/python/m2-tool-runtime -p 'test_*.py'
mdbook build book --dest-dir <临时输出目录>
git diff --check
git status --short
```

还应验证：

- 在临时目录 fixture 上运行离线一步闭环测试，确认默认测试没有访问真实网络或真实
  文件系统之外的路径；
- 路径越界、未知工具、参数非法等失败路径都有对应断言，而不是只测试成功路径；
- 实际变更文件严格等于批准白名单；
- `tool_result_message` 的处理方式（复用/迁移/移除）在完成报告中有明确说明和理由；
- mdBook 生成的 ch4 页面（如涉及）存在，include 和内部链接没有错误；
- 如本机安装 Rust 1.85，执行 MSRV 检查；未安装时如实报告，不得声称通过；
- 不把未运行的真实 Provider 一步闭环写成成功；
- 不把根 workspace 绿色结果写成 GitHub Actions 已通过。

mdBook 必须构建到临时目录，不能用构建产物污染工作区。验证如因离线依赖缓存、工具
缺失或现有无关修改失败，必须区分本任务缺陷与环境/基线问题，不得扩大范围修复。

## 十二、人工门禁与角色顺序

必须遵循 `docs/prompts/workflow/v2/`：

```text
任务与需求分析
→ 用户确认需求理解
→ 计划与完整编码 Prompt
→ 用户明确批准实施
→ 编码与验证
→ 执行后审计
→ 用户确认审计结论
→ 十一项汇总报告
→ 用户批准本地提交
→ 白名单提交
→ 提交审核
→ 归档记录（Artifact Recorder）
→ 用户最终验收
```

当前只允许执行第一步。任务分析完成后必须停在
`awaiting_requirement_confirmation`，向用户展示事实、假设、争议、风险和待确认决定。

至少要求用户确认：

1. 是否接受“ch2+ch3 已在工作树完成但尚未提交”作为 ch4 的真实起点，而不是等待一次
   正式的 ch3 commit 再开始；
2. 是否确认第四节的输入、输出、主流程、失败分类和有意限制；
3. 是否确认本章只注册 `read`，`ls`/`find`/`grep`/`write`/`edit`/`bash` 仍留在 `examples/`
   与实验附录；
4. 是否确认默认验收只使用 Fake Transport，真实网络调用不作为通过条件；
5. 是否确认 `Workspace` 只实现 `read` 所需能力，暂不引入 `atomic_write`；
6. 如何处理 `tutorial/agent-harness/src/openai_responses.rs` 中已存在但未使用的
   `tool_result_message`；
7. 是否确认本章不引入 ch5 的 Agent Loop、预算或停止策略。

“开始拟写 Prompt”、对本草案的局部意见或历史批准都不等于批准实施。只有用户在审阅
计划和完整编码 Prompt 后明确回复“批准实施”，才可进入编码。

## 十三、完成报告要求

实施角色和汇总角色必须分别报告：

1. 实现或修改摘要；
2. 新增文件；
3. 修改文件；
4. 公共 API 变化；
5. 新依赖及用途；
6. 执行的验证命令；
7. 验证结果；
8. 未运行的检查及原因；
9. 已知限制；
10. 遗留问题；
11. 是否触及后续 milestone。

额外列出：

- 起点是 commit 还是脏工作树坐标，以及 ch2/ch3 的哪些改动被当作既有基线；
- 新增 Runtime 与旧 M2 证据链（`examples/`）的关系；
- `tool_result_message` 最终如何处理，以及理由；
- Fake Transport 如何证明默认测试离线；
- 哪些失败路径（未知工具/非法参数/路径越界/二次仍要工具/二次无文本）已有自动化
  证据；
- 是否运行真实 Provider 一步闭环；
- 是否存在未经批准的范围扩张；
- 是否产生本地 commit，以及该 commit 是否经过独立提交审核和归档记录。

## 十四、停止条件

出现以下任一情况立即停止，不得自行扩大范围：

- 用户尚未确认业务逻辑或模块划分；
- 当前 checkout 与任务坐标冲突，尤其是发现 ch2/ch3 与本 Prompt 描述的状态不一致；
- 目标路径存在未知用户内容；
- 计划需要修改候选范围外文件，尤其是 ch2/ch3 已验证的 `protocol.rs`/`transport.rs`；
- 需要新增依赖但用途、MSRV 或离线影响尚未批准；
- 默认测试需要真实网络、真实 API Key、费用，或访问 Workspace 之外的真实文件系统
  路径；
- Rust 实现必须提前引入 Agent Loop、预算、重试或多轮工具调用才能继续；
- `tool_result_message` 的处理方式无法在计划阶段达成一致；
- ch4 正文需要大规模风格重写而不是局部同步；
- 发现疑似密钥或敏感信息；
- 当前脏工作区无法用文件白名单与 ch4 变更安全分离；
- 验证失败原因不能区分为本任务缺陷、环境问题或既有基线问题；
- 需要 Git 提交、push、tag、发布或远程配置，但没有对应人工批准。

停止时应保留现场并报告事实、已发生变化、失败证据和恢复所需决定；不得 reset、
revert、覆盖用户文件或把计划外修复混入本轮。

## 十五、禁止事项

- 不把当前 Prompt 草案当作实施批准；
- 不让总控、任务分析或计划角色亲自编码；
- 不修改或删除旧 M0–M2 示例、P0 和 `crates/` 证据链；
- 不修改 ch2/ch3 已验证的 `protocol.rs`、`transport.rs`，以及 `config.rs`/`model_call.rs`/
  `openai_responses.rs` 中与 `tool_result_message` 无关的部分；
- 不提前实现 ch5 的 Agent Loop、Context 预算、Session、Recovery；
- 不实现 Policy、Sandbox、Validation、Evidence、Observability 或 Forge Studio 领域能力；
- 不引入除 `read` 以外的工具，即使 Python/Rust M2 示例已经实现；
- 不加入完整 Agent 框架、沙箱 SDK 或 Git dependency；
- 不在测试、fixture、文档或日志中保存真实凭据；
- 不让默认测试访问网络或工作区之外的真实文件系统路径；
- 不删除、跳过或弱化测试以获得绿色结果；
- 不使用 `git add .`；
- 不在未经明确批准时执行 `git add`、`git commit`、push、tag、发布、切换分支或修改
  remote；
- 不清理、覆盖或吸收用户已有无关修改；
- 不伪造 commit SHA、CI、真实 API、浏览器或发布结果；
- 不把 Prototype 写成生产级 Runtime 或生产级 sandbox。

## 十六、初始任务包

```yaml
protocol_version: "1"
task_id: "ch04-tool-runtime-001"
chapter: "ch04"
user_request: "把一次工具候选（ToolUseBlock）接到 Runtime，执行 read 并产生一步闭环观察，不引入 Agent Loop。"

project:
  root: "<开始时只读解析的仓库根目录>"
  start_state: "<开始时记录的 HEAD 与工作树状态；当前已知 ch2/ch3 在工作树完成但未提交>"
  target_state: "累计 Rust 工程新增 ToolCall/ToolResult/ToolRegistry/Workspace 与仅含 read 的一步闭环，并与 ch4 教学资产同步"
  previous_commit: null

platform:
  multi_agent_support: "supported | unsupported | uncertain"
  execution_mode: "automatic_multi_agent | manual_session_handoff"
  evidence: []
  limitations: []

model_assignment:
  recommended_profile: "sol"
  selected_model: "<由总控根据平台填写>"
  selection_reason: "ch4 起点涉及未提交的 ch2/ch3 基线、Runtime 边界设计和与 M1 的分层判断"
  complexity: "high"
  delegation_plan: []

assets:
  design:
    - "docs/decisions/reader-ai-coding-workflow-v2.md"
    - "docs/decisions/repository-baseline.md"
    - "docs/chapters/ch04/design.md"
    - "book/src/ch3.md"
    - "book/src/labs/m2-tool-runtime.md"
  architecture: []
  python:
    - "examples/python/m2-tool-runtime/tool_types.py"
    - "examples/python/m2-tool-runtime/registry.py"
    - "examples/python/m2-tool-runtime/workspace.py"
    - "examples/python/m2-tool-runtime/bridge.py"
    - "examples/python/m2-tool-runtime/one_step.py"
    - "examples/python/m2-tool-runtime/tools/read.py"
  python_prototype:
    status: "available"
    reason: "当前 Python M2 覆盖 ToolCall/ToolResult、Registry、Workspace、read 和一步闭环的离线失败路径"
    suggested_business_logic: "见本 Prompt 第四节"
    user_confirmation: "pending"

business_logic:
  confirmed_requirements: []
  disagreements: []
  risks:
    - "ch2/ch3 在工作树完成但未提交，起点不是不可变 commit"
    - "openai_responses.rs 中的 tool_result_message 与本章 bridge 职责重叠，处理方式未定"
    - "examples/rust/m2-tool-runtime 依赖独立的 examples/rust/m1-unified-protocol crate，不能被直接照搬进累计工程"
    - "旧 M2 七工具参考实现容易被误写为本轮新完成状态"
    - "真实网络结果不可作为默认确定性验收"

approval:
  approved_prompt: null
  approval_status: "pending"
  approval_evidence: null

scope:
  allowed_paths:
    - "只读分析当前仓库"
  forbidden_paths:
    - "当前阶段禁止修改任何工程文件"
  non_goals:
    - "Agent Loop（ch5/M3）"
    - "完整七工具集（ls/find/grep/write/edit/bash）"
    - "Policy、Sandbox、Validation、Evidence、Observability"
    - "会话、恢复、重试"

validation:
  required_commands:
    - "当前阶段只做只读坐标和资产核对"
  required_evidence:
    - "当前目录、Git 状态、相关决策、代码和测试事实"
  network_policy: "offline_by_default"
  credential_policy: "no_real_credentials"

workflow:
  current_state: "initialized"
  next_role: "task_analysis"
  requires_user_confirmation: true
```

## 十七、总控启动指令

请使用 `docs/prompts/workflow/v2/controller.md` 初始化任务，先检测当前平台能否可靠支持
隔离的多 Agent 工作流，并记录判断证据。然后只读核对本 Prompt 指定的工程坐标、决策、
正文、Python 原型、Rust 参考实现、测试和脏工作区边界——尤其要确认 ch2/ch3 在工作树
中的真实完成状态，以及 `tool_result_message` 的现状。

本轮只派发任务与需求分析角色。收到结构化分析结果后，停在
`awaiting_requirement_confirmation`，完整展示：

- 已确认事实；
- 仍是推断的内容；
- 需求冲突；
- 工程和安全风险；
- 本 Prompt 第十二节列出的七项用户决定；
- 是否已具备进入计划阶段的条件。

不要修改文件，不要运行实现测试，不要生成或执行编码方案，不要提交 Git。用户确认需求
分析后，才能派发计划角色生成精确文件白名单和完整编码 Prompt；计划仍需再次获得明确
的实施批准。
<!-- END DRAFT PROMPT -->

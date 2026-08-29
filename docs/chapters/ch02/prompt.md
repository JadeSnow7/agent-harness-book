# ch2 需求 Prompt 草案：第一次模型调用与累计 Rust 工程起点

- **章节：** ch2
- **Prompt 原文状态：** 草案原文（BEGIN/END 标记之间）保留不变，不根据最终代码反向改写；以下状态字段为回填更新（2026-08-21）
- **需求确认状态：** Confirmed（回填依据：`tutorial/agent-harness/` 已按本草案第五节的候选路径和职责分离落地，并通过 `cargo test -p tutorial-agent-harness --offline`）
- **实施批准状态：** Approved（回填依据同上）
- **实施状态：** Completed（依据：git tag `ch02` 指向 commit `5eeaf6a`「feat: add ch2+ch3 cumulative Rust tutorial project (M0 model call, M1 unified protocol)」，`book/src/ch2.md` 已标注"已实现并验证"）
- **上一章不可变坐标：** 无；开始任务时必须重新核对当前 Git 状态，不得猜测 commit SHA（此为草案撰写时的记录，不因回填而改写）
- **适用工作流：** `docs/prompts/workflow/v1/`
- **回填说明：** 本章实际实施发生在 `docs/chapters/`、`docs/workflow-runs/` 等工作流留痕资产建立之前，未保存独立的需求确认、计划批准和执行后审计记录文件。以上状态字段仅依据当前可验证的源码、测试和 git tag 回填，不代表 v1 协议要求的角色交接证据链在当时被完整保存；这与 ch05 起（`docs/chapters/ch05/prompt.md`）留有完整同步记录的情况不同。

这份文件保存 ch2 的需求 Prompt 草案。它用于启动任务与需求分析，而不是授权编码。
需求确认后，计划角色还必须生成精确文件白名单和完整编码 Prompt；只有用户明确回复
“批准实施”后，编码角色才可以修改工程。Prompt 一旦获批，不得根据最终代码反向改写；
实施、审计、提交和验收证据应保存到独立结果文件。

<!-- BEGIN DRAFT PROMPT -->
任务名称：
建立 ch2 累计 Rust 教学工程：完成一次可离线验证的模型调用

任务性质：
这是《从 0 搭建 AI Agent》逐章累计教学工程的第一个真实代码增量，对应 M0 Model
Call。任务需要先完成只读需求分析，再经过计划、实施、审计、汇总和人工门禁；当前
Prompt 只授权任务与需求分析，不授权修改代码、正文、配置或 Git 状态。

本章要建立一个后续章节持续演进的 Rust 工程，而不是新增另一个做完即丢弃的独立
片段。现有 Python/Rust M0–M2 和 P0 仍是仓库已有证据链，只能作为设计、行为和测试
参考，不能被描述成本轮新实现。

## 一、任务目标

需求分析应核对并向用户确认：ch2 是否以如下目标进入计划阶段。

1. 在仓库中建立独立的累计 Rust 教学工程，候选路径为
   `tutorial/agent-harness/`；后续 ch3–ch16 在同一工程上增加最小能力，而不是每章
   重建一个互不相干的示例。
2. 完成一次非流式模型请求的最小闭环：读取配置、构造 Provider 请求、通过 HTTP
   Transport 发送请求、检查状态码、解析 JSON、提取文本并向 CLI 调用者返回结果。
3. 默认验收完全离线，不读取真实 API Key，不访问真实 Provider，不产生调用费用。
4. 保留一个显式、手动触发的真实请求入口；缺少配置、依赖或网络时必须安全失败，
   不能 panic，也不能泄露 Authorization Header、API Key 或 Provider 错误正文。
5. 让读者理解这次调用只返回模型文本，不能读取工作区、不能执行工具、不能证明任务
   完成，也还不是 Agent。
6. 将 Rust 工程、Python 原型、ch2 正文、实现索引和真实验证证据同步起来。
7. 在不把 ch2 改写成 Shell/Git 教程的前提下，补充读者运行和保存本章工程坐标所需的
   最小命令行与 Git 基础。

## 二、本章结束时的可观察结果

若后续计划获得批准并完成实施，读者应能从仓库根目录完成以下动作：

1. 找到一个名称和路径稳定的累计 Rust 工程；
2. 在没有真实凭据、没有网络的环境中编译并运行全部默认测试；
3. 从测试中看到完整的一次调用边界，而不只是单独的 JSON 解析函数；
4. 看到 Fake Transport 记录请求并返回预置响应，理解测试为什么没有访问网络；
5. 在缺少 `OPENAI_API_KEY` 或 `OPENAI_MODEL` 时运行 CLI，得到非零退出码和安全错误；
6. 在自愿提供环境变量时手动发起一次真实的非流式请求；
7. 说出请求输入、响应输出、正常主流程、失败分类和本章有意留下的限制；
8. 通过 ch2 正文中的命令理解当前目录、命令参数、退出码、工作区、暂存区、commit
   与 push 的最小区别；
9. 明白文档构建、单元测试、真实网络请求和人工验收各自能证明什么，不能证明什么。

## 三、必须先核对的事实基线

任务与需求分析角色必须只读检查当前 checkout，不得直接相信本 Prompt 中可能随时间
变化的状态。至少核对：

- 当前工作目录和仓库根目录；
- `git status --short --branch`；
- 当前 HEAD、已跟踪修改、未跟踪文件和目标路径冲突；
- 根 `Cargo.toml`、`Cargo.lock`、Rust Edition、MSRV 和共享依赖；
- `AGENTS.md` 和任务范围内的已接受决策；
- `docs/decisions/reader-ai-coding-workflow-v1.md`；
- `docs/decisions/repository-baseline.md` 中的 M0、依赖、安全和验证边界；
- `book/src/ch2.md`、`book/src/implementations.md` 和阅读指南；
- `examples/python/m0-model-call/` 的业务逻辑和离线测试；
- `examples/rust/m0-model-call/` 的当前结构、能力和测试缺口；
- ch0 已保存的 Prompt、设计和工作流资产是否只是工作树坐标，还是已有不可变提交坐标。

已知但必须重新验证的基线假设：

- 当前仓库已有 Python/Rust M0–M2 和 Rust P0 证据链；
- 现有 Python M0 已将配置、请求构造、Transport 和解析分开，并通过 Fake Transport
  离线测试一次调用；
- 现有 Rust M0 可以编译和测试，但主要位于单个 `main.rs`，没有可注入 Transport，
  不应未经分析就直接成为累计教学工程；
- 当前根 workspace 已使用 Rust Edition 2024、MSRV 1.85、`reqwest` 和
  `serde_json`；
- 当前工作区可能包含用户未提交修改，任何实现都必须保护这些修改；
- 当前没有可作为 ch2 起点的、经确认的上一章 commit SHA。

如果实际 checkout 与上述假设不同，必须以当前事实为准，列出冲突并停止在需求确认
门禁；不得静默调整任务目标。

## 四、待用户确认的业务逻辑

以下内容来自当前 ch2 正文和 Python M0 原型，是需求草案，不是已经批准的实施依据。
任务与需求分析角色必须逐项核对，并将 `user_confirmation` 保持为 `pending`。

### 输入

- 一个普通 UTF-8 prompt 字符串；
- 环境配置：`OPENAI_API_KEY`、`OPENAI_MODEL`；
- 可选配置：`OPENAI_BASE_URL`、`OPENAI_TIMEOUT_S`；
- 一个真实或伪造的 HTTP Transport。

本章不接收统一 `Message`、工具定义、会话历史、结构化候选动作或 Agent 状态。

### 输出

- 成功时返回按 Provider 响应顺序连接的、非空 `output_text`；
- 失败时返回可分类、可安全展示的应用错误；
- CLI 成功时向标准输出写入文本并返回退出码 0；
- CLI 失败时向标准错误写入安全错误并返回非零退出码。

### 主流程

```text
读取并验证配置
→ 构造 { model, input } 请求
→ 构造 endpoint 与必要 Header
→ Transport 发起一次非流式 POST
→ 检查 HTTP 状态码
→ 解析 JSON 对象
→ 遍历 output/content
→ 提取并连接非空 output_text
→ 返回文本
```

### 失败分类

至少区分：

- 配置缺失或配置格式错误；
- Transport 无法完成请求，包括可控模拟的连接失败或超时；
- Provider 返回非 2xx 状态；
- Provider 返回非法 JSON 或非对象 JSON；
- 响应缺少 `output` 列表；
- 响应中不存在可用的 `output_text`。

错误信息不得包含：

- API Key；
- Authorization Header；
- Provider 错误响应正文；
- 与任务无关的环境变量；
- 测试中用于证明不泄露的 sentinel secret。

### 响应解析规则

- 不固定读取 `output[0]`；
- 忽略 reasoning 和当前示例不认识的输出项目；
- 只接受类型为 `output_text` 且内容非空的文本；
- 多个文本块按原响应顺序以换行连接；
- 只有拒答、reasoning 或未知项目而没有 `output_text` 时，返回明确的响应格式错误；
- 教学解析器可以忽略未知项目，但正文必须说明这不是生产级多轮上下文保留策略。

### 状态变化

本章不维护会话或 Agent 状态。单次调用只产生一个进程内请求和响应；真实网络请求在
超时后是否已被远端接收可能未知。本章不实现恢复、幂等、重放或持久化。

### 有意限制

- 只支持一次非流式调用；
- 只实现一个 Provider 形状；
- 不实现 streaming、retry、fallback、成本预算或速率限制策略；
- 不实现统一协议、工具声明、工具执行或 Agent Loop；
- 不让模型访问文件、Shell、网络工具或其他环境能力；
- 不把 `200 OK` 或模型文本当成任务完成证据；
- 不声称 Transport 抽象、workspace 路径或普通进程边界等同于 OS sandbox。

## 五、累计 Rust 工程的候选边界

以下是进入需求确认的候选方案，不是已批准的文件级设计。

### 工程位置

推荐：

```text
tutorial/agent-harness/
```

推荐理由：

- 与现有 `examples/rust/m0-model-call/` 证据链清楚分离；
- 路径不绑定 M0，便于 ch3–ch16 在同一工程继续演进；
- 可作为根 Cargo workspace 的明确成员，由根验证命令覆盖；
- 读者可以把它识别为自己的累计工程，而不是一次性示例。

任务分析必须请用户确认：

1. 是否采用 `tutorial/agent-harness/`；
2. 是否将它加入根 Cargo workspace；
3. 是否接受它从 ch2 起持续演进，而现有 `examples/rust/m0-model-call/` 保持不动；
4. 如果选择其他路径，是否仍能避免与旧 M0–M2 证据链混淆。

### 最小职责分离

候选实现应让读者能看见以下职责，但计划角色应避免为追求目录数量而机械拆文件：

- CLI 入口：选择 prompt、调用一次、打印结果并映射退出码；
- Config：从环境或显式映射加载和验证配置；
- Request builder：纯函数生成当前 Provider 的请求 JSON；
- HTTP Transport：真实请求实现和 Fake Transport 共用的最小测试缝；
- Response parser：状态检查、JSON 解析和文本提取；
- AppError：表达本章确实需要区分的错误类别。

`HttpTransport` 只用于隔离真实网络和离线测试。它不能提前演变为 ch3 的 Provider-neutral
`ModelProvider`，也不能引入 `Message`、`ContentBlock`、`ToolDefinition` 或未来 Loop
接口。公共 API 必须保持最小；仅为测试而需要的接口应评估能否保持 crate 内可见。

## 六、Python 原型的角色

业务参考：

- `examples/python/m0-model-call/chat_once.py`
- `examples/python/m0-model-call/test_chat_once.py`

Python 原型提供：

- 配置、请求、Transport 和解析的责任分离；
- `build_request` 纯函数；
- Fake Transport 离线测试；
- Responses API 文本提取规则；
- 安全错误和真实请求的手动边界。

Rust 实现不得机械逐行翻译 Python。计划必须说明 Rust 在所有权、错误类型、trait 可见性、
依赖和 CLI 退出码方面的选择。Python 原型不提供 Agent Loop、工具执行、会话、重试或
生产级 Provider 兼容性。

## 七、依赖与公共 API 候选约束

需求分析和计划阶段应优先验证能否只复用 workspace 已有依赖：

- `reqwest`：手动真实请求的阻塞 HTTP 客户端；
- `serde_json`：构造和解析当前 Provider JSON。

默认不为 Fake Transport 引入 mock server、异步 runtime、完整 Agent SDK 或测试框架。
如计划认为必须新增依赖，必须说明当前 ch2 的直接用途、离线缓存和 MSRV 影响，并单独
请求用户确认。共享依赖应遵守根 `[workspace.dependencies]`，不得使用 Git dependency。

本章不承诺稳定公共 API。可以提供供 CLI 和集成测试使用的最小 library surface，但不能
为了未来章节预留空 trait、占位方法、兼容层或未经验证的扩展点。

## 八、默认离线测试契约

后续计划至少应覆盖以下正常、错误和边界行为：

1. 缺少 API Key 时在 Transport 调用前失败；
2. 缺少模型名时在 Transport 调用前失败；
3. Base URL 为空、超时为非数字、非有限值、零或负数时失败；
4. endpoint 能正确处理 Base URL 末尾斜杠；
5. 请求 JSON 只包含当前章需要的 `model` 和 `input`；
6. Fake Transport 能记录 URL、Header、payload 和 timeout，并证明完整调用经过该边界；
7. Fake Transport 可以注入 Transport 失败；
8. HTTP 401、429、500 返回状态错误，且不回显响应正文；
9. 非法 JSON、非对象 JSON和缺少 `output` 分别明确失败；
10. reasoning 出现在文本之前时仍可提取文本；
11. 多个 `output_text` 按顺序连接；
12. 空文本、只有拒答或只有未知项目时不伪造答案；
13. 错误显示文本不包含测试 sentinel secret；
14. CLI 缺配置时返回非零退出码且不 panic；
15. 默认测试在没有真实环境变量和网络的条件下确定性通过。

测试不得通过删除、跳过或降低断言获得绿色结果；不得使用真实 API Key、真实 Provider、
`sleep` 或不可控 timing race。

## 九、ch2 正文与基础知识同步

如果需求和计划随后获得批准，`book/src/ch2.md` 应在保留作者声音和 Python-first 教学
顺序的前提下，与新的累计 Rust 工程同步。正文修改应局部完成，不得整章改写为 Rust
Reference 或产品说明书。

正文至少应让初学者理解：

- 当前目录决定相对路径从哪里解析；
- Shell 命令由程序名、参数和环境变量组成；
- 退出码 0 通常表示成功，非 0 表示程序报告失败；
- `cargo test` 编译并执行测试，绿色结果不等于真实 Provider 一定可用；
- Git 工作区保存当前文件变化，暂存区选择下一次提交内容；
- commit 是本地不可变坐标，push 是把本地提交发送到远端；
- 本章没有真实提交时，不能把脏工作区或当前 HEAD 伪装成章节验收坐标。

这些说明只需支撑读者运行、检查和保存 ch2 工程，不展开 Shell 语法大全、Git 分支策略、
网络协议栈或 TLS 教程。

正文和实现索引必须明确区分：

- 旧 Python/Rust M0–M2：已有并验证的参考证据；
- 新累计 Rust 工程 ch2 增量：只有代码、测试、文档和审计实际完成后才能标记；
- P0：确定性组合参考，不等于新工程已经完成 M3–M10；
- 真实网络：手动、可选、未执行时必须如实报告。

## 十、计划阶段的候选文件范围

以下范围只用于任务分析和计划，不构成当前修改授权。计划角色必须根据确认后的工程路径
给出精确到文件的白名单。

候选新增：

- `tutorial/agent-harness/Cargo.toml`
- `tutorial/agent-harness/src/lib.rs`
- `tutorial/agent-harness/src/main.rs`
- `tutorial/agent-harness/src/` 下经计划证明必要的 M0 模块
- `tutorial/agent-harness/tests/` 下的离线集成测试
- `docs/chapters/ch02/design.md`
- `docs/chapters/ch02/` 下独立保存的任务包、实施结果、审计或完成报告

候选修改：

- `Cargo.toml`，仅用于加入累计教学工程和复用必要 workspace dependency；
- `Cargo.lock`，仅允许由已批准的 Cargo 操作产生必要变化；
- `book/src/ch2.md`，仅同步真实实现、命令、Shell/Git 最小知识和限制；
- `book/src/implementations.md`，仅增加经验证的新累计工程入口并保留旧实现状态。

默认禁止修改：

- `examples/python/m0-model-call/`；
- `examples/python/m1-unified-protocol/`；
- `examples/python/m2-tool-runtime/`；
- `examples/rust/m0-model-call/`；
- `examples/rust/m1-unified-protocol/`；
- `examples/rust/m2-tool-runtime/`；
- `examples/rust/p0-demo/`；
- `crates/`；
- `fixtures/`；
- `book/src/ch0.md`、`book/src/ch1.md` 和 ch3–ch16；
- `.github/`、remote、branch、tag 和发布配置；
- 用户已有、与 ch2 无关的未提交文件。

已批准的 `docs/chapters/ch02/prompt.md` 原文不属于编码角色的修改范围。若需求审阅要求
调整本草案，应在批准前完成；批准后产生的新争议必须创建版本化修订 Prompt，不能反向
改写原文。

## 十一、建议验证与证据

计划角色应根据最终包名和路径给出可直接运行的命令。默认至少考虑：

```text
cargo fmt --all -- --check
cargo check -p <confirmed-package-name> --offline
cargo test -p <confirmed-package-name> --offline
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
python3.11 -m unittest discover -s examples/python/m0-model-call -p 'test_*.py'
mdbook build book --dest-dir <temporary-output-directory>
git diff --check
git status --short
```

还应验证：

- 在显式移除相关环境变量的进程环境中运行 CLI，确认安全失败和非零退出码；
- Fake Transport 证明默认测试没有访问网络；
- 实际变更文件严格等于批准白名单；
- mdBook 生成的 ch2 页面存在，include 和内部链接没有错误；
- 如本机安装 Rust 1.85，执行 MSRV 检查；未安装时如实报告，不得声称通过；
- 不把未运行的真实 Provider 请求写成成功；
- 不把根 workspace 绿色结果写成 GitHub Actions 已通过。

mdBook 必须构建到临时目录，不能用构建产物污染工作区。验证如因离线依赖缓存、工具缺失
或现有无关修改失败，必须区分本任务缺陷与环境/基线问题，不得扩大范围修复。

## 十二、人工门禁与角色顺序

必须遵循 `docs/prompts/workflow/v1/`：

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
→ 用户最终验收
```

当前只允许执行第一步。任务分析完成后必须停在
`awaiting_requirement_confirmation`，向用户展示事实、假设、争议、风险和待确认决定。

至少要求用户确认：

1. 是否采用 `tutorial/agent-harness/` 作为累计工程路径；
2. 是否把它加入根 Cargo workspace；
3. 是否确认第四节的输入、输出、主流程、错误规则和有意限制；
4. 是否确认默认验收只使用 Fake Transport，真实网络调用不作为通过条件；
5. 是否确认旧 M0–M2/P0 只作参考且不在本轮修改；
6. 是否确认 ch2 正文需要同步 Rust 工程和 Shell/Git 最小基础；
7. 是否接受本章不引入 ch3 的统一协议和任何 Agent Loop 能力。

“开始拟写 Prompt”、对本草案的局部意见或历史批准都不等于批准实施。只有用户在审阅计划
和完整编码 Prompt 后明确回复“批准实施”，才可进入编码。

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

- 起点是 commit 还是脏工作树坐标；
- 新累计工程与旧 M0 证据链的关系；
- Fake Transport 如何证明默认测试离线；
- 哪些错误路径已有自动化证据；
- 是否运行真实 Provider 请求；
- Shell/Git 基础在正文中的范围；
- 是否存在未经批准的范围扩张；
- 是否产生本地 commit，以及该 commit 是否经过独立提交审核。

## 十四、停止条件

出现以下任一情况立即停止，不得自行扩大范围：

- 用户尚未确认业务逻辑或累计工程路径；
- 当前 checkout 与任务坐标冲突；
- 目标路径存在未知用户内容；
- 计划需要修改候选范围外文件；
- 需要新增依赖但用途、MSRV 或离线影响尚未批准；
- 默认测试需要真实网络、真实 API Key 或费用；
- Rust 实现必须提前引入 M1 统一协议、工具或 Agent Loop 才能继续；
- ch2 正文需要大规模风格重写而不是局部同步；
- 发现疑似密钥或敏感信息；
- 当前脏工作区无法用文件白名单与 ch2 变更安全分离；
- 验证失败原因不能区分为本任务缺陷、环境问题或既有基线问题；
- 需要 Git 提交、push、tag、发布或远程配置，但没有对应人工批准。

停止时应保留现场并报告事实、已发生变化、失败证据和恢复所需决定；不得 reset、revert、
覆盖用户文件或把计划外修复混入本轮。

## 十五、禁止事项

- 不把当前 Prompt 草案当作实施批准；
- 不让总控、任务分析或计划角色亲自编码；
- 不修改或删除旧 M0–M2、P0 和 `crates/` 证据链；
- 不提前实现 ch3 的统一协议；
- 不实现工具、Workspace、Agent Loop、Context、Session、Policy、Sandbox、Validation、
  Evidence、Observability 或 Forge Studio 领域能力；
- 不加入完整 Agent 框架、Provider SDK 或 Git dependency；
- 不在测试、fixture、文档或日志中保存真实凭据；
- 不让默认测试访问网络；
- 不删除、跳过或弱化测试以获得绿色结果；
- 不使用 `git add .`；
- 不在未经明确批准时执行 `git add`、`git commit`、push、tag、发布、切换分支或修改
  remote；
- 不清理、覆盖或吸收用户已有无关修改；
- 不伪造 commit SHA、CI、真实 API、浏览器或发布结果；
- 不把 Prototype 写成生产级 Harness。

## 十六、初始任务包

```yaml
protocol_version: "1"
task_id: "ch02-model-call-001"
chapter: "ch02"
user_request: "为 ch2 建立累计 Rust 教学工程，完成一次可离线验证的模型调用。"

project:
  root: "<开始时只读解析的仓库根目录>"
  start_state: "<开始时记录的 HEAD 与工作树状态>"
  target_state: "累计 Rust 工程完成 M0 单次非流式模型调用并与 ch2 教学资产同步"
  previous_commit: null

platform:
  multi_agent_support: "supported | unsupported | uncertain"
  execution_mode: "automatic_multi_agent | manual_session_handoff"
  evidence: []
  limitations: []

model_assignment:
  recommended_profile: "sol"
  selected_model: "<由总控根据平台填写>"
  selection_reason: "ch2 起点涉及业务边界、工程路径、脏工作区和后续章节演进判断"
  complexity: "high"
  delegation_plan: []

assets:
  design:
    - "docs/decisions/reader-ai-coding-workflow-v1.md"
    - "docs/decisions/repository-baseline.md"
    - "book/src/ch2.md"
  architecture: []
  python:
    - "examples/python/m0-model-call/chat_once.py"
    - "examples/python/m0-model-call/test_chat_once.py"
  python_prototype:
    status: "available"
    reason: "当前 Python M0 覆盖配置、请求、Transport、解析和离线失败路径"
    suggested_business_logic: "见本 Prompt 第四节"
    user_confirmation: "pending"

business_logic:
  confirmed_requirements: []
  disagreements: []
  risks:
    - "候选累计工程路径尚未确认"
    - "当前工作树可能包含用户未提交修改"
    - "旧 M0 参考实现容易被误写为本轮新完成状态"
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
    - "统一协议"
    - "工具调用或工具执行"
    - "Agent Loop"
    - "M1 及后续 milestone"

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

请使用 `docs/prompts/workflow/v1/controller.md` 初始化任务，先检测当前平台能否可靠支持隔离
的多 Agent 工作流，并记录判断证据。然后只读核对本 Prompt 指定的工程坐标、决策、正文、
Python 原型、Rust 参考实现、测试和脏工作区边界。

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

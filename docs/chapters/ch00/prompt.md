# ch0 初次实施 Prompt

- **章节：** ch0
- **Prompt 原文状态：** 完整保存
- **审批证据状态：** 审批证据未入库；当前文件保存的是实施前拟定的完整 Prompt 原文。
- **实施状态：** 已收到完成报告
- **验收状态：** Changes requested

<!-- BEGIN APPROVED PROMPT -->
任务名称：
重新组织 ch0：建立全书架构地图与 AI Coding 工作流

任务性质：
这是从 ch0 开始的逐章重新组织。代码优先原则从 ch2 起进入真实实现；
ch0 只交付架构、工作流程和章节设计资产，不新增 Python/Rust 运行时代码。

一、目标

重组《从 0 搭建 AI Agent》的 ch0，使读者在进入具体代码前能够回答：

1. Model、Harness、Environment 分别处于什么位置；
2. 一个 Agent 系统如何在模型、Harness 和真实环境之间形成反馈闭环；
3. 本书将怎样从 ch0 逐章演进为同一个完整工程；
4. 作者和 AI 如何通过“Prompt → 审批 → 实现 → 验证 → Review”协作；
5. 为什么模型输出不是环境事实，测试、状态和证据才是验收依据。

ch0 不详细定义 Model/Harness 的责任边界，该内容留给 ch1。
ch0 不实现模型调用、网络 Transport 或任何 Agent Runtime，该内容从 ch2 开始。

二、必须保留的现有证据链

以下内容只作为参考证据，不得删除、迁移、重命名或改写：

- examples/python/m0-model-call/
- examples/python/m1-unified-protocol/
- examples/python/m2-tool-runtime/
- examples/rust/m0-model-call/
- examples/rust/m1-unified-protocol/
- examples/rust/m2-tool-runtime/
- examples/rust/p0-demo/
- crates/
- fixtures/
- docs/adr/
- docs/specs/
- docs/architecture/p0-component-boundaries.md

不得把现有 M0–M2 或 P0 代码描述为本轮新实现。
不得把 P0 参考切片描述为 M3–M10 已经完成。

三、允许修改或新增的文件

允许修改：

- book/src/ch0.md
- OUTLINE.md，仅在同步 ch0 Chapter Contract 确有必要时修改

允许新增：

- docs/chapters/ch00/prompt.md
- docs/chapters/ch00/design.md
- book/src/assets/ch00/agent-harness-map.mmd
- book/src/assets/ch00/ai-coding-workflow.mmd

除上述文件外，不得修改其他文件。
如果实际构建要求额外修改，先停止并报告，不得自行扩大范围。

四、章节资产

1. 保存 Prompt

将用户批准后的本 Prompt 保存到：

docs/chapters/ch00/prompt.md

需要注明：

- 状态：Approved；
- 章节：ch0；
- 本轮允许范围；
- 本轮禁止事项；
- 验收命令；
- 不包含实现结果，避免事后改写 Prompt。

2. 建立设计方案

新增：

docs/chapters/ch00/design.md

内容包括：

- 本章要解决的问题；
- 读者进入本章时具备的知识；
- 本章交付的认知增量；
- 两张架构图各自承担的认知任务；
- 本章不新增运行时代码的理由；
- 与 ch1、ch2 的边界；
- 当前仓库事实状态；
- 验收标准；
- 已知限制。

不要把设计文档写成未来所有章节的完整架构规范。

3. Agent Harness 总览图

新增 Mermaid 源文件：

book/src/assets/ch00/agent-harness-map.mmd

图中至少包含：

- User Goal；
- Harness；
- Model；
- Environment；
- Context；
- Policy；
- Runtime；
- State / Events；
- Validation / Evidence；
- Human Review / Stop。

必须清楚表达：

- Harness 向 Model 提供输入；
- Model 只提出文本或候选动作；
- Policy 决定动作是否允许进入 Runtime；
- Runtime 才能接触 Environment；
- Environment 返回 Observation；
- Validation 根据环境状态形成 Evidence；
- 人根据目标、证据、风险和预算决定继续或停止。

图中需要区分：

- 逻辑目标架构；
- 当前仓库已经实现的 M0–M2；
- 只作为组合参考的 P0；
- 尚未逐章实现的 M3–M10。

不得通过颜色或措辞暗示所有组件已经生产可用。

4. AI Coding 工作流程图

新增 Mermaid 源文件：

book/src/assets/ch00/ai-coding-workflow.mmd

图中至少包含：

- 只读检查当前仓库；
- 定义本章目标与验收条件；
- AI 起草编码 Prompt；
- 人工审阅 Prompt；
- 未批准时修改 Prompt；
- 批准后实施最小代码/文档增量；
- 运行测试、lint、类型检查或文档构建；
- 检查 diff、退出码和环境状态；
- 生成证据与完成报告；
- 人工决定接受、修正、回退或进入下一章。

必须明确画出两个人工门禁：

1. 实施前批准 Prompt；
2. 实施后审阅代码、验证证据和章节结果。

AI 的“已经完成”不得直接连接到“任务完成”。

五、重组 ch0 正文

修改 book/src/ch0.md，但保留作者已有的个人声音和有效内容。

推荐叙事顺序：

1. 为什么模型不等于 Agent；
2. Model、Harness、Environment 的总览；
3. 第一张架构图；
4. 为什么本书从 Harness 入手；
5. 同一个工程怎样逐章获得能力；
6. 我们怎样使用 AI Coding 推进这本书；
7. 第二张工作流程图；
8. 必要的计算机基础补充；
9. 当前仓库起点；
10. ch0 的限制与 ch1 要回答的问题。

不得把正文改写成产品说明书、术语词典或抽象架构 Reference。
避免连续罗列概念；每个抽象都要回答它解决了什么具体问题。

六、基础知识说明

不能假设读者已经熟悉网络、操作系统或数据结构。

只在首次需要时加入简短说明，每项控制在能够支持当前论证的范围：

- 网络：模型 API 本质上是客户端向远程服务发送请求并接收响应；
- OS：文件、进程、网络和权限由操作系统及运行时控制，模型本身没有这些权限；
- 数据结构：
  - Input 是交给模型的数据；
  - Candidate Action 是模型建议执行的动作；
  - Observation 是环境执行后返回的事实；
  - Event 是系统保存的不可混淆的过程记录；
  - Evidence 是支持验收结论的可复查材料。

不要在 ch0 展开 HTTP、进程模型、序列化或事件溯源的完整教程。
需要深入讲解的知识应指出将在后续哪一章展开。

七、代码与注释规则

本章不新增真实 Python/Rust 代码，也不要为了展示注释而创建没有用途的占位函数。

从后续真实代码章节开始，统一遵守：

- 注释解释设计原因、边界、不变量和不直观的失败路径；
- 不逐行翻译代码；
- 不给显而易见的赋值和控制流添加噪声注释；
- 涉及网络、OS、并发、数据结构或安全假设时，添加恰到好处的解释；
- 注释必须与当前行为一致；
- 关键代码应让初学者能够理解“输入是什么、输出是什么、失败会怎样”。

八、状态表达

ch0 完成后只能声明：

- ch0 架构与工作流程资产已建立；
- mdBook 构建已验证；
- 没有新增 Agent Runtime 能力；
- ch1 尚未实现 Model/Harness 边界伪代码；
- ch2 及之后的逐章真实实现尚未开始。

不得声明：

- Harness 已完整实现；
- M3–M10 已完成；
- 已具备生产级 Sandbox、Recovery、Validation 或 Observability；
- 文档构建通过等同于运行时代码正确。

九、验证

至少执行：

1. 检查实际变更文件是否严格位于允许范围；
2. 检查两份 Mermaid 源文件已被 ch0 正确引用；
3. 将 mdBook 构建到临时目录；
4. 检查生成的 ch0 页面存在；
5. 检查内部链接和 include 没有报错；
6. git diff --check；
7. git status --short；
8. 人工检查两张图是否分别承担“系统架构”和“AI Coding 流程”的认知任务。

本轮没有运行时代码变化，因此不要求用测试结果冒充代码验证。
如果现有环境缺少 mdBook，应如实报告，不能声称构建通过。

十、完成报告

完成后必须报告：

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
11. 是否触及 ch1、ch2 或后续 milestone。

额外列出：

- 两张图分别解决什么认知问题；
- 哪些内容来自现有证据链；
- 哪些只是路线说明；
- 是否存在未经批准的范围扩张。

十一、禁止事项

- 不修改任何 Python/Rust 源码；
- 不新增依赖；
- 不运行真实模型 API；
- 不访问真实网络；
- 不提交、推送、切换分支或创建标签；
- 不处理未跟踪的 docs/report.md；
- 不提前实现 ch1；
- 不提前实现 ch2；
- 不把旧 P0 拆分后冒充逐章新实现；
- 不覆盖作者已有但与本任务无关的内容。
<!-- END APPROVED PROMPT -->

```yaml
protocol_version: "1"
task_id: "workflow-v2-artifact-recorder-001"
role: "task_analysis"
status: "needs_user_input"

understanding:
  user_goal: >-
    建立并独立验收一个 breaking-change workflow/v2，使原本只读的
    task-analysis、planning、post-implementation-audit、summary、
    commit-audit 等角色输出能够成为可追溯的仓库长期资产；v2 验收前继续暂停 ch3。
  observable_result: >-
    v1 原样保留；v2 明确定义受控 Artifact Recorder 的输入、权限、完整性验证、
    失败行为、资产生命周期和人工门禁。Recorder 只能逐字保存获准的上游原文，
    不能反向修改角色输出、业务文件、测试或 Git 状态。
  requested_method: >-
    采用用户已选择的 B 路线：先按 v1 门禁设计和批准 workflow/v2，再恢复 ch3；
    当前仅完成 v2 的只读需求分析，不授权计划、实施、落盘或提交。

coordinates:
  project_root: "/Users/huaodong/Documents/ChatGPT/agent-harness-book"
  start_state: >-
    HEAD 355a7d0d240e68f9e7ffc336e42913b609f5ac5e；分支 main，
    相对 origin/main ahead 4；工作树包含既有已修改和未跟踪内容；
    docs/decisions/reader-ai-coding-workflow-v1.md、
    docs/prompts/workflow/v1/、docs/chapters/ch00/、ch02/、ch03/
    均未被当前 Git 索引跟踪
  start_state_verified: true
  target_state: >-
    经独立门禁批准的 workflow/v2：新增受控 Artifact Recorder，
    使只读角色结果可逐字、可校验、不可反向改写地保存为长期资产，
    同时不授权 Recorder 修改业务文件、执行实现测试或进行 Git 操作

findings:
  facts:
    - >-
      reader-ai-coding-workflow-v1.md 将 v1 标为 Accepted，并明确规定：
      破坏角色权限、人工门禁、状态语义或任务包必填字段的变更必须创建新主版本目录。
    - >-
      v1 按顺序定义七种逻辑角色；总控、任务分析、计划、执行后审计、汇总和提交审核
      均为只读，编码角色只能修改已批准路径，提交角色不编辑业务文件。
    - >-
      v1 当前没有合法负责“把只读角色输出写入仓库”的角色、状态或交接字段。
      在 v1 内直接赋予任一现有只读角色写权限会破坏现有权限契约。
    - >-
      新增 Recorder 会改变角色集合、权限矩阵、状态机、任务包和结果字段，
      因而属于 v2 breaking change，不能覆盖或静默修订 v1。
    - >-
      用户上一门禁明确回复“B”，只批准进入 workflow/v2 的需求分析；
      没有批准计划、文件修改、测试、Git 提交或 ch3 继续实施。
    - >-
      既有 ch00 资产采用 prompt.md 与独立 revision Prompt 分离的方式；
      初次 Prompt 明示“审批证据未入库”，修订 Prompt 单独保存状态。
      ch02 保存设计、完整 Draft Prompt 和初始任务包，但没有通用角色结果目录。
    - >-
      当前 ch03 只有 design.md，没有经本轮工作流生成和批准的 Prompt 或角色结果资产。
    - >-
      v1 decision、完整 workflow/v1 目录以及 ch00/ch02/ch03 章节资产当前均未跟踪，
      因而没有可由 Git 验证的不可变基线；本轮不得把文件内的状态文字冒充已提交证据。
    - >-
      若把原文包进 Markdown 围栏、YAML 字段或 JSON 字符串，转义、缩进和换行可能改变字节；
      因此“逐字原文”和“附带元数据”需要可分别验证的存储边界。
    - >-
      本轮只运行了 pwd、Git 状态/HEAD/跟踪状态检查、文件枚举、只读内容检查和 SHA-256
      读取；没有修改文件，也没有运行实现测试。
  assumptions:
    - >-
      “逐字”至少要求保存载荷与 Recorder 接收的规范化输入在 UTF-8 字节层面一致，
      但规范化边界、换行规则和平台消息元数据是否属于原文尚未确认。
    - >-
      “所有长期角色资产”可能不仅包含只读角色结果，还可能包含用户门禁原文、
      获批 Prompt、实施结果、提交结果和控制器状态迁移；具体资产清单尚未确认。
    - >-
      Recorder 失败应阻止工作流越过对应状态，而不是允许稍后补写；该 fail-closed
      语义仍需用户确认。
    - >-
      Artifact Recorder 只负责机械记录，不应成为新的技术审计者或内容批准者。
  unclear_points:
    - >-
      原文捕获边界不明确：只保存角色结构化 YAML，还是保存“结构化 YAML + 自然语言全文”；
      平台包装、消息 ID、时间戳和 Markdown 渲染后的内容是否属于原文。
    - >-
      长期资产清单不明确：是否覆盖 task-analysis、planning、approved prompt、
      implementation、audit、summary、commit、commit-audit、用户审批证据和 controller
      状态快照的全部或子集。
    - >-
      尚未选择集中式任务目录或章节邻接目录，也未确定 task_id、role、attempt/revision
      的文件命名规则。
    - >-
      尚未选择原文文件格式及清单格式：Markdown、YAML、JSONL 或“原始载荷文件 +
      独立机器可读 manifest”。
    - >-
      尚未决定 Recorder 写入后的完整性复核由独立只读 Artifact Audit 承担，
      还是扩展 controller 的只读校验职责。
    - >-
      尚未确定修订策略：禁止覆盖并创建 attempt/revision，还是允许受控追加到单一日志。
    - >-
      尚未确认哈希算法、字节长度、输入/输出哈希、任务坐标及状态匹配字段是否都必须进入
      manifest。
    - >-
      尚未明确自动多 Agent 与 manual_session_handoff 两种模式下，Recorder 如何获得
      相同的受控写权限和原文输入。
  disagreements:
    - >-
      “所有长期资产必须入库”与 v1 多个角色必须只读之间存在真实权限冲突；
      用户已选择通过新主版本解决，但 v2 的具体角色和状态语义尚未确认。
    - >-
      “逐字保存”与“发现敏感信息后自动脱敏”不可同时满足；Recorder 若修改或删减原文，
      就不再是字节一致的记录者。
    - >-
      “目标文件 append-only”与“对既有结果原地修正”不可同时满足；若强调不可反向改写，
      修订应产生新 attempt/revision，而不是覆盖历史。
    - >-
      若 controller 自行判断内容正确性，会扩大其只读编排职责；若新增 Artifact Audit，
      则会增加角色数和状态机节点。两者都属于必须显式批准的 v2 契约变化。
  risks:
    - >-
      当前 v1 和 decision 本身未跟踪；若直接创建 v2，难以用 Git 证明 v1 被原样保留，
      必须在后续计划中用起始哈希和显式白名单保护既有内容。
    - >-
      Bootstrap 存在循环依赖：v2 Recorder 创建前，v2 自身的 task-analysis、planning、
      approved prompt 和 implementation 资产无法由 Recorder 写入。
    - >-
      若让 implementation coder“总结”前置角色结果，会伪造来源；bootstrap 最多只能在
      获批白名单内逐字转录已冻结原文与哈希，并明确标记其记录者不是 Recorder。
    - >-
      若输入含 API Key、Authorization Header、.env 内容或无关个人信息，既不能原样入库，
      也不能由 Recorder 自行脱敏；缺少 fail-closed 规则会造成泄密或证据失真。
    - >-
      任意目标路径或可覆盖路径会使 Recorder 退化成通用写文件能力，可能修改业务源码、
      既有 Prompt 或 v1 资产。
    - >-
      若使用“先覆盖再校验”，失败可能破坏既有资产；需求应要求目标不存在或受控追加、
      原子发布，并在失败时保留原目标不变。
    - >-
      若 Recorder 可预写未来角色结果或未来门禁证据，会破坏时间顺序和审批真实性。
    - >-
      若记录失败不阻断状态机，后续仍会出现无法追溯的流程空洞。
  decision_conflicts:
    - >-
      已接受 v1 要求 breaking change 新建主版本；因此 v2 必须与
      docs/prompts/workflow/v1/ 并存，不能覆盖 v1。
    - >-
      AGENTS.md 要求保护未知未提交内容；v1、decision 和章节资产均为未跟踪用户内容，
      后续计划必须采用精确路径和哈希边界，不得把整个 docs/ 视为可重写范围。
    - >-
      v1 禁止已批准 Prompt 被最终代码反向改写；Recorder 必须只接受已经冻结的原文，
      不能根据实现或审计结果修正上游文本。
    - >-
      v1 要求审批、实现、审计、汇总和提交分离；Recorder 不能兼任审批、技术审计、
      实现或 Git 提交。

python_prototype:
  status: "not_applicable"
  references: []
  suggested_business_logic:
    input:
      - "上游角色完整原文的冻结字节载荷"
      - "输入内容哈希和字节长度"
      - "task_id、role、status、attempt/revision 等任务坐标"
      - "由当前门禁批准的唯一目标路径白名单"
    output:
      - "原文载荷的不可变长期资产"
      - "独立 manifest 中的输入哈希、落盘哈希、字节长度和任务坐标"
      - "成功或 fail-closed 的结构化 Recorder 结果"
    main_flow:
      - "验证任务坐标、角色、状态和目标路径"
      - "确认目标不存在或符合获批 append-only 规则"
      - "执行敏感信息和路径白名单检查"
      - "以不改变载荷字节的方式写入临时目标"
      - "计算落盘哈希并验证与输入哈希及字节长度一致"
      - "原子发布原文和 manifest；失败时不改变已有目标"
      - "禁止预写尚未实际产生的未来阶段资产"
    error_paths:
      - "哈希、长度、task_id、role 或 status 不匹配时拒绝写入"
      - "目标已存在且不允许追加时拒绝写入"
      - "路径越界、敏感信息、非法格式或非原子发布失败时阻断工作流"
      - "Recorder 不得自行修正、补全、总结、脱敏或重试为不同内容"
    intentional_limits:
      - "不修改业务源码、测试、书籍正文、Cargo、examples、crates 或 ch3 资产"
      - "不执行代码、实现测试、Git、网络、发布或远程操作"
      - "不判断上游内容的技术正确性"
      - "不把记录成功等同于审计通过或用户验收"
  user_confirmation: "pending"

readiness:
  ready_for_planning: false
  missing_information:
    - "v2 的完整长期资产清单"
    - "Artifact Audit 与 controller 校验的职责选择"
    - "资产拓扑、命名、原文格式和 manifest 格式"
    - "原文字节的规范化边界"
    - "覆盖、append-only、attempt/revision 策略"
    - "敏感信息处理与 Recorder 失败后的状态机行为"
    - "v2 bootstrap 前置资产的合法落盘方式"
    - "手动交接模式下 Recorder 的等价权限模型"
  required_user_decisions:
    - >-
      确认 v2 必须保留 v1 原样并创建独立主版本资产；不得覆盖
      docs/prompts/workflow/v1/ 或把 v1 状态反向改写。
    - >-
      确认 Recorder 的绝对权限边界：只接收冻结原文、哈希和任务坐标，并逐字写入
      获批路径；不能总结、修正、补全、脱敏、执行代码/测试/Git、修改业务文件或预写未来结果。
    - >-
      在“独立只读 Artifact Audit”与“扩展 controller 做哈希/字节校验”之间选择；
      前者增加角色与状态节点，后者扩大 controller 职责。
    - >-
      明确需要长期保存的资产集合，尤其是否包含用户门禁原文、controller 状态、
      implementation/commit 结果以及只读角色自然语言全文。
    - >-
      选择资产拓扑与命名：集中按 task_id 保存，或按章节/主题邻接保存；
      同时定义 role、sequence、attempt/revision 和未来修订的唯一命名规则。
    - >-
      选择原文与元数据格式；建议将原始 UTF-8 载荷单独保存，把 SHA-256、字节长度、
      task_id、role、status、来源和目标写入独立机器可读 manifest，避免包装改变原文字节。
    - >-
      确认不可变策略：目标默认必须不存在；纠错创建新的 attempt/revision；
      禁止覆盖历史资产。若需要日志追加，必须定义严格的 append-only 边界。
    - >-
      确认安全策略：敏感信息检查失败时 Recorder 必须拒绝落盘且不得自动脱敏；
      由上游通过新的受审角色结果产生安全的新原文。
    - >-
      确认 Recorder 或其完整性审计失败会阻断状态推进，不能先越过门禁再补资产。
    - >-
      确认 bootstrap 候选：经实施门禁批准后，由 v2 implementation coder 在精确白名单内
      逐字转录已冻结的 task-analysis、planning、approved prompt 及其哈希；
      v2 可运行后，再由新 Recorder 保存 post-implementation audit、summary 等后置资产。
      必须如实标注前置资产的记录者，不能伪造为 Recorder 历史。
    - >-
      确认 v2 任务继续冻结 ch3，并禁止触碰 tutorial/、book 章节、Cargo.toml、
      Cargo.lock、examples/、crates/ 和 ch3 资产。
    - >-
      确认本任务不提交、不推送、不切换分支、不修改 remote；本地提交仍需独立提交门禁。

requires_user_confirmation: true
requested_confirmation: >-
  请确认上述 v2 需求边界，并逐项决定：Artifact Audit 归属、长期资产清单、
  资产拓扑/格式/修订策略、原文字节边界、安全与失败语义，以及 bootstrap 转录方案。
  在这些业务决定明确前，不进入计划阶段。
```

已经确认的事实：v1 的只读权限模型确实无法合法完成全部角色资产落盘；新增 Recorder 会改变角色、权限、状态和任务包，所以必须建立并保留独立的 v2。当前 v1、decision 和章节资产都还没有 Git 跟踪坐标，只能以工作树文件及当前哈希作为候选基线。

会影响实现的争议主要有三组：谁负责校验 Recorder 写入、什么内容才算“完整原文和全部长期资产”、以及不可改写与敏感信息处理如何同时成立。尤其是字节一致记录不能与自动脱敏兼得；较安全的候选语义是发现敏感信息即拒绝落盘，而不是让 Recorder 改写内容。

用户现在需要确认上面列出的业务决定。确认前，`workflow-v2-artifact-recorder-001` 不具备进入计划阶段的条件；ch3 继续暂停。本次没有修改文件、运行实现测试、提交或执行任何远程操作。

<oai-mem-citation>
<citation_entries>
MEMORY.md:42-44|note=[used prior rule to reuse workflow assets and inspect repository facts]
</citation_entries>
<rollout_ids>
01a0104b-8de7-7513-82d4-1d5ad076e3f0
</rollout_ids>
</oai-mem-citation>
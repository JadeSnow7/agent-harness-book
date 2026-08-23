```yaml
protocol_version: "1"
task_id: "workflow-v2-artifact-recorder-001"
role: "planning"
attempt: 2
status: "passed"

expected_result:
  observable_behavior:
    - "保留 workflow/v1 的 13 个现有文件字节不变：1 个 decision、5 个 workflow 根资产、7 个既有角色文件。"
    - "新增 17 个 v2 协议/工具/测试文件：1 个 decision、6 个 workflow 根资产、8 个角色文件、1 个工具、1 个测试。"
    - "新增 9 个 bootstrap 事件，每个事件严格由 1 个 payload 与 1 个 manifest 组成，共 18 个运行资产。"
    - "本任务 files_to_add 总数为 35：17 个 v2 基础文件加 18 个 bootstrap 文件；files_to_modify 为 0。"
    - "v2 共定义 8 个角色：v1 的 7 个既有逻辑角色加 Artifact Recorder。"
    - "Recorder 从独立原始 UTF-8 payload 文件读取字节，以外部 expected SHA-256 和 byte length 校验，不从会话复制文本冒充原始字节。"
    - "每个 payload 以 sequence、role、attempt 命名，并配对独立 .manifest.yaml；已有文件不可覆盖。"
    - "record 拒绝路径逃逸、符号链接、非法任务根、序号错误、attempt/revision 错误、身份不匹配、摘要不匹配、非法 UTF-8 和确定性敏感信息命中。"
    - "verify 复用 Recorder 的同一实现，重新检查路径、配对、序号、manifest canonical bytes、长度、哈希、身份、revision 链和敏感信息；Controller 不重复实现校验。"
    - "自动模式只接受平台原始导出；手动模式只接受受控 raw export 和明确 provenance/attestation；缺少原始文件时 fail closed。"
    - "bootstrap coder 只保存实施前已经形成或实施交接时真实生成的资产，并在 manifest 中标明 bootstrap-coder 与 historical；不预写 implementation、audit、summary、commit 或 commit-audit 结果。"
    - "v2 建立后，每次角色结果和人工门禁必须先由 Recorder 保存并经 Controller 调用 verify 成功，状态机才可继续。"
    - "Recorder 的终端产物是 payload/manifest 对和不含正文的机器结果，不递归保存 Recorder 自己的输出。"
  target_state: "workflow/v2 作为与 v1 并存的可运行、可离线测试、可解释长期资产协议存在；ch3 继续冻结。"

scope:
  counts:
    v1_files_protected: 13
    v2_base_files_to_add: 17
    v2_roles: 8
    bootstrap_events: 9
    bootstrap_payload_manifest_pairs: 9
    bootstrap_files_to_add: 18
    total_files_to_add: 35
    files_to_modify: 0
  files_to_add:
    - "docs/decisions/reader-ai-coding-workflow-v2.md"
    - "docs/prompts/workflow/v2/README.md"
    - "docs/prompts/workflow/v2/controller.md"
    - "docs/prompts/workflow/v2/manual-handoff.md"
    - "docs/prompts/workflow/v2/task-package.template.yaml"
    - "docs/prompts/workflow/v2/agent-result.template.yaml"
    - "docs/prompts/workflow/v2/artifact-input.template.json"
    - "docs/prompts/workflow/v2/roles/task-analysis.md"
    - "docs/prompts/workflow/v2/roles/planning.md"
    - "docs/prompts/workflow/v2/roles/implementation.md"
    - "docs/prompts/workflow/v2/roles/post-implementation-audit.md"
    - "docs/prompts/workflow/v2/roles/summary.md"
    - "docs/prompts/workflow/v2/roles/commit.md"
    - "docs/prompts/workflow/v2/roles/commit-audit.md"
    - "docs/prompts/workflow/v2/roles/artifact-recorder.md"
    - "docs/prompts/workflow/v2/tools/artifact_recorder.py"
    - "docs/prompts/workflow/v2/tests/test_artifact_recorder.py"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/001-task-analysis-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/001-task-analysis-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/002-controller-state-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/002-controller-state-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/003-user-gate-requirement-attempt-01.payload.txt"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/003-user-gate-requirement-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/004-planning-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/004-planning-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/005-controller-state-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/005-controller-state-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/006-user-gate-implementation-attempt-01.payload.txt"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/006-user-gate-implementation-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/007-approved-prompt-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/007-approved-prompt-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/008-task-package-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/008-task-package-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/009-controller-state-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/009-controller-state-attempt-01.manifest.yaml"
  files_to_modify: []
  forbidden_paths:
    - "docs/decisions/reader-ai-coding-workflow-v1.md"
    - "docs/prompts/workflow/v1/**"
    - "docs/chapters/**"
    - "tutorial/**"
    - "book/**"
    - "Cargo.toml"
    - "Cargo.lock"
    - "crates/**"
    - "examples/**"
    - ".github/**"
    - "docs/report.md"
    - "其他未列入 files_to_add 的既有文件"
  non_goals:
    - "不修改、迁移或删除 workflow/v1。"
    - "不重新执行、修复、审计或追认 ch3。"
    - "不实现独立 Artifact Audit 角色。"
    - "不让 Recorder 总结、改写、补全、规范化或自动脱敏 payload。"
    - "不提供签名、远程时间戳、透明日志或防恶意管理员能力。"
    - "不默认联网，不使用真实凭据。"
    - "不运行 Cargo、mdBook 或章节工程测试。"
    - "不执行 Git 写操作或远程操作。"

impact:
  public_api_changes:
    - "新增 workflow protocol_version 2 的任务包、角色结果、状态机及 Artifact Recorder 契约；v1 API 原样保留。"
    - "新增 Python CLI：artifact_recorder.py inspect、record、verify。"
    - "新增严格 JSON input descriptor 和 canonical JSON-as-YAML manifest schema。"
  dependency_changes:
    - "无；仅使用 Python 标准库。"
  side_effects:
    - "record 仅能在合法 task artifact root 下创建 payload/manifest 对和必要目录。"
    - "测试仅在 /tmp 下创建临时 fixture，并由 unittest 清理。"
    - "inspect 与 verify 必须只读。"
  risks:
    - "仓库当前脏且 v1 为未跟踪用户资产，常规 git diff 无法保护它，必须对恰好 13 个 v1 文件逐项 SHA-256 前后比对。"
    - "平台可能无法提供真正的原始消息 payload 文件；这种情况下 bootstrap 必须停止，不能用复制后的文本替代。"
    - "确定性敏感扫描只能识别规定模式，不能证明不存在所有秘密或无关私人内容。"
    - "两个最终文件无法获得跨平台单事务原子性；实现必须采用同目录临时文件、no-overwrite 安装、常规失败清理和可验证 orphan 恢复规则。"
    - "严格扫描可能产生误报；误报只能由上游产生新的安全 payload 或新计划处理。"
  known_limitations:
    - "manifest 不提供密码学签名或外部可信时间。"
    - "manual_raw_export 的真实性依赖操作者使用平台 raw export 并作明确 attestation；复制粘贴不被接受。"
    - "Recorder 不做业务语义审计；语义正确性仍由 post-implementation-audit 等角色负责。"
    - "Recorder 自己的命令结果不递归保存；配对 manifest 和 Controller verify 结果构成终端证据。"
    - "bootstrap 只能保存实际可获得的原始前置输出和实施交接时真实生成的 snapshot，不能补造不存在的历史 snapshot。"
  leftovers:
    - "实现完成后仍需独立执行后审计、用户审计确认、汇总、提交批准、提交审核和最终验收。"
    - "ch3 继续暂停；v2 最终验收后才可从 ch3 计划阶段重新开始。"
  later_milestones_touched: false

delegation:
  complexity: "high"
  coding_agents:
    - "单一 Luna 类 implementation coder"
  file_ownership:
    - "该 coder 独占全部 35 个 files_to_add；禁止第二个写 Agent 并发修改。"
  integration_owner: "单一 implementation coder"

validation:
  required_commands:
    - "pwd"
    - "git status --short --branch"
    - "git rev-parse HEAD"
    - "python3 --version"
    - "python3 -m py_compile docs/prompts/workflow/v2/tools/artifact_recorder.py docs/prompts/workflow/v2/tests/test_artifact_recorder.py"
    - "python3 -m unittest discover -s docs/prompts/workflow/v2/tests -p 'test_*.py' -v"
    - "python3 docs/prompts/workflow/v2/tools/artifact_recorder.py --help"
    - "python3 docs/prompts/workflow/v2/tools/artifact_recorder.py verify --repo-root /Users/huaodong/Documents/ChatGPT/agent-harness-book --artifact-root docs/workflow-runs/workflow-v2-artifact-recorder-001 --task-id workflow-v2-artifact-recorder-001 --chapter meta"
    - "shasum -a 256 <恰好 13 个显式 v1 文件>"
    - "git diff --check"
    - "git status --short"
  expected_evidence:
    - "Python 编译与 unittest 的真实退出码。"
    - "测试覆盖 exact bytes、hash/length、UTF-8、敏感信息、路径、symlink、no-overwrite、sequence、attempt/revision、identity、pair recovery 与 verify 只读性。"
    - "bootstrap verify 返回成功，artifact_count=9，last_sequence=9；运行目录恰好有 9 个 payload 和 9 个 manifest。"
    - "恰好 13 个 v1 文件的 SHA-256 与计划记录完全相同。"
    - "新增文件恰好 35 个：17 个 v2 基础文件和 18 个 bootstrap 文件；既有文件修改数为 0。"
    - "v2 角色文件恰好 8 个：7 个迁移角色和 1 个 Recorder。"
    - "所有命令 network_used=false。"
  checks_not_required:
    - "Cargo 和 Rust 测试：未修改 Rust/Cargo。"
    - "mdBook：未修改 book/。"
    - "真实网络、API Key、远程 CI、push、部署。"

implementation_prompt:
  status: "draft"
  full_text: |
    任务：workflow-v2-artifact-recorder-001

    你是本任务唯一的 Luna 类 implementation coder。只有收到本轮 approval_status: approved、用户批准原文、未被改写的完整 Prompt 和完整 bootstrap source bundle 后才能实施。

    一、目标

    保持 workflow/v1 的 13 个现有文件字节不变，新建 breaking workflow/v2。v2 新增 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具，用于保存冻结的完整角色输出、自然语言、用户门禁原文、task package、approved prompt、Controller 状态和后续角色结果。

    数量契约：

    - v1 保护文件：13；
    - v2 基础新增文件：17；
    - v2 角色：8，其中 7 个从 v1 迁移、1 个为 Artifact Recorder；
    - bootstrap 事件：9；
    - bootstrap pair：9，即 9 payload 加 9 manifest，共 18 文件；
    - files_to_add 总数：35；
    - files_to_modify：0。

    Recorder 只能机械保存冻结 UTF-8 bytes 和 metadata，不能总结、修正、补全、规范化或自动脱敏。Controller 复用同一工具的 verify 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 验证，不另写校验逻辑，也不判断内容正确性。

    章节 run root 为 docs/chapters/chXX/workflow/<task_id>/；meta run root 为 docs/workflow-runs/<task_id>/。本任务使用后者。

    payload 原样保存，不包装、不转换换行、不追加 newline。metadata 保存为独立 .manifest.yaml，内容采用标准库产生的 canonical JSON；JSON 是 YAML 1.2 子集。纠错新建 sequence 和 attempt/revision，禁止覆盖。

    Recorder 的 payload/manifest 对和不含正文的 JSON 命令结果是终端证据，不递归保存 Recorder 自身输出。

    二、基线

    repo:
    /Users/huaodong/Documents/ChatGPT/agent-harness-book

    HEAD:
    355a7d0d240e68f9e7ffc336e42913b609f5ac5e

    branch:
    main，ahead origin/main 4，工作树已脏，previous_commit=null。

    Python:
    /opt/homebrew/bin/python3，计划时为 Python 3.14.6。

    必须保护的 13 个 v1 文件及正确 SHA-256：

    1. 5ed492e0bc21d9ddc4e910401bb79258c3dd1c7da5fcbf6306ee208e5738c52a  docs/decisions/reader-ai-coding-workflow-v1.md
    2. 55d1fc1ca1aa52a6f6db89ab952e18ca3ad6fb319ec2619ff2c891376db522c4  docs/prompts/workflow/v1/README.md
    3. bdb91956c17e98688e2b382e616417e75cd73f89bb0a907d4ea848e4e9db94f6  docs/prompts/workflow/v1/controller.md
    4. 64918b4c5895c86ad55dcc788eaaa1ca8f2f362a3ed41fa033bfc31aff0a0a09  docs/prompts/workflow/v1/manual-handoff.md
    5. c1991453be55040fc6d82f041c272fda468569650676113535eac2881bda5aa1  docs/prompts/workflow/v1/agent-result.template.yaml
    6. 584ec93f89c8c83bd8ec1ded6a3829988123856e68d9de8a7aa85560a30fb21b  docs/prompts/workflow/v1/task-package.template.yaml
    7. 6d911bfa1a1bc6b88b3a70e9bcb549ecef9a1e9a54a9a56450039e50d3136cdc  docs/prompts/workflow/v1/roles/task-analysis.md
    8. 7c09885ad499cde7bc693a9866b0da89d819fa2758361c41965a53cf7ebfb955  docs/prompts/workflow/v1/roles/planning.md
    9. f0dc3e5f6f99a512311f6da064b09779fda35465da9bfb89346002f149d18a30  docs/prompts/workflow/v1/roles/implementation.md
    10. 0dbb752e55daf74adbc58f47b92e744332809f83481c2342b07775b16034b43e  docs/prompts/workflow/v1/roles/post-implementation-audit.md
    11. 5fe21f781731ad0d89598c37ec8e758da8fe584ff3a0aea2d14c29eadb5014e0  docs/prompts/workflow/v1/roles/summary.md
    12. 0122acd806a5cdefe57f8dddd8fe354d14f8d0ff2135298277898d25d4a3c2fc  docs/prompts/workflow/v1/roles/commit.md
    13. cd7113da6430594225acfa626ac109a093cde46b71e3b039f1cfa78e251034d0  docs/prompts/workflow/v1/roles/commit-audit.md

    开始和结束时均逐项计算这 13 个值。任一不一致立即停止，不能更新基线。

    三、bootstrap 输入门禁

    实施前执行 pwd、git status --short --branch、git rev-parse HEAD、python3 --version，并确认所有 35 个目标文件均不存在。

    交接必须提供 /tmp 下的 bootstrap source bundle，包含 9 个独立 UTF-8 source 和 9 个 strict JSON descriptor。每个 descriptor 必须包含：

    source_path、expected_byte_length、expected_sha256、task_id、chapter、sequence、artifact_key、role、attempt、status、payload_extension、media_type、target_path、source_provenance、source_reference、identity_source、historical、recorded_by、revision_of。

    九个事件固定为：

    1. task_analysis / passed / payload_header / 原始完整角色输出；
    2. controller_state / awaiting_requirement_confirmation / external_descriptor / 原始 Controller 响应或真实 snapshot；
    3. user_gate_requirement / confirmed / external_descriptor / 用户门禁原文；
    4. planning / passed / payload_header / planning attempt-02 完整原始输出；
    5. controller_state / awaiting_implementation_approval / external_descriptor / 原始 Controller 响应或真实 snapshot；
    6. user_gate_implementation / approved / external_descriptor / 本轮批准实施原文；
    7. approved_prompt / approved / external_descriptor / 本 Prompt 的冻结批准文本；
    8. task_package / approved / external_descriptor / 实施交接时真实生成的获批任务包；
    9. controller_state / implementation / external_descriptor / 派发 implementation 时真实生成的状态 snapshot。

    文件名使用 sequence 001 至 009。planning 的运行文件名仍是 004-planning-attempt-01.payload.md，因为它是该 run 中第一次被正式批准并归档的 planning artifact；manifest 必须另外记录 source planning attempt=2 和 bootstrap provenance，不能伪称 attempt-01 的内容来自被退回计划。若工具 schema 不能诚实表达 source attempt 与 artifact attempt 的区别，停止并回到计划，不得篡改事实。

    自动模式只接受 platform_raw_export 或 bootstrap_handoff。手动模式只接受 manual_raw_export，并要求 export_method 与固定 attestation，证明使用 raw/download export、未复制粘贴、导出后未修改。manual_copy、clipboard_copy、reconstructed_from_chat 必须拒绝。

    bundle 不完整、expected 值不独立、含 symlink、无法证明 raw export 或只能从聊天复制时，在仓库写入前停止。

    四、精确白名单

    新增 17 个基础文件：

    1. docs/decisions/reader-ai-coding-workflow-v2.md
    2. docs/prompts/workflow/v2/README.md
    3. docs/prompts/workflow/v2/controller.md
    4. docs/prompts/workflow/v2/manual-handoff.md
    5. docs/prompts/workflow/v2/task-package.template.yaml
    6. docs/prompts/workflow/v2/agent-result.template.yaml
    7. docs/prompts/workflow/v2/artifact-input.template.json
    8. docs/prompts/workflow/v2/roles/task-analysis.md
    9. docs/prompts/workflow/v2/roles/planning.md
    10. docs/prompts/workflow/v2/roles/implementation.md
    11. docs/prompts/workflow/v2/roles/post-implementation-audit.md
    12. docs/prompts/workflow/v2/roles/summary.md
    13. docs/prompts/workflow/v2/roles/commit.md
    14. docs/prompts/workflow/v2/roles/commit-audit.md
    15. docs/prompts/workflow/v2/roles/artifact-recorder.md
    16. docs/prompts/workflow/v2/tools/artifact_recorder.py
    17. docs/prompts/workflow/v2/tests/test_artifact_recorder.py

    新增 18 个 bootstrap 文件：

    - 001-task-analysis-attempt-01.payload.md 及同 stem manifest；
    - 002-controller-state-attempt-01.payload.md 及 manifest；
    - 003-user-gate-requirement-attempt-01.payload.txt 及 manifest；
    - 004-planning-attempt-01.payload.md 及 manifest；
    - 005-controller-state-attempt-01.payload.md 及 manifest；
    - 006-user-gate-implementation-attempt-01.payload.txt 及 manifest；
    - 007-approved-prompt-attempt-01.payload.md 及 manifest；
    - 008-task-package-attempt-01.payload.yaml 及 manifest；
    - 009-controller-state-attempt-01.payload.yaml 及 manifest。

    以上均位于 docs/workflow-runs/workflow-v2-artifact-recorder-001/。

    仅用 apply_patch 创建基础文件。bootstrap payload/manifest 只能通过 Recorder 从 source bundle 写入。

    禁止修改任何既有文件，尤其是 v1、docs/chapters、tutorial、book、Cargo、crates、examples、.github 和 docs/report.md。

    五、文档与角色要求

    reader-ai-coding-workflow-v2.md 必须记录：

    - v2 与 v1 并存；
    - Recorder 的最小写权限；
    - raw export 和 bootstrap 边界；
    - record/verify 成功后才推进状态；
    - no-overwrite、sequence、attempt/revision；
    - 敏感信息 fail closed；
    - Controller 只做确定性校验；
    - Recorder 非递归终止规则；
    - payload/manifest 为可恢复顺序，不虚构跨平台双文件事务；
    - 无签名、外部可信时间和完备 secret detection。

    README 索引 17 个基础文件中的协议资产，权限表列出 8 个角色，并给出：

    raw export → Recorder record → Controller verify → state transition

    controller.md 保留 v1 的五个人工门禁和职责隔离，升级 protocol_version 2。每个角色结果、用户门禁、task package、approved prompt、状态 snapshot 都要先保存和 verify。source 不可用时 fail closed。Controller 不判断业务正确性，不递归记录 Recorder 结果。

    manual-handoff.md 必须区分“复制结果用于协作”和“raw export 用于 byte evidence”。没有平台 raw/download export 时停止，不能把 clipboard copy 当 original。

    两个 YAML 模板升级到 v2，增加 artifact root、sequence、attempt、artifact key、raw source、expected length/hash、provenance、manifest、verification、bootstrap 和 approval artifact reference。

    artifact-input.template.json 展示严格 descriptor，不含疑似真实 secret。

    8 个角色文件恰好为：

    - task-analysis
    - planning
    - implementation
    - post-implementation-audit
    - summary
    - commit
    - commit-audit
    - artifact-recorder

    前 7 个完整保留 v1 权限、人工门禁、停止条件和报告要求。其输出必须以 fenced YAML 结构头开始，至少包含 protocol_version、task_id、role、status，后面保存完整自然语言。角色不自行落盘，由 Controller 派发 Recorder。

    Artifact Recorder 只调用 inspect、record、verify，只写当前 artifact root，不碰业务文件和 Git，不回显 payload 或 secret。它的配对 manifest 与无正文 JSON 结果是终端输出，不再递归记录。

    六、工具要求

    artifact_recorder.py 只用标准库，提供：

    - inspect --source
    - record --repo-root --artifact-root --descriptor
    - verify --repo-root --artifact-root --task-id --chapter

    必须实现：

    - 成功输出稳定 JSON，不打印 payload；
    - 错误只报告 code/category/field/safe location；
    - strict JSON descriptor，拒绝未知字段；
    - source 为普通非 symlink 文件，严格 UTF-8，保存原 bytes；
    - 写前校验 expected length/hash；
    - task_id、artifact_key、sequence、attempt、role/status allowlist；
    - target 由 sequence、role、attempt、extension 推导：
      NNN-<role-with-hyphens>-attempt-NN.payload.<ext>
    - target 为 artifact root 下单一 basename；
    - artifact root 只能是约定的 meta/chapter 形式；
    - repo 到 root 的已有组件、source、descriptor、payload、manifest 全部 lstat 并拒绝 symlink；
    - manifest path 唯一派生；
    - manifest 为 sort_keys、ensure_ascii=False、indent=2 加单个 LF 的 canonical JSON-as-YAML；
    - manifest 不保存 source 绝对路径；
    - payload_header 解析开头 fenced YAML 中的 task_id/role/status；
    - external_descriptor 用于 gate、approved prompt、task package、Controller state；
    - sequence 从 1 连续追加；
    - correction 使用新 sequence、递增 attempt 和 revision_of；
    - 不覆盖，成功记录的重复调用返回 already_exists；
    - 同目录 temp、flush/fsync、no-overwrite 安装、普通失败清理；
    - 仅在 payload orphan 完整匹配、无后续 sequence 时允许补装 deterministic manifest；
    - manifest orphan 永远 fail closed；
    - verify 只读检查 pair、sequence、canonical manifest、hash/length、identity、revision、symlink、未知文件和临时残留；
    - inspect 只返回 length/hash/utf8/sensitive categories；
    - 敏感扫描覆盖非占位 Authorization credential、常见 token/key、敏感环境变量赋值、private-key 内容和 source 名为 .env，同时避免政策文字误报；
    - exit code 区分 success、input/schema、integrity/security；
    - 不联网、不读任务外文件。

    七、测试矩阵

    test_artifact_recorder.py 使用 unittest、tempfile、subprocess/mock，fixture 位于 /tmp，至少覆盖：

    - ASCII、中文、emoji、LF、CRLF、无末尾 newline 的 exact bytes；
    - length/hash mismatch、invalid UTF-8；
    - 敏感模式拒绝且不回显值，政策词语不误报；
    - meta/chapter root 正常；
    - absolute target、traversal、slash、错误 root/chapter；
    - source/root/payload/manifest symlink；
    - descriptor 缺字段、未知字段、错误类型、非法 provenance/attestation；
    - target/task/role/status/header identity mismatch；
    - sequence 首项、gap、重复、倒退；
    - attempt/revision 正确和错误链；
    - no-overwrite 且原 bytes 不变；
    - record 后 verify；
    - manifest/payload tamper、orphan、未知/temp 文件；
    - 合法 payload orphan recovery；
    - 模拟第二步安装失败的清理；
    - inspect/verify 前后目录不变；
    - CLI 输出不含正文。

    八、bootstrap

    工具通过 py_compile 和 unittest 后，按 001 至 009 逐项 record。每次后运行 verify；上一项失败不得继续。

    最终必须同时满足：

    - artifact_count=9；
    - last_sequence=9；
    - 运行目录恰好 9 payload + 9 manifest；
    - 不存在 010-implementation 或任何后置资产。

    九、验证

    执行并记录真实退出码：

    python3 -m py_compile docs/prompts/workflow/v2/tools/artifact_recorder.py docs/prompts/workflow/v2/tests/test_artifact_recorder.py

    python3 -m unittest discover -s docs/prompts/workflow/v2/tests -p 'test_*.py' -v

    python3 docs/prompts/workflow/v2/tools/artifact_recorder.py --help

    python3 docs/prompts/workflow/v2/tools/artifact_recorder.py verify --repo-root /Users/huaodong/Documents/ChatGPT/agent-harness-book --artifact-root docs/workflow-runs/workflow-v2-artifact-recorder-001 --task-id workflow-v2-artifact-recorder-001 --chapter meta

    shasum -a 256 后跟恰好 13 个显式 v1 路径。

    git diff --check

    git status --short

    不运行 Cargo、mdBook、网络、真实凭据、远程 CI 或部署。

    十、停止条件与禁止事项

    以下任一情况立即停止：

    - 缺少本轮批准、Prompt 被改写或 task_id 不匹配；
    - HEAD、root 或 13 个 v1 SHA 任一不符；
    - 35 个目标文件任一已有未知内容；
    - bootstrap bundle 不完整、非 raw、hash/length 不独立、含 symlink、identity 不符或敏感命中；
    - planning attempt-02 无法被 manifest 诚实表达；
    - 需要修改白名单外文件或新增依赖；
    - 测试环境不可靠；
    - 需要网络、凭据、Git 写或远程动作。

    禁止覆盖、自动脱敏、补造历史、预写后置结果、修改 v1/ch3、git add、commit、push、tag、branch/remote 操作、reset、revert 或清理用户工作树。

    十一、完成报告

    返回 implementation 结构化结果和十一项报告：

    1. 实现或修改摘要；
    2. 新增文件；
    3. 修改文件；
    4. 公共 API 变化；
    5. 新依赖及用途；
    6. 执行的验证命令；
    7. 验证结果；
    8. 未运行检查及原因；
    9. 已知限制；
    10. 遗留问题；
    11. 是否触及后续 milestone。

    另列：

    - 新增文件是否恰好 35，基础文件是否 17，bootstrap 是否 18；
    - v2 角色是否恰好 8；
    - 9 个 bootstrap provenance 和 manifest；
    - 13 个 v1 SHA 前后是否一致；
    - actual files 与白名单；
    - network/credential/Git 写操作；
    - ch3 仍冻结；
    - 结果只是待独立审计的 implementation 声明。

requires_user_confirmation: true
requested_confirmation: "请审阅 planning attempt-02 和完整编码 Prompt；只有明确回复‘批准实施’后才会准备 bootstrap source bundle 并派发唯一 coder。"
```

完整编码 Prompt如下，与结构化字段中的正文一致。

---

# workflow-v2-artifact-recorder-001 实施 Prompt

你是本任务唯一的 Luna 类 implementation coder。只有收到本轮 `approval_status: approved`、用户批准原文、未被改写的完整 Prompt 和完整 bootstrap source bundle 后才能实施。

## 一、目标

保持 workflow/v1 的 13 个现有文件字节不变，新建 breaking workflow/v2。v2 新增 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具，用于保存冻结的完整角色输出、自然语言、用户门禁原文、task package、approved prompt、Controller 状态和后续角色结果。

数量契约：

- v1 保护文件：13；
- v2 基础新增文件：17；
- v2 角色：8，其中 7 个从 v1 迁移、1 个为 Artifact Recorder；
- bootstrap 事件：9；
- bootstrap pair：9，即 9 payload 加 9 manifest，共 18 文件；
- `files_to_add` 总数：35；
- `files_to_modify`：0。

Recorder 只能机械保存冻结 UTF-8 bytes 和 metadata，不能总结、修正、补全、规范化或自动脱敏。Controller 复用同一工具的 `verify` 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 验证，不另写校验逻辑，也不判断内容正确性。

章节 run root 为：

```text
docs/chapters/chXX/workflow/<task_id>/
```

meta run root 为：

```text
docs/workflow-runs/<task_id>/
```

payload 原样保存，不包装、不转换换行、不追加 newline。metadata 保存为独立 `.manifest.yaml`，内容采用标准库产生的 canonical JSON；JSON 是 YAML 1.2 子集。纠错新建 sequence 和 attempt/revision，禁止覆盖。

Recorder 的 payload/manifest 对和不含正文的 JSON 命令结果是终端证据，不递归保存 Recorder 自身输出。

## 二、基线

工程根目录：

```text
/Users/huaodong/Documents/ChatGPT/agent-harness-book
```

工程坐标：

```text
HEAD 355a7d0d240e68f9e7ffc336e42913b609f5ac5e
branch main
main ahead origin/main 4
previous_commit null
```

工作树已脏。Python 位于 `/opt/homebrew/bin/python3`，计划时版本为 3.14.6。

必须保护的 13 个 v1 文件及正确 SHA-256：

```text
5ed492e0bc21d9ddc4e910401bb79258c3dd1c7da5fcbf6306ee208e5738c52a  docs/decisions/reader-ai-coding-workflow-v1.md
55d1fc1ca1aa52a6f6db89ab952e18ca3ad6fb319ec2619ff2c891376db522c4  docs/prompts/workflow/v1/README.md
bdb91956c17e98688e2b382e616417e75cd73f89bb0a907d4ea848e4e9db94f6  docs/prompts/workflow/v1/controller.md
64918b4c5895c86ad55dcc788eaaa1ca8f2f362a3ed41fa033bfc31aff0a0a09  docs/prompts/workflow/v1/manual-handoff.md
c1991453be55040fc6d82f041c272fda468569650676113535eac2881bda5aa1  docs/prompts/workflow/v1/agent-result.template.yaml
584ec93f89c8c83bd8ec1ded6a3829988123856e68d9de8a7aa85560a30fb21b  docs/prompts/workflow/v1/task-package.template.yaml
6d911bfa1a1bc6b88b3a70e9bcb549ecef9a1e9a54a9a56450039e50d3136cdc  docs/prompts/workflow/v1/roles/task-analysis.md
7c09885ad499cde7bc693a9866b0da89d819fa2758361c41965a53cf7ebfb955  docs/prompts/workflow/v1/roles/planning.md
f0dc3e5f6f99a512311f6da064b09779fda35465da9bfb89346002f149d18a30  docs/prompts/workflow/v1/roles/implementation.md
0dbb752e55daf74adbc58f47b92e744332809f83481c2342b07775b16034b43e  docs/prompts/workflow/v1/roles/post-implementation-audit.md
5fe21f781731ad0d89598c37ec8e758da8fe584ff3a0aea2d14c29eadb5014e0  docs/prompts/workflow/v1/roles/summary.md
0122acd806a5cdefe57f8dddd8fe354d14f8d0ff2135298277898d25d4a3c2fc  docs/prompts/workflow/v1/roles/commit.md
cd7113da6430594225acfa626ac109a093cde46b71e3b039f1cfa78e251034d0  docs/prompts/workflow/v1/roles/commit-audit.md
```

开始和结束时均逐项计算这 13 个值。任一不一致立即停止，不能更新基线。

## 三、bootstrap 输入门禁

实施前执行并记录：

```bash
pwd
git status --short --branch
git rev-parse HEAD
python3 --version
```

确认所有 35 个目标文件均不存在。

交接必须提供 `/tmp` 下的 bootstrap source bundle，包含 9 个独立 UTF-8 source 和 9 个 strict JSON descriptor。每个 descriptor 必须包含：

```text
source_path
expected_byte_length
expected_sha256
task_id
chapter
sequence
artifact_key
role
attempt
status
payload_extension
media_type
target_path
source_provenance
source_reference
identity_source
historical
recorded_by
revision_of
```

九个事件固定为：

1. `task_analysis / passed / payload_header`：原始完整角色输出；
2. `controller_state / awaiting_requirement_confirmation / external_descriptor`：原始 Controller 响应或真实 snapshot；
3. `user_gate_requirement / confirmed / external_descriptor`：用户门禁原文；
4. `planning / passed / payload_header`：planning attempt-02 完整原始输出；
5. `controller_state / awaiting_implementation_approval / external_descriptor`：原始 Controller 响应或真实 snapshot；
6. `user_gate_implementation / approved / external_descriptor`：本轮批准实施原文；
7. `approved_prompt / approved / external_descriptor`：本 Prompt 的冻结批准文本；
8. `task_package / approved / external_descriptor`：实施交接时真实生成的获批任务包；
9. `controller_state / implementation / external_descriptor`：派发 implementation 时真实生成的状态 snapshot。

文件名使用 sequence 001 至 009。planning 的运行文件名仍为 `004-planning-attempt-01.payload.md`，因为它是 run 中第一次正式批准并归档的 planning artifact；manifest 必须另外记录 source planning attempt=2 和 bootstrap provenance，不能伪称内容来自被退回的 attempt-01。若 schema 不能诚实表达 source attempt 与 artifact attempt 的区别，停止并回到计划。

自动模式只接受 `platform_raw_export` 或 `bootstrap_handoff`。手动模式只接受 `manual_raw_export`，并要求 `export_method` 与固定 attestation，证明使用 raw/download export、未复制粘贴、导出后未修改。`manual_copy`、`clipboard_copy`、`reconstructed_from_chat` 必须拒绝。

bundle 不完整、expected 值不独立、含 symlink、无法证明 raw export 或只能从聊天复制时，在仓库写入前停止。

## 四、精确白名单

新增 17 个基础文件：

1. `docs/decisions/reader-ai-coding-workflow-v2.md`
2. `docs/prompts/workflow/v2/README.md`
3. `docs/prompts/workflow/v2/controller.md`
4. `docs/prompts/workflow/v2/manual-handoff.md`
5. `docs/prompts/workflow/v2/task-package.template.yaml`
6. `docs/prompts/workflow/v2/agent-result.template.yaml`
7. `docs/prompts/workflow/v2/artifact-input.template.json`
8. `docs/prompts/workflow/v2/roles/task-analysis.md`
9. `docs/prompts/workflow/v2/roles/planning.md`
10. `docs/prompts/workflow/v2/roles/implementation.md`
11. `docs/prompts/workflow/v2/roles/post-implementation-audit.md`
12. `docs/prompts/workflow/v2/roles/summary.md`
13. `docs/prompts/workflow/v2/roles/commit.md`
14. `docs/prompts/workflow/v2/roles/commit-audit.md`
15. `docs/prompts/workflow/v2/roles/artifact-recorder.md`
16. `docs/prompts/workflow/v2/tools/artifact_recorder.py`
17. `docs/prompts/workflow/v2/tests/test_artifact_recorder.py`

新增 18 个 bootstrap 文件：

- `001-task-analysis-attempt-01.payload.md` 及同 stem manifest；
- `002-controller-state-attempt-01.payload.md` 及 manifest；
- `003-user-gate-requirement-attempt-01.payload.txt` 及 manifest；
- `004-planning-attempt-01.payload.md` 及 manifest；
- `005-controller-state-attempt-01.payload.md` 及 manifest；
- `006-user-gate-implementation-attempt-01.payload.txt` 及 manifest；
- `007-approved-prompt-attempt-01.payload.md` 及 manifest；
- `008-task-package-attempt-01.payload.yaml` 及 manifest；
- `009-controller-state-attempt-01.payload.yaml` 及 manifest。

以上均位于：

```text
docs/workflow-runs/workflow-v2-artifact-recorder-001/
```

仅用 `apply_patch` 创建基础文件。bootstrap payload/manifest 只能通过 Recorder 从 source bundle 写入。

禁止修改任何既有文件，尤其是 v1、`docs/chapters`、`tutorial`、`book`、Cargo、`crates`、`examples`、`.github` 和 `docs/report.md`。

## 五、文档与角色要求

v2 decision 必须记录：

- v2 与 v1 并存；
- Recorder 最小写权限；
- raw export 和 bootstrap 边界；
- record/verify 成功后才推进状态；
- no-overwrite、sequence、attempt/revision；
- 敏感信息 fail closed；
- Controller 只做确定性校验；
- Recorder 非递归终止；
- payload/manifest 使用可恢复顺序，不虚构跨平台双文件事务；
- 无签名、外部可信时间和完备 secret detection。

README 索引完整 v2 资产，权限表列出 8 个角色，并给出：

```text
raw export → Recorder record → Controller verify → state transition
```

Controller 保留 v1 的五个人工门禁和职责隔离，升级到 protocol_version 2。每个角色结果、用户门禁、task package、approved prompt、状态 snapshot 都要先保存和 verify。source 不可用时 fail closed。Controller 不判断业务正确性，不递归记录 Recorder 结果。

manual handoff 必须区分“复制结果用于协作”和“raw export 用于 byte evidence”。没有平台 raw/download export 时停止，不能把 clipboard copy 当 original。

两个 YAML 模板增加 artifact root、sequence、attempt、artifact key、raw source、expected length/hash、provenance、manifest、verification、bootstrap 和 approval artifact reference。

`artifact-input.template.json` 展示严格 descriptor，不含疑似真实 secret。

8 个角色文件恰好为：

- task-analysis
- planning
- implementation
- post-implementation-audit
- summary
- commit
- commit-audit
- artifact-recorder

前 7 个完整保留 v1 权限、人工门禁、停止条件和报告要求。输出以 fenced YAML 结构头开始，至少包含 `protocol_version`、`task_id`、`role`、`status`，后面保留完整自然语言。角色不自行落盘，由 Controller 派发 Recorder。

Artifact Recorder 只调用 inspect、record、verify，只写当前 artifact root，不碰业务文件和 Git，不回显 payload 或 secret。其配对 manifest 与无正文 JSON 结果是终端输出，不递归记录。

## 六、工具要求

`artifact_recorder.py` 只用标准库，提供：

```text
inspect --source
record --repo-root --artifact-root --descriptor
verify --repo-root --artifact-root --task-id --chapter
```

必须实现：

- 成功输出稳定 JSON，不打印 payload；
- 错误只报告 code/category/field/safe location；
- strict JSON descriptor，拒绝未知字段；
- source 为普通非 symlink 文件，严格 UTF-8，保存原 bytes；
- 写前校验 expected length/hash；
- task_id、artifact_key、sequence、attempt、role/status allowlist；
- target 由 sequence、role、attempt、extension 推导：
  `NNN-<role-with-hyphens>-attempt-NN.payload.<ext>`；
- target 为 artifact root 下单一 basename；
- artifact root 只能是约定的 meta/chapter 形式；
- repo 到 root 的已有组件、source、descriptor、payload、manifest 全部 `lstat` 并拒绝 symlink；
- manifest path 唯一派生；
- manifest 为 `sort_keys=True`、`ensure_ascii=False`、`indent=2` 加单个 LF 的 canonical JSON-as-YAML；
- manifest 不保存 source 绝对路径；
- `payload_header` 解析开头 fenced YAML 中的 task_id/role/status；
- `external_descriptor` 用于 gate、approved prompt、task package、Controller state；
- sequence 从 1 连续追加；
- correction 使用新 sequence、递增 attempt 和 revision_of；
- 不覆盖，成功记录的重复调用返回 `already_exists`；
- 同目录 temp、flush/fsync、no-overwrite 安装、普通失败清理；
- 仅在 payload orphan 完整匹配、无后续 sequence 时允许补装 deterministic manifest；
- manifest orphan 永远 fail closed；
- verify 只读检查 pair、sequence、canonical manifest、hash/length、identity、revision、symlink、未知文件和临时残留；
- inspect 只返回 length/hash/UTF-8/sensitive categories；
- 敏感扫描覆盖非占位 Authorization credential、常见 token/key、敏感环境变量赋值、private-key 内容和 source 名为 `.env`，同时避免政策文字误报；
- exit code 区分 success、input/schema、integrity/security；
- 不联网、不读任务外文件。

## 七、测试矩阵

测试使用 `unittest`、`tempfile`、`subprocess/mock`，fixture 位于 `/tmp`，至少覆盖：

- ASCII、中文、emoji、LF、CRLF、无末尾 newline；
- length/hash mismatch、invalid UTF-8；
- 敏感模式拒绝且不回显值，政策词语不误报；
- meta/chapter root；
- absolute target、traversal、slash、错误 root/chapter；
- source/root/payload/manifest symlink；
- descriptor 缺字段、未知字段、错误类型、非法 provenance/attestation；
- target/task/role/status/header identity mismatch；
- sequence 首项、gap、重复、倒退；
- attempt/revision 正确和错误链；
- no-overwrite 且原 bytes 不变；
- record 后 verify；
- manifest/payload tamper、orphan、未知/temp 文件；
- 合法 payload orphan recovery；
- 模拟第二步安装失败的清理；
- inspect/verify 前后目录不变；
- CLI 输出不含正文。

## 八、bootstrap

工具通过 `py_compile` 和 `unittest` 后，按 001 至 009 逐项 `record`。每次后运行 `verify`；上一项失败不得继续。

最终必须同时满足：

- `artifact_count=9`；
- `last_sequence=9`；
- 运行目录恰好 9 payload 加 9 manifest；
- 不存在 010-implementation 或任何后置资产。

## 九、验证

执行并记录真实退出码：

```bash
python3 -m py_compile \
  docs/prompts/workflow/v2/tools/artifact_recorder.py \
  docs/prompts/workflow/v2/tests/test_artifact_recorder.py

python3 -m unittest discover \
  -s docs/prompts/workflow/v2/tests \
  -p 'test_*.py' \
  -v

python3 docs/prompts/workflow/v2/tools/artifact_recorder.py --help

python3 docs/prompts/workflow/v2/tools/artifact_recorder.py verify \
  --repo-root /Users/huaodong/Documents/ChatGPT/agent-harness-book \
  --artifact-root docs/workflow-runs/workflow-v2-artifact-recorder-001 \
  --task-id workflow-v2-artifact-recorder-001 \
  --chapter meta

git diff --check
git status --short
```

另执行 `shasum -a 256`，后跟恰好 13 个显式 v1 路径。

不运行 Cargo、mdBook、网络、真实凭据、远程 CI 或部署。

## 十、停止条件与禁止事项

以下任一情况立即停止：

- 缺少本轮批准、Prompt 被改写或 task_id 不匹配；
- HEAD、root 或 13 个 v1 SHA 任一不符；
- 35 个目标文件任一已有未知内容；
- bootstrap bundle 不完整、非 raw、hash/length 不独立、含 symlink、identity 不符或敏感命中；
- planning attempt-02 无法被 manifest 诚实表达；
- 需要修改白名单外文件或新增依赖；
- 测试环境不可靠；
- 需要网络、凭据、Git 写或远程动作。

禁止覆盖、自动脱敏、补造历史、预写后置结果、修改 v1/ch3、`git add`、commit、push、tag、branch/remote 操作、reset、revert 或清理用户工作树。

## 十一、完成报告

返回 implementation 结构化结果和十一项报告：

1. 实现或修改摘要；
2. 新增文件；
3. 修改文件；
4. 公共 API 变化；
5. 新依赖及用途；
6. 执行的验证命令；
7. 验证结果；
8. 未运行检查及原因；
9. 已知限制；
10. 遗留问题；
11. 是否触及后续 milestone。

另列：

- 新增文件是否恰好 35，基础文件是否 17，bootstrap 是否 18；
- v2 角色是否恰好 8；
- 9 个 bootstrap provenance 和 manifest；
- 13 个 v1 SHA 前后是否一致；
- actual files 与白名单；
- network/credential/Git 写操作；
- ch3 仍冻结；
- 当前结果只是待独立审计的 implementation 声明。

本次 planning attempt-02 仍为只读；没有修改或创建文件，也没有运行实现测试。
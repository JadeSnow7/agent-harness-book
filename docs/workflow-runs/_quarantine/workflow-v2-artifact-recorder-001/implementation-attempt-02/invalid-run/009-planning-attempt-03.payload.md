```yaml
protocol_version: "1"
task_id: "workflow-v2-artifact-recorder-001"
role: "planning"
attempt: 3
status: "passed"

expected_result:
  observable_behavior:
    - "严格保护 workflow/v1 的 13 个现有文件：1 个 decision、5 个 workflow 根资产、7 个角色文件。"
    - "新增 17 个 v2 基础文件：1 个 decision、6 个 workflow 根资产、8 个角色文件、1 个工具、1 个测试。"
    - "v2 的角色文件恰好为 8 个：v1 的 7 个角色加 Artifact Recorder；controller_state、controller_correction 和 user_gate 是事件种类，不是额外角色。"
    - "保存 14 个真实 bootstrap 事件，每个事件由 1 个 payload 和 1 个 manifest 组成，共 28 个运行资产。"
    - "files_to_add 总数为 45：17 个基础文件加 28 个 bootstrap 文件；files_to_modify 为 0。"
    - "task-analysis attempt-01 原文以 status=needs_user_input 保存；不得改写为 passed。"
    - "planning attempt-01、attempt-02、attempt-03 均保存为独立事件；三者使用同一 artifact_key，并形成 01 → 02 → 03 的 revision_of 链。"
    - "planning 原始输出中的 status=passed 与 Controller 后续 returned/correction 事件分别保存；不得反向改写 planning 原文。"
    - "真实用户需求确认原文作为独立 user_gate_requirement/confirmed 事件保存。"
    - "实施批准、获批 Prompt、获批 task package 和 implementation 派发状态只在未来实际发生后记录；不得在当前计划阶段预写。"
    - "Recorder 从独立原始 UTF-8 payload 文件读取字节，并校验外部 expected SHA-256 和 byte length；复制粘贴文本不能冒充原始 bytes。"
    - "record/verify 拒绝路径逃逸、符号链接、非法任务根、sequence、attempt/revision、身份、摘要、UTF-8 或敏感信息错误。"
    - "Controller 复用同一 verify 子命令做确定性验证，不重复实现校验，也不判断业务内容。"
    - "Recorder 的终端产物是 payload/manifest 对和不含正文的机器结果，不递归记录 Recorder 自己的输出。"
  target_state: "workflow/v2 与 v1 并存，并拥有可运行、可离线测试、可解释的 Artifact Recorder；ch3 继续冻结。"

history_model:
  already_existed_before_attempt_03:
    - "001 task_analysis attempt-01 / needs_user_input"
    - "002 controller_state / awaiting_requirement_confirmation"
    - "003 user_gate_requirement / confirmed"
    - "004 controller_state / planning"
    - "005 planning attempt-01 / passed"
    - "006 controller_correction attempt-01 / returned"
    - "007 planning attempt-02 / passed"
    - "008 controller_correction attempt-02 / returned"
  produced_by_this_attempt_if_returned:
    - "009 planning attempt-03 / passed"
  produced_only_after_controller_accepts_attempt_03:
    - "010 controller_state / awaiting_implementation_approval"
  produced_only_after_future_user_approval_and_dispatch:
    - "011 user_gate_implementation / approved"
    - "012 approved_prompt / approved"
    - "013 task_package / approved"
    - "014 controller_state / implementation"
  prohibited_prewrite:
    - "implementation role result"
    - "post-implementation audit result"
    - "audit confirmation"
    - "summary"
    - "commit approval"
    - "commit"
    - "commit audit"
    - "acceptance"

scope:
  counts:
    v1_files_protected: 13
    v2_base_files_to_add: 17
    v2_role_files: 8
    bootstrap_events: 14
    bootstrap_pairs: 14
    bootstrap_files_to_add: 28
    total_files_to_add: 45
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
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/004-controller-state-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/004-controller-state-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/005-planning-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/005-planning-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/006-controller-correction-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/006-controller-correction-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/007-planning-attempt-02.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/007-planning-attempt-02.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/008-controller-correction-attempt-02.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/008-controller-correction-attempt-02.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/009-planning-attempt-03.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/009-planning-attempt-03.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/010-controller-state-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/010-controller-state-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/011-user-gate-implementation-attempt-01.payload.txt"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/011-user-gate-implementation-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/012-approved-prompt-attempt-01.payload.md"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/012-approved-prompt-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/013-task-package-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/013-task-package-attempt-01.manifest.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/014-controller-state-attempt-01.payload.yaml"
    - "docs/workflow-runs/workflow-v2-artifact-recorder-001/014-controller-state-attempt-01.manifest.yaml"
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
    - "不提供签名、远程时间戳或完备秘密识别。"
    - "不运行 Cargo、mdBook、真实网络、真实凭据或远程 CI。"
    - "不执行 Git 写操作或远程操作。"

impact:
  public_api_changes:
    - "新增 workflow protocol_version 2 的任务包、角色结果、状态机和 Artifact Recorder 契约；v1 原样保留。"
    - "新增 Python CLI：inspect、record、verify。"
    - "新增严格 JSON input descriptor 和 canonical JSON-as-YAML manifest schema。"
  dependency_changes:
    - "无；仅使用 Python 标准库。"
  side_effects:
    - "record 仅能在合法 task artifact root 下创建 payload/manifest 对和必要目录。"
    - "测试仅在 /tmp 下创建临时 fixture。"
    - "inspect 与 verify 只读。"
  risks:
    - "v1 是未跟踪用户资产，必须对恰好 13 个文件逐项比较 SHA-256。"
    - "平台若不能提供真实 raw payload，bootstrap 必须停止。"
    - "敏感扫描只能识别规定模式，不能证明没有所有秘密或无关私人内容。"
    - "双文件无法跨平台单事务提交，需要可恢复安装顺序和 orphan 规则。"
    - "planning 原文 status=passed 与 Controller returned 必须分别建模，否则会篡改历史。"
  known_limitations:
    - "manifest 无签名和外部可信时间。"
    - "manual_raw_export 依赖操作者 attestation。"
    - "Recorder 不做业务语义审计。"
    - "Recorder 输出不递归保存。"
    - "bootstrap 不能补造不存在的历史 snapshot。"
  leftovers:
    - "实施后仍需独立审计和其余人工门禁。"
    - "ch3 继续暂停。"
  later_milestones_touched: false

delegation:
  complexity: "high"
  coding_agents:
    - "单一 Luna 类 implementation coder"
  file_ownership:
    - "该 coder 独占全部 45 个 files_to_add；禁止第二个写 Agent 并发修改。"
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
    - "bootstrap verify 返回 artifact_count=14、last_sequence=14，运行目录恰好有 14 payload 和 14 manifest。"
    - "planning attempts 01、02、03 使用相同 artifact_key；attempt-01 revision_of=null，attempt-02 指向 005 manifest，attempt-03 指向 007 manifest。"
    - "task-analysis payload 保持 needs_user_input。"
    - "两个 planning correction 分别保存为 controller_correction/returned。"
    - "恰好 13 个 v1 SHA-256 前后相同。"
    - "新增文件恰好 45：17 个基础文件和 28 个 bootstrap 文件；修改既有文件数为 0。"
    - "角色文件恰好 8 个。"
    - "所有验证 network_used=false。"
  checks_not_required:
    - "Cargo/Rust 测试。"
    - "mdBook。"
    - "真实网络、凭据、远程 CI、push 或部署。"

implementation_prompt:
  status: "draft"
  full_text: |
    任务：workflow-v2-artifact-recorder-001

    你是本任务唯一的 Luna 类 implementation coder。只有收到本轮 approval_status: approved、用户批准原文、未被改写的 planning attempt-03 完整 Prompt，以及完整 bootstrap source bundle 后才能实施。

    一、目标和数量契约

    保持 workflow/v1 的 13 个现有文件字节不变，新建 breaking workflow/v2。v2 新增 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具。

    数量必须严格一致：

    - v1 保护文件：13；
    - v2 基础新增文件：17；
    - v2 角色文件：8，其中 7 个迁移角色加 1 个 Artifact Recorder；
    - bootstrap 历史/交接事件：14；
    - bootstrap pair：14，即 14 payload 加 14 manifest，共 28 文件；
    - files_to_add：45；
    - files_to_modify：0。

    controller_state、controller_correction、user_gate、approved_prompt 和 task_package 是 artifact event kind，不是新的逻辑角色，因此不改变“8 个角色文件”的计数。

    Recorder 只能机械保存冻结 UTF-8 bytes 和 metadata，不能总结、修正、补全、规范化或自动脱敏。Controller 必须复用同一工具的 verify 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 校验，不另写校验逻辑，也不判断内容正确性。

    payload 原样保存，不包装、不转换换行、不追加 newline。metadata 使用独立 .manifest.yaml，内容采用 Python 标准库生成的 canonical JSON；JSON 是 YAML 1.2 子集。

    Recorder 的 payload/manifest 对及不含正文的 JSON 命令结果是终端证据，不递归保存 Recorder 自身输出。

    二、事实基线

    repo:
    /Users/huaodong/Documents/ChatGPT/agent-harness-book

    HEAD:
    355a7d0d240e68f9e7ffc336e42913b609f5ac5e

    branch:
    main，ahead origin/main 4，工作树已脏，previous_commit=null。

    计划阶段 Python 为 /opt/homebrew/bin/python3，Python 3.14.6。

    必须保护的 13 个 v1 文件：

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

    开始和结束时均逐项计算这 13 个 SHA。任一不一致立即停止，不得更新基线。

    三、真实历史和 bootstrap source bundle

    实施前执行 pwd、git status --short --branch、git rev-parse HEAD、python3 --version，并确认 45 个目标文件均不存在。

    交接必须提供 /tmp 下的 source bundle，含 14 个独立 UTF-8 source 和 14 个 strict JSON descriptor。每个 descriptor 必须包含 source_path、expected_byte_length、expected_sha256、task_id、chapter、sequence、artifact_key、role、attempt、status、payload_extension、media_type、target_path、source_provenance、source_reference、identity_source、historical、recorded_by、revision_of。

    14 个事件固定如下：

    1. 001 task_analysis attempt-01，status=needs_user_input，artifact_key=task-analysis，revision_of=null。必须保存真实原文，不得改为 passed。
    2. 002 controller_state attempt-01，status=awaiting_requirement_confirmation。
    3. 003 user_gate_requirement attempt-01，status=confirmed，保存真实用户确认原文。
    4. 004 controller_state attempt-01，status=planning，表示需求确认后派发 planning attempt-01。
    5. 005 planning attempt-01，status 保持原始输出中的 passed，artifact_key=planning，revision_of=null。
    6. 006 controller_correction attempt-01，status=returned，artifact_key=planning-controller-correction，revision_of=null，保存因 13/14 计数矛盾退回的原始指令或真实 snapshot。
    7. 007 planning attempt-02，status 保持 passed，artifact_key=planning，revision_of 指向 005-planning-attempt-01.manifest.yaml。
    8. 008 controller_correction attempt-02，status=returned，artifact_key=planning-controller-correction，revision_of 指向 006-controller-correction-attempt-01.manifest.yaml，保存因历史/bootstrap 矛盾退回的原始指令或真实 snapshot。
    9. 009 planning attempt-03，status=passed，artifact_key=planning，revision_of 指向 007-planning-attempt-02.manifest.yaml，保存本次完整原始输出。
    10. 010 controller_state attempt-01，status=awaiting_implementation_approval。它只能在 Controller 完整性检查通过 attempt-03 后生成。
    11. 011 user_gate_implementation attempt-01，status=approved。它只能在用户未来明确批准后生成。
    12. 012 approved_prompt attempt-01，status=approved。内容必须是用户实际批准的 planning attempt-03 implementation Prompt。
    13. 013 task_package attempt-01，status=approved。它只能在批准后由实施交接真实生成。
    14. 014 controller_state attempt-01，status=implementation。它只能在实际派发 implementation 时生成。

    当前计划阶段不得创建 010 至 014。implementation coder 只有在这些未来事件真实发生并被 raw export 后，才可将 001 至 014 一起 bootstrap 入库。

    001 至 008 在 attempt-03 之前已发生；009 是本次结果；010 在 Controller 接受本次结果后发生；011 至 014 在用户批准和实施派发时发生。manifest 必须诚实保存 historical 和 source provenance。

    工具 role/status allowlist 必须支持：

    - task_analysis: passed、needs_user_input、blocked；
    - planning: passed、needs_user_input、blocked；
    - controller_correction: returned、rejected、correction_required；
    - controller_state: v2 状态机的合法状态，包括 planning、awaiting_requirement_confirmation、awaiting_implementation_approval、implementation；
    - user_gate_requirement: confirmed、rejected；
    - user_gate_implementation: approved、rejected；
    - approved_prompt: approved；
    - task_package: approved、snapshot；
    - 其他后续 v2 角色和门禁的实际状态。

    planning attempt-01、02、03 是同一 artifact_key 的三个 revisions。文件名、manifest attempt 和 revision_of 必须完全一致；不得把 attempt-03 放进 attempt-01 文件名。

    自动模式只接受 platform_raw_export 或 bootstrap_handoff。手动模式只接受 manual_raw_export，并要求 export_method 和明确 attestation。manual_copy、clipboard_copy、reconstructed_from_chat 一律拒绝。

    bundle 缺失、未来事件尚未发生、hash/length 不独立、含 symlink、身份不符或只能从聊天复制时，在仓库写入前停止。

    四、精确白名单

    17 个基础文件：

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

    28 个 bootstrap 文件是以下 14 个 payload 与各自同 stem manifest：

    - 001-task-analysis-attempt-01.payload.md
    - 002-controller-state-attempt-01.payload.md
    - 003-user-gate-requirement-attempt-01.payload.txt
    - 004-controller-state-attempt-01.payload.yaml
    - 005-planning-attempt-01.payload.md
    - 006-controller-correction-attempt-01.payload.md
    - 007-planning-attempt-02.payload.md
    - 008-controller-correction-attempt-02.payload.md
    - 009-planning-attempt-03.payload.md
    - 010-controller-state-attempt-01.payload.yaml
    - 011-user-gate-implementation-attempt-01.payload.txt
    - 012-approved-prompt-attempt-01.payload.md
    - 013-task-package-attempt-01.payload.yaml
    - 014-controller-state-attempt-01.payload.yaml

    它们全部位于 docs/workflow-runs/workflow-v2-artifact-recorder-001/。

    用 apply_patch 创建 17 个基础文件。28 个 bootstrap 文件只能由 Recorder 工具从 source bundle 写入。

    禁止修改任何既有文件，尤其是 v1、docs/chapters、tutorial、book、Cargo、crates、examples、.github 和 docs/report.md。

    五、文档和角色要求

    v2 decision 必须记录 breaking version、v1 并存、Recorder 最小写权限、raw export、bootstrap、先 record/verify 后推进、no-overwrite、revision、敏感 fail closed、Controller 确定性校验、Recorder 非递归终止、可恢复双文件顺序以及无签名/可信时间/完备 secret detection 的限制。

    README 必须索引完整 v2 资产，权限表列出恰好 8 个角色，并给出：

    raw export → Recorder record → Controller verify → state transition

    Controller 保留 v1 五个人工门禁和职责隔离。每个角色结果、Controller correction/state、用户门禁、task package 和 approved prompt 都先保存和 verify。source 不可用时 fail closed。Controller 不判断业务正确性，不递归记录 Recorder 结果。

    manual handoff 必须区分协作复制和 byte-level raw export。没有 raw/download export 时停止。

    task package 和 result 模板增加 artifact root、sequence、attempt、artifact key、raw source、expected hash/length、provenance、manifest、verification、bootstrap 和 approval artifact reference。

    artifact-input.template.json 展示 strict descriptor。

    8 个角色文件为 task-analysis、planning、implementation、post-implementation-audit、summary、commit、commit-audit、artifact-recorder。前 7 个完整保留 v1 权限和门禁。角色完整输出从 fenced YAML header 开始并保留自然语言。

    Artifact Recorder 只调用 inspect、record、verify，只写当前 artifact root，不碰业务文件和 Git，不回显 payload 或 secret。

    六、工具要求

    artifact_recorder.py 只使用标准库，提供：

    inspect --source
    record --repo-root --artifact-root --descriptor
    verify --repo-root --artifact-root --task-id --chapter

    必须实现：

    - 稳定 JSON 成功输出，不打印正文；
    - 安全错误 code/category/field/location，不回显值；
    - strict descriptor，拒绝未知字段；
    - source 为非 symlink 普通文件，严格 UTF-8，保存原 bytes；
    - 写前校验 expected hash/length；
    - 严格 task_id、artifact_key、role/status、sequence、attempt；
    - target 格式 NNN-<role-with-hyphens>-attempt-NN.payload.<ext>；
    - 合法 meta/chapter root，拒绝绝对路径、slash、.. 和 symlink escape；
    - manifest 唯一派生，使用 sort_keys=True、ensure_ascii=False、indent=2 加一个 LF 的 canonical JSON-as-YAML；
    - manifest 不保存 source 绝对路径；
    - payload_header 从开头 fenced YAML 提取 task_id、role、status；
    - external_descriptor 用于 Controller、gate、prompt 和 task package；
    - sequence 从 1 连续追加；
    - 同 artifact_key attempt 递增且 revision_of 指向上一 attempt manifest；
    - 不覆盖；重复成功记录返回 already_exists；
    - 同目录 temp、flush/fsync、no-overwrite 安装、普通失败清理；
    - 只允许精确匹配、无后续 sequence 的 payload orphan 恢复；
    - manifest orphan fail closed；
    - verify 只读检查 pair、sequence、canonical bytes、hash/length、identity、revision、symlink、未知和临时文件；
    - inspect 不写入且不打印正文；
    - 敏感扫描覆盖非占位凭据、常见 token/key、敏感环境变量、private-key 内容和 .env source，同时避免政策文字误报；
    - exit code 区分 success、schema/input、integrity/security；
    - 不联网、不读取任务外文件。

    七、测试矩阵

    使用 unittest、tempfile、subprocess/mock，fixture 位于 /tmp，覆盖：

    - Unicode、LF/CRLF、无末尾 newline exact bytes；
    - hash/length、invalid UTF-8；
    - 敏感拒绝、不回显和政策文字不误报；
    - meta/chapter root；
    - traversal、错误 target/root/chapter；
    - source/root/payload/manifest symlink；
    - descriptor schema/provenance/attestation；
    - task/role/status/header identity；
    - sequence gap/duplicate/backward；
    - planning 01→02→03 revision 链；
    - controller correction 01→02 revision 链；
    - no-overwrite；
    - canonical manifest、tamper、orphan、unknown/temp；
    - orphan recovery；
    - 第二步安装失败清理；
    - inspect/verify 只读；
    - CLI 不输出正文。

    八、bootstrap 与验证

    工具通过 py_compile 和 unittest 后，按 001 至 014 依次 record。每次后运行 verify。最终必须：

    - artifact_count=14；
    - last_sequence=14；
    - 恰好 14 payload 加 14 manifest；
    - task-analysis status 仍为 needs_user_input；
    - planning attempts 为 01、02、03 且 revision 链正确；
    - controller correction attempts 为 01、02；
    - 不存在 015-implementation 或任何后置结果。

    执行：

    python3 -m py_compile docs/prompts/workflow/v2/tools/artifact_recorder.py docs/prompts/workflow/v2/tests/test_artifact_recorder.py

    python3 -m unittest discover -s docs/prompts/workflow/v2/tests -p 'test_*.py' -v

    python3 docs/prompts/workflow/v2/tools/artifact_recorder.py --help

    python3 docs/prompts/workflow/v2/tools/artifact_recorder.py verify --repo-root /Users/huaodong/Documents/ChatGPT/agent-harness-book --artifact-root docs/workflow-runs/workflow-v2-artifact-recorder-001 --task-id workflow-v2-artifact-recorder-001 --chapter meta

    shasum -a 256 后跟恰好 13 个显式 v1 路径。

    git diff --check

    git status --short

    不运行 Cargo、mdBook、网络、凭据、远程 CI 或部署。

    九、停止条件

    缺少批准、Prompt 被改写、13 个 v1 SHA 不符、45 个目标已有未知内容、14 个 source 不完整、未来事件未实际发生、raw/hash/identity/revision 不成立、需要范围外修改或 Git/网络/凭据时立即停止。

    禁止覆盖、自动脱敏、补造历史、修改 v1/ch3、预写 implementation/audit/summary/commit、Git 写、远程操作、reset/revert 或清理用户工作树。

    十、完成报告

    返回 implementation 结构化结果和仓库规定的十一项报告，并单列：

    - 新增是否恰好 45=17+28；
    - 角色是否恰好 8；
    - bootstrap 是否恰好 14 pairs；
    - task-analysis needs_user_input 是否保留；
    - planning 01/02/03 和 correction 01/02 是否保留且 revision 正确；
    - 13 个 v1 SHA 前后是否一致；
    - actual files、network、credential、Git 写情况；
    - ch3 仍冻结；
    - 当前只是待独立审计的 implementation 声明。

requires_user_confirmation: true
requested_confirmation: "请审阅 planning attempt-03 和完整编码 Prompt；只有明确回复‘批准实施’后，未来的 010 至 014 事件才可按真实发生顺序生成并进入 bootstrap source bundle。"
```

完整编码 Prompt如下，与结构化字段中的正文一致。

---

# workflow-v2-artifact-recorder-001 实施 Prompt

你是本任务唯一的 Luna 类 implementation coder。只有收到本轮 `approval_status: approved`、用户批准原文、未被改写的 planning attempt-03 完整 Prompt，以及完整 bootstrap source bundle 后才能实施。

## 一、目标和数量契约

保持 workflow/v1 的 13 个现有文件字节不变，新建 breaking workflow/v2。v2 新增 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具。

数量必须严格一致：

- v1 保护文件：13；
- v2 基础新增文件：17；
- v2 角色文件：8，其中 7 个迁移角色加 1 个 Artifact Recorder；
- bootstrap 历史/交接事件：14；
- bootstrap pair：14，即 14 payload 加 14 manifest，共 28 文件；
- `files_to_add`：45；
- `files_to_modify`：0。

`controller_state`、`controller_correction`、`user_gate`、`approved_prompt` 和 `task_package` 是 artifact event kind，不是新的逻辑角色，因此不改变“8 个角色文件”的计数。

Recorder 只能机械保存冻结 UTF-8 bytes 和 metadata，不能总结、修正、补全、规范化或自动脱敏。Controller 必须复用同一工具的 `verify` 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 校验，不另写校验逻辑，也不判断内容正确性。

payload 原样保存，不包装、不转换换行、不追加 newline。metadata 使用独立 `.manifest.yaml`，内容采用 Python 标准库生成的 canonical JSON；JSON 是 YAML 1.2 子集。

Recorder 的 payload/manifest 对及不含正文的 JSON 命令结果是终端证据，不递归保存 Recorder 自身输出。

## 二、事实基线

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

必须保护的 13 个 v1 文件：

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

开始和结束时均逐项计算这 13 个 SHA。任一不一致立即停止，不得更新基线。

## 三、真实历史和 bootstrap source bundle

实施前执行：

```bash
pwd
git status --short --branch
git rev-parse HEAD
python3 --version
```

确认 45 个目标文件均不存在。

交接必须提供 `/tmp` 下的 source bundle，包含 14 个独立 UTF-8 source 和 14 个 strict JSON descriptor。每个 descriptor 必须包含：

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

14 个事件固定如下：

1. `001 task_analysis attempt-01`：`status=needs_user_input`，`artifact_key=task-analysis`，`revision_of=null`。必须保存真实原文，不得改为 passed。
2. `002 controller_state attempt-01`：`status=awaiting_requirement_confirmation`。
3. `003 user_gate_requirement attempt-01`：`status=confirmed`，保存真实用户确认原文。
4. `004 controller_state attempt-01`：`status=planning`，表示需求确认后派发 planning attempt-01。
5. `005 planning attempt-01`：status 保持原始输出中的 `passed`，`artifact_key=planning`，`revision_of=null`。
6. `006 controller_correction attempt-01`：`status=returned`，`artifact_key=planning-controller-correction`，`revision_of=null`，保存因 13/14 计数矛盾退回的原始指令或真实 snapshot。
7. `007 planning attempt-02`：status 保持 `passed`，`artifact_key=planning`，`revision_of` 指向 `005-planning-attempt-01.manifest.yaml`。
8. `008 controller_correction attempt-02`：`status=returned`，`artifact_key=planning-controller-correction`，`revision_of` 指向 `006-controller-correction-attempt-01.manifest.yaml`，保存因历史/bootstrap 矛盾退回的原始指令或真实 snapshot。
9. `009 planning attempt-03`：`status=passed`，`artifact_key=planning`，`revision_of` 指向 `007-planning-attempt-02.manifest.yaml`，保存本次完整原始输出。
10. `010 controller_state attempt-01`：`status=awaiting_implementation_approval`。只能在 Controller 完整性检查通过 attempt-03 后生成。
11. `011 user_gate_implementation attempt-01`：`status=approved`。只能在用户未来明确批准后生成。
12. `012 approved_prompt attempt-01`：`status=approved`。内容必须是用户实际批准的 planning attempt-03 implementation Prompt。
13. `013 task_package attempt-01`：`status=approved`。只能在批准后由实施交接真实生成。
14. `014 controller_state attempt-01`：`status=implementation`。只能在实际派发 implementation 时生成。

当前计划阶段不得创建 010 至 014。implementation coder 只有在这些未来事件真实发生并被 raw export 后，才可将 001 至 014 一起 bootstrap 入库。

001 至 008 在 attempt-03 之前已发生；009 是本次结果；010 在 Controller 接受本次结果后发生；011 至 014 在用户批准和实施派发时发生。manifest 必须诚实保存 `historical` 和 source provenance。

工具 role/status allowlist 必须支持：

- `task_analysis`：passed、needs_user_input、blocked；
- `planning`：passed、needs_user_input、blocked；
- `controller_correction`：returned、rejected、correction_required；
- `controller_state`：v2 状态机合法状态，包括 planning、awaiting_requirement_confirmation、awaiting_implementation_approval、implementation；
- `user_gate_requirement`：confirmed、rejected；
- `user_gate_implementation`：approved、rejected；
- `approved_prompt`：approved；
- `task_package`：approved、snapshot；
- 其他后续 v2 角色和门禁的实际状态。

planning attempt-01、02、03 是同一 `artifact_key` 的三个 revisions。文件名、manifest attempt 和 `revision_of` 必须完全一致；不得把 attempt-03 放进 attempt-01 文件名。

自动模式只接受 `platform_raw_export` 或 `bootstrap_handoff`。手动模式只接受 `manual_raw_export`，并要求 `export_method` 和明确 attestation。`manual_copy`、`clipboard_copy`、`reconstructed_from_chat` 一律拒绝。

bundle 缺失、未来事件尚未发生、hash/length 不独立、含 symlink、身份不符或只能从聊天复制时，在仓库写入前停止。

## 四、精确白名单

17 个基础文件：

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

28 个 bootstrap 文件是以下 14 个 payload 与各自同 stem manifest：

- `001-task-analysis-attempt-01.payload.md`
- `002-controller-state-attempt-01.payload.md`
- `003-user-gate-requirement-attempt-01.payload.txt`
- `004-controller-state-attempt-01.payload.yaml`
- `005-planning-attempt-01.payload.md`
- `006-controller-correction-attempt-01.payload.md`
- `007-planning-attempt-02.payload.md`
- `008-controller-correction-attempt-02.payload.md`
- `009-planning-attempt-03.payload.md`
- `010-controller-state-attempt-01.payload.yaml`
- `011-user-gate-implementation-attempt-01.payload.txt`
- `012-approved-prompt-attempt-01.payload.md`
- `013-task-package-attempt-01.payload.yaml`
- `014-controller-state-attempt-01.payload.yaml`

它们全部位于：

```text
docs/workflow-runs/workflow-v2-artifact-recorder-001/
```

用 `apply_patch` 创建 17 个基础文件。28 个 bootstrap 文件只能由 Recorder 工具从 source bundle 写入。

禁止修改任何既有文件，尤其是 v1、`docs/chapters`、`tutorial`、`book`、Cargo、`crates`、`examples`、`.github` 和 `docs/report.md`。

## 五、文档和角色要求

v2 decision 必须记录 breaking version、v1 并存、Recorder 最小写权限、raw export、bootstrap、先 record/verify 后推进、no-overwrite、revision、敏感 fail closed、Controller 确定性校验、Recorder 非递归终止、可恢复双文件顺序，以及无签名、可信时间和完备 secret detection 的限制。

README 必须索引完整 v2 资产，权限表列出恰好 8 个角色，并给出：

```text
raw export → Recorder record → Controller verify → state transition
```

Controller 保留 v1 五个人工门禁和职责隔离。每个角色结果、Controller correction/state、用户门禁、task package 和 approved prompt 都先保存和 verify。source 不可用时 fail closed。Controller 不判断业务正确性，不递归记录 Recorder 结果。

manual handoff 必须区分协作复制和 byte-level raw export。没有 raw/download export 时停止。

task package 和 result 模板增加 artifact root、sequence、attempt、artifact key、raw source、expected hash/length、provenance、manifest、verification、bootstrap 和 approval artifact reference。

`artifact-input.template.json` 展示 strict descriptor。

8 个角色文件为：

- task-analysis
- planning
- implementation
- post-implementation-audit
- summary
- commit
- commit-audit
- artifact-recorder

前 7 个完整保留 v1 权限和门禁。角色完整输出从 fenced YAML header 开始并保留自然语言。

Artifact Recorder 只调用 inspect、record、verify，只写当前 artifact root，不碰业务文件和 Git，不回显 payload 或 secret。

## 六、工具要求

`artifact_recorder.py` 只使用标准库，提供：

```text
inspect --source
record --repo-root --artifact-root --descriptor
verify --repo-root --artifact-root --task-id --chapter
```

必须实现：

- 稳定 JSON 成功输出，不打印正文；
- 安全错误 code/category/field/location，不回显值；
- strict descriptor，拒绝未知字段；
- source 为非 symlink 普通文件，严格 UTF-8，保存原 bytes；
- 写前校验 expected hash/length；
- 严格 task_id、artifact_key、role/status、sequence、attempt；
- target 格式 `NNN-<role-with-hyphens>-attempt-NN.payload.<ext>`；
- 合法 meta/chapter root，拒绝绝对路径、slash、`..` 和 symlink escape；
- manifest 唯一派生，使用 `sort_keys=True`、`ensure_ascii=False`、`indent=2` 加一个 LF 的 canonical JSON-as-YAML；
- manifest 不保存 source 绝对路径；
- `payload_header` 从开头 fenced YAML 提取 task_id、role、status；
- `external_descriptor` 用于 Controller、gate、prompt 和 task package；
- sequence 从 1 连续追加；
- 同 artifact_key attempt 递增且 revision_of 指向上一 attempt manifest；
- 不覆盖；重复成功记录返回 `already_exists`；
- 同目录 temp、flush/fsync、no-overwrite 安装、普通失败清理；
- 只允许精确匹配、无后续 sequence 的 payload orphan 恢复；
- manifest orphan fail closed；
- verify 只读检查 pair、sequence、canonical bytes、hash/length、identity、revision、symlink、未知和临时文件；
- inspect 不写入且不打印正文；
- 敏感扫描覆盖非占位凭据、常见 token/key、敏感环境变量、private-key 内容和 `.env` source，同时避免政策文字误报；
- exit code 区分 success、schema/input、integrity/security；
- 不联网、不读取任务外文件。

## 七、测试矩阵

使用 `unittest`、`tempfile`、`subprocess/mock`，fixture 位于 `/tmp`，覆盖：

- Unicode、LF/CRLF、无末尾 newline exact bytes；
- hash/length、invalid UTF-8；
- 敏感拒绝、不回显和政策文字不误报；
- meta/chapter root；
- traversal、错误 target/root/chapter；
- source/root/payload/manifest symlink；
- descriptor schema/provenance/attestation；
- task/role/status/header identity；
- sequence gap/duplicate/backward；
- planning 01→02→03 revision 链；
- controller correction 01→02 revision 链；
- no-overwrite；
- canonical manifest、tamper、orphan、unknown/temp；
- orphan recovery；
- 第二步安装失败清理；
- inspect/verify 只读；
- CLI 不输出正文。

## 八、bootstrap 与验证

工具通过 `py_compile` 和 `unittest` 后，按 001 至 014 依次 `record`。每次后运行 `verify`。最终必须：

- `artifact_count=14`；
- `last_sequence=14`；
- 恰好 14 payload 加 14 manifest；
- task-analysis status 仍为 `needs_user_input`；
- planning attempts 为 01、02、03 且 revision 链正确；
- controller correction attempts 为 01、02；
- 不存在 015-implementation 或任何后置结果。

执行：

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

不运行 Cargo、mdBook、网络、凭据、远程 CI 或部署。

## 九、停止条件

缺少批准、Prompt 被改写、13 个 v1 SHA 不符、45 个目标已有未知内容、14 个 source 不完整、未来事件未实际发生、raw/hash/identity/revision 不成立、需要范围外修改或 Git/网络/凭据时立即停止。

禁止覆盖、自动脱敏、补造历史、修改 v1/ch3、预写 implementation/audit/summary/commit、Git 写、远程操作、reset/revert 或清理用户工作树。

## 十、完成报告

返回 implementation 结构化结果和仓库规定的十一项报告，并单列：

- 新增是否恰好 `45=17+28`；
- 角色是否恰好 8；
- bootstrap 是否恰好 14 pairs；
- task-analysis `needs_user_input` 是否保留；
- planning 01/02/03 和 correction 01/02 是否保留且 revision 正确；
- 13 个 v1 SHA 前后是否一致；
- actual files、network、credential、Git 写情况；
- ch3 仍冻结；
- 当前只是待独立审计的 implementation 声明。

本次 planning attempt-03 保持只读；没有修改或创建文件，也没有运行实现测试。
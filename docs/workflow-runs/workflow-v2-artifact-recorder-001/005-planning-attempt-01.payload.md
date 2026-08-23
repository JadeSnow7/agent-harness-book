```yaml
protocol_version: "1"
task_id: "workflow-v2-artifact-recorder-001"
role: "planning"
status: "passed"

expected_result:
  observable_behavior:
    - "保留 docs/prompts/workflow/v1/ 与 reader-ai-coding-workflow-v1.md 的现有字节内容，不修改 v1。"
    - "新增独立、breaking 的 workflow/v2 决策、Controller、手动交接、任务包、结果模板和八类角色资产。"
    - "新增仅依赖 Python 标准库的确定性 Artifact Recorder CLI，提供 inspect、record、verify 子命令。"
    - "Recorder 从独立原始 UTF-8 payload 文件读取字节，以外部 expected SHA-256 和 byte length 校验，不从会话复制文本冒充原始字节。"
    - "每个 payload 以 sequence、role、attempt 命名，并配对独立 .manifest.yaml；已有文件不可覆盖。"
    - "record 拒绝路径逃逸、符号链接、非法任务根、序号错误、attempt/revision 错误、身份不匹配、摘要不匹配、非法 UTF-8 和确定性敏感信息命中。"
    - "verify 复用 Recorder 的同一实现，重新检查路径、配对、序号、manifest canonical bytes、长度、哈希、身份、revision 链和敏感信息；Controller 不另写一套校验逻辑。"
    - "自动模式只接受平台原始导出；手动模式只接受受控 raw export 和明确 provenance/attestation；缺少原始文件时 fail closed。"
    - "bootstrap coder 只保存本任务实施前已经形成或实施交接时真实生成的资产，并在 manifest 中标明 bootstrap-coder 与历史属性；不预写 implementation、audit、summary、commit 或 commit-audit 结果。"
    - "v2 建立后，每次角色结果和人工门禁必须先由 Recorder 保存并经 Controller 调用 verify 成功，状态机才可继续。"
    - "Recorder 的终端产物是 payload/manifest 对和不含正文的机器结果，不递归保存 Recorder 自己的输出。"
  target_state: "workflow/v2 作为与 v1 并存的可运行、可离线测试、可解释的长期资产协议存在；ch3 继续冻结，尚未实施或追认。"

scope:
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
    - "不证明 payload 的作者身份或提供签名、远程时间戳、透明日志或防恶意管理员能力。"
    - "不默认联网，不使用真实凭据。"
    - "不运行 Cargo、mdBook 或章节工程测试。"
    - "不执行任何 Git 写操作或远程操作。"

impact:
  public_api_changes:
    - "新增 workflow protocol_version 2 的任务包、角色结果和状态机字段；v1 API 原样保留。"
    - "新增 Python CLI：artifact_recorder.py inspect、record、verify。"
    - "新增严格 JSON input descriptor 和 canonical JSON-as-YAML manifest schema。"
  dependency_changes:
    - "无；仅使用 Python 标准库。"
  side_effects:
    - "record 仅能在合法 task artifact root 下创建 payload/manifest 对和必要目录。"
    - "测试仅在 /tmp 下创建临时 fixture，并由 unittest 清理。"
    - "inspect 与 verify 必须只读。"
  risks:
    - "仓库当前脏且 v1 为未跟踪用户资产，常规 git diff 无法保护它，必须逐文件 SHA-256 前后比对。"
    - "平台可能无法提供真正的原始消息 payload 文件；这种情况下 bootstrap 必须停止，不能用复制后的文本替代。"
    - "确定性敏感扫描只能识别规定模式，不能证明不存在所有秘密或无关私人内容。"
    - "两个最终文件无法获得跨平台单事务原子性；实现必须采用同目录临时文件、no-overwrite 安装、常规失败清理和可验证的 orphan 恢复规则。"
    - "严格扫描可能产生误报；误报只能由上游生成新的安全 payload 或新的获批计划处理，Recorder 不自动修改正文。"
  known_limitations:
    - "manifest 不提供密码学签名或外部可信时间。"
    - "manual_raw_export 的真实性依赖操作者使用平台 raw export 并作明确 attestation；复制粘贴不被接受。"
    - "Recorder 不做业务语义审计；语义正确性仍由既有 post-implementation-audit 等角色负责。"
    - "Recorder 自己的命令结果不递归保存；配对 manifest 和 Controller verify 结果构成其终端证据。"
    - "bootstrap 只能保存实际可获得的原始前置输出和实施交接时真实生成的 snapshot，不能补造不存在的历史 snapshot。"
  leftovers:
    - "实现完成后仍需独立执行后审计、用户审计确认、汇总、提交批准、提交审核和最终验收。"
    - "ch3 任务继续暂停；v2 最终验收后才可从 ch3 计划阶段重新开始。"
  later_milestones_touched: false

delegation:
  complexity: "high"
  coding_agents:
    - "单一 Luna 类 implementation coder"
  file_ownership:
    - "该 coder 独占本计划全部 files_to_add；禁止第二个写 Agent 并发修改。"
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
    - "shasum -a 256 <显式 v1 文件清单>"
    - "git diff --check"
    - "git status --short"
  expected_evidence:
    - "Python 编译与 unittest 的真实退出码。"
    - "测试矩阵覆盖 exact bytes、hash/length、UTF-8、敏感信息、路径和 symlink、no-overwrite、sequence、attempt/revision、identity、pair recovery 与 verify 只读性。"
    - "bootstrap run verify 返回成功，artifact_count=9，last_sequence=9。"
    - "v1 的十四个 SHA-256 与计划记录完全相同。"
    - "实际变更仅包含 files_to_add，无既有文件被修改。"
    - "所有命令 network_used=false。"
  checks_not_required:
    - "Cargo 和 Rust 测试：未修改 Rust/Cargo。"
    - "mdBook：未修改 book/。"
    - "真实网络、API Key、远程 CI、push、部署。"

implementation_prompt:
  status: "draft"
  full_text: |
    任务：workflow-v2-artifact-recorder-001

    你是本任务唯一的 Luna 类 implementation coder。你只能执行下面已经获得本轮用户明确“批准实施”的 Prompt；未收到 approval_status: approved、批准原文、完整 bootstrap source bundle 和本 Prompt 原文时必须停止。

    一、目标与已确认业务逻辑

    在保留 workflow/v1 完全不变的前提下，新建 breaking workflow/v2。v2 增加一个最小写权限的 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具，用于保存冻结的角色完整输出、自然语言正文、人工门禁原文、任务包、获批 Prompt、Controller 状态、implementation/audit/summary/commit/commit-audit 结果及配对 manifest。

    Recorder 只能机械保存已冻结的 UTF-8 bytes 和元数据，不能总结、修正、补全、规范化或自动脱敏，不能修改业务文件。v2 不增加 Artifact Audit；Controller 必须复用同一工具的 verify 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 的确定性验证，不判断业务内容。

    章节任务根目录固定为：

    docs/chapters/chXX/workflow/<task_id>/

    非章节/meta 工作流任务根目录固定为：

    docs/workflow-runs/<task_id>/

    本任务属于 meta，task_id 为 workflow-v2-artifact-recorder-001。

    raw payload 独立保存，不添加 Markdown 包装、不做换行转换、不添加末尾换行。metadata 保存为独立 .manifest.yaml。manifest 使用 Python 标准库生成的 canonical JSON 文本；JSON 是 YAML 1.2 的有效子集，因此无需引入 YAML 依赖。纠错必须使用新 sequence 和递增 attempt/revision，禁止覆盖旧资产。

    敏感信息、路径或校验不匹配必须 fail closed 并阻止状态推进。bootstrap coder 只能保存真实可取得的前置资产并标记 recorded_by=bootstrap-coder，不能伪装成历史 Recorder，也不能预写 implementation、audit、summary、commit 或 commit-audit 结果。v2 建立后，后置结果由 Artifact Recorder 角色保存。

    Recorder 自己的终端结果是 payload/manifest 对和不含 payload 正文的 JSON 命令结果，不再递归记录 Recorder 输出。这是明确的递归终止边界。

    二、事实基线

    工程根目录：

    /Users/huaodong/Documents/ChatGPT/agent-harness-book

    计划阶段确认：

    - HEAD: 355a7d0d240e68f9e7ffc336e42913b609f5ac5e
    - branch: main
    - main 相对 origin/main ahead 4
    - previous_commit: null
    - 工作树已脏，包含大量用户修改和未跟踪资产
    - Python: /opt/homebrew/bin/python3，Python 3.14.6
    - Cargo 可用但本任务不需要运行
    - docs/decisions/reader-ai-coding-workflow-v1.md 与 docs/prompts/workflow/v1/ 当前未跟踪，属于用户既有资产

    v1 前置保护 SHA-256：

    - 5ed492e0bc21d9ddc4e910401bb79258c3dd1c7da5fcbf6306ee208e5738c52a  docs/decisions/reader-ai-coding-workflow-v1.md
    - 55d1fc1ca1aa52a6f6db89ab952e18ca3ad6fb319ec2619ff2c891376db522c4  docs/prompts/workflow/v1/README.md
    - bdb91956c17e98688e2b382e616417e75cd73f89bb0a907d4ea848e4e9db94f6  docs/prompts/workflow/v1/controller.md
    - 64918b4c5895c86ad55dcc788eaaa1ca8f2f362a3ed41fa033bfc31aff0a0a09  docs/prompts/workflow/v1/manual-handoff.md
    - c1991453be55040fc6d82f041c272fda468569650676113535eac2881bda5aa1  docs/prompts/workflow/v1/agent-result.template.yaml
    - 584ec93f89c8c83bd8ec1ded6a3829988123856e68d9de8a7aa85560a30fb21b  docs/prompts/workflow/v1/task-package.template.yaml
    - 6d911bfa1a1bc6b88b3a70e9bcb549ecef9a1e9a54a9a56450039e50d3136cdc  docs/prompts/workflow/v1/roles/task-analysis.md
    - 7c09885ad499cde7bc693a9866b0da89d819fa2758361c41965a53cf7ebfb955  docs/prompts/workflow/v1/roles/planning.md
    - f0dc3e5f6f99a512311f6da064b09779fda35465da9bfb89346002f149d18a30  docs/prompts/workflow/v1/roles/implementation.md
    - 0dbb752e55daf74adbc58f47b92e744332809f83481c2342b07775b16034b43e  docs/prompts/workflow/v1/roles/post-implementation-audit.md
    - 5fe21f781731ad0d89598c37ec8e758da8fe584ff3a0aea2d14c29eadb5014e0  docs/prompts/workflow/v1/roles/summary.md
    - 0122acd806a5cdefe57f8dddd8fe354d14c29eadb5014e0  docs/prompts/workflow/v1/roles/commit.md
    - cd7113da6430594225acfa626ac109a093cde46b71e3b039f1cfa78e251034d0  docs/prompts/workflow/v1/roles/commit-audit.md

    注意：上一清单中 commit.md 的完整正确 SHA-256 必须以如下值为准：

    0122acd806a5cdefe57f8dddd8fe354d14f8d0ff2135298277898d25d4a3c2fc

    实施开始时必须重新计算全部十四个值；任一不一致立即停止，不得“更新基线”。

    三、实施前检查和 bootstrap 输入门禁

    先执行并记录：

    - pwd
    - git status --short --branch
    - git rev-parse HEAD
    - python3 --version
    - 上述十四个 v1 文件的显式 shasum -a 256

    确认所有允许新增文件尚不存在。若任一目标文件已经存在或包含未知内容，停止并报告，不覆盖。

    Controller/操作者必须随实施交接提供一个位于 /tmp 下的 bootstrap source bundle。它必须包含九个独立 UTF-8 source 文件及九个严格 JSON descriptor；descriptor 必须给出 source_path、expected_byte_length、expected_sha256、task_id、chapter、sequence、artifact_key、role、attempt、status、payload_extension、media_type、target_path、source_provenance、source_reference、identity_source、historical、recorded_by 和 revision_of。

    九个逻辑事件固定为：

    1. task_analysis / passed / payload_header / 原始完整角色输出；
    2. controller_state / awaiting_requirement_confirmation / external_descriptor / 原始 Controller 响应或真实状态 snapshot；
    3. user_gate_requirement / confirmed / external_descriptor / 用户门禁原文；
    4. planning / passed / payload_header / 本计划角色完整原始输出；
    5. controller_state / awaiting_implementation_approval / external_descriptor / 原始 Controller 响应或真实状态 snapshot；
    6. user_gate_implementation / approved / external_descriptor / 本轮批准实施原文；
    7. approved_prompt / approved / external_descriptor / 本 Prompt 的冻结批准文本；
    8. task_package / approved / external_descriptor / 实施交接时真实生成的获批任务包；
    9. controller_state / implementation / external_descriptor / 派发 implementation 时真实生成的状态 snapshot。

    这些文件的 target_path 必须分别对应 scope 白名单中的 001 至 009 文件。historical 必须真实填写：过去原始输出可以是 true；实施交接时才生成的 task package 或 snapshot 必须是 false。recorded_by 必须是 bootstrap-coder。不得补造不存在的早期 task package 或状态快照。

    自动模式只接受 source_provenance=platform_raw_export 或 bootstrap_handoff，并要求 expected hash/length 来自平台/交接元数据。手动模式只接受 manual_raw_export，descriptor 还必须包含非空 export_method 和固定 attestation，声明使用平台 raw/download export、没有使用复制粘贴、导出后未修改。manual_copy、clipboard_copy、reconstructed_from_chat 一律拒绝。

    若 bundle 缺失、数量不对、使用 symlink、没有独立 expected hash/length、无法证明 raw export，或只能从对话复制重建，必须在任何仓库写入前停止。不得把复制后的文本标成原始 bytes。

    四、精确文件白名单

    只允许新增：

    - docs/decisions/reader-ai-coding-workflow-v2.md
    - docs/prompts/workflow/v2/README.md
    - docs/prompts/workflow/v2/controller.md
    - docs/prompts/workflow/v2/manual-handoff.md
    - docs/prompts/workflow/v2/task-package.template.yaml
    - docs/prompts/workflow/v2/agent-result.template.yaml
    - docs/prompts/workflow/v2/artifact-input.template.json
    - docs/prompts/workflow/v2/roles/task-analysis.md
    - docs/prompts/workflow/v2/roles/planning.md
    - docs/prompts/workflow/v2/roles/implementation.md
    - docs/prompts/workflow/v2/roles/post-implementation-audit.md
    - docs/prompts/workflow/v2/roles/summary.md
    - docs/prompts/workflow/v2/roles/commit.md
    - docs/prompts/workflow/v2/roles/commit-audit.md
    - docs/prompts/workflow/v2/roles/artifact-recorder.md
    - docs/prompts/workflow/v2/tools/artifact_recorder.py
    - docs/prompts/workflow/v2/tests/test_artifact_recorder.py
    - scope 中明确列出的九个 bootstrap payload 和九个 manifest

    不修改任何既有文件。禁止范围包括：

    - docs/decisions/reader-ai-coding-workflow-v1.md
    - docs/prompts/workflow/v1/**
    - docs/chapters/**
    - tutorial/**
    - book/**
    - Cargo.toml
    - Cargo.lock
    - crates/**
    - examples/**
    - .github/**
    - docs/report.md
    - 其他未明确列出的文件

    使用 apply_patch 创建决策、Prompt、模板、工具和测试。bootstrap payload/manifest 必须通过新 Recorder 工具从 source bundle 写入，不能用 apply_patch 手工复制。

    五、文件级实现要求

    1. reader-ai-coding-workflow-v2.md

    写明 v2 是 breaking change，与 v1 并存，v1 不被废弃或改写。记录：

    - 新增 Artifact Recorder 的原因；
    - 写权限最小化；
    - 先记录并 verify、后状态推进；
    - 自动/手动原始 payload 边界；
    - bootstrap 例外和诚实 provenance；
    - manifest 与 no-overwrite 规则；
    - 敏感信息 fail closed；
    - Controller 只做确定性校验，不替代语义审计；
    - Recorder 输出不递归记录的终止规则；
    - 两文件不是跨平台单事务原子提交，但采用可恢复顺序；
    - 没有签名、外部时间戳和完备 secret detection 的限制。

    2. workflow/v2/README.md

    索引全部 v2 资产，给出完整状态顺序。每个角色结果和人工门禁后插入：

    raw export → Artifact Recorder record → Controller verify → 状态推进

    权限表中新增 Recorder：只允许写当前 task artifact root，Git 只读且禁止提交。说明 chapter/meta 根路径、命名、attempt/revision、bootstrap、递归终止和离线使用方式。

    3. controller.md

    保留 v1 的职责隔离与五个人工门禁，但升级为 protocol_version 2。Controller 仍只读，不保存文件。它必须：

    - 为每个角色输出、用户门禁、task package、approved prompt 和状态 snapshot 获得 raw payload source；
    - 派发 Artifact Recorder；
    - Recorder 返回后调用同一 artifact_recorder.py verify；
    - verify 成功前不改变 current_state；
    - source 缺失或不是真实 raw export 时进入 blocked/needs_clarification；
    - 自动模式说明平台 raw export 证据；
    - 手动模式要求受控导出；
    - 不判断 payload 技术正确性；
    - 不把 Recorder 的结果递归成新记录任务；
    - implementation 完成后先记录 implementation 原始结果，再派发 audit；
    - audit、用户 audit gate、summary、commit gate、commit、commit-audit、acceptance gate 同样记录；
    - 修复仍必须回到 planning 并重新批准。

    Controller 状态摘要新增 artifact 字段，例如 last_recorded_sequence、last_manifest、verification_status、source_provenance，但不得塞入 payload 正文或秘密。

    4. manual-handoff.md

    明确复制粘贴结构化结果只够角色协作，不够长期 raw-byte 证据。长期记录必须使用平台 download/export raw response 或等价无转换导出。规定：

    - raw export 文件；
    - inspect 获得 hash/length且不打印正文；
    - descriptor；
    - manifest；
    - 操作者 attestation；
    - 不能导出时 fail closed；
    - 不得把 clipboard/copy 后内容标成 original；
    - 新会话只能承担一个角色；
    - 任何敏感信息命中都返回上游重新生成安全结果。

    5. task-package.template.yaml 与 agent-result.template.yaml

    升级 protocol_version 2，保留 v1 核心字段，增加：

    - artifact_root
    - next_sequence
    - attempt
    - artifact_key
    - raw_payload_source
    - expected_byte_length
    - expected_sha256
    - source_provenance
    - manifest_path
    - artifact_verification
    - bootstrap 标记
    - approval evidence artifact reference

    模板不得内嵌真实凭据，不得把未来结果预填为 passed。

    6. artifact-input.template.json

    给出严格 descriptor 示例。工具必须拒绝未知字段。至少包含：

    schema_version、task_id、chapter、sequence、artifact_key、role、attempt、status、payload_extension、media_type、source_path、expected_byte_length、expected_sha256、target_path、source_provenance、source_reference、identity_source、historical、recorded_by、revision_of、export_method、manual_export_attestation。

    chapter 为 meta 时使用 null；文件名将 role 中下划线转换为连字符。不要在示例中放疑似真实 secret。

    7. 七个既有角色的 v2 版本

    将 v1 权限、人工门禁、停止条件和十一项报告要求完整迁移到 v2，不弱化原规则。每个角色：

    - 输出必须从第一个字节开始包含 fenced YAML 结构头；
    - 结构头至少有 protocol_version、task_id、role、status；
    - fenced YAML 后保留完整自然语言；
    - 返回后不自行落盘，由 Controller 派发 Recorder；
    - 其完成声明在 Recorder/verify 后仍不等于语义审计或用户验收；
    - 不得把 Recorder 当成业务审计；
    - 不得自行修改其他角色资产。

    8. roles/artifact-recorder.md

    定义一个 Luna 类、机械执行、最小写权限角色。输入必须包括：

    - 已冻结 source file；
    - 独立 expected byte length 和 SHA-256；
    - repo root；
    - artifact root；
    - strict descriptor；
    - 当前 verify 结果；
    - 唯一 target path。

    只允许调用 inspect、record、verify。禁止打开或修改业务文件，禁止总结、修正、补全、换行规范化、自动脱敏，禁止 Git 写操作。敏感命中时只报告类别和位置，不回显值。写入成功后返回 payload path、manifest path、hash、length 和 verify 结果，不返回 payload 正文。

    明确该命令结果不再递归记录；manifest 是 Recorder 的终端持久化输出。

    9. tools/artifact_recorder.py

    只用标准库，提供：

    - inspect --source ...
    - record --repo-root ... --artifact-root ... --descriptor ...
    - verify --repo-root ... --artifact-root ... --task-id ... --chapter meta|chNN

    设计要求：

    - 所有成功输出为稳定 JSON，不打印 payload。
    - 错误输出只含错误码、类别、字段或安全位置，不回显敏感值或整段正文。
    - descriptor 是严格 JSON；拒绝缺字段、未知字段、错误类型和非法枚举。
    - source 必须为显式普通文件，不允许 symlink；严格 UTF-8 decode，但写入使用原始 bytes。
    - expected_byte_length 与 expected_sha256 必须在写入前匹配。
    - task_id、artifact_key 使用受限 slug；sequence 1..999；attempt 1..99。
    - role/status 使用明确 allowlist 和按角色 status 映射。
    - target_path 必须由 sequence、role、attempt 和 extension 确定性计算，格式为：
      NNN-<role-with-hyphens>-attempt-NN.payload.<ext>
    - target 必须是 artifact root 下的单一 basename，不允许绝对路径、斜杠、.. 或编码绕过。
    - artifact root 只能是：
      docs/workflow-runs/<task_id>
      或 docs/chapters/chNN/workflow/<task_id>
    - 对 repo root 到 artifact root 的所有已存在组件执行 lstat；任一 symlink 或非目录都拒绝。payload、manifest、descriptor、source 也拒绝 symlink。
    - manifest 路径由 payload stem 唯一派生。
    - manifest 用 json.dumps(sort_keys=True, ensure_ascii=False, indent=2) 加一个 LF，保存为 .manifest.yaml；verify 重新序列化并要求 canonical bytes 完全一致。
    - manifest 至少记录 schema_version、task_id、chapter、sequence、artifact_key、role、attempt、status、target_path、media_type、byte_length、sha256、source_provenance、source_reference、identity_source、historical、recorded_by、revision_of。
    - manifest 不保存 source 的绝对本地路径。
    - role result 使用 identity_source=payload_header。工具从开头 fenced YAML 头提取 task_id、role、status 并与 descriptor 比较。bootstrap 的 source protocol 可以为 1；v2 后续必须为 2。
    - user gate、approved prompt、task package 和 controller state 使用 external_descriptor；其身份来自可信 descriptor、target 和 expected digest，不能声称正文自带身份。
    - sequence 必须从 1 连续追加；不允许 gap、重复或倒退。
    - 同一 artifact_key 的 correction 必须使用递增 attempt；attempt>1 必须 revision_of 指向同 key、前一 attempt 的 manifest；attempt=1 的 revision_of 必须为空。
    - 不覆盖任何已有 payload 或 manifest。成功后的相同 record 再次调用也必须报告 already_exists，不静默视为新写入。
    - 在 artifact root 同目录创建临时文件，写入、flush、fsync 后使用 no-overwrite 安装。普通异常清理本次临时文件和本次已安装的半对。
    - 若进程崩溃留下“payload 已完整安装、manifest 缺失”的 orphan，后续相同 descriptor 可在 bytes/hash/sequence 全部吻合且没有后续 sequence 时只补装 deterministic manifest；其他 orphan 一律 fail closed。
    - manifest 存在但 payload 缺失永远 fail closed。
    - verify 只读，扫描整个 root，拒绝未知文件、临时残留、孤儿、sequence gap、非 canonical manifest、重复 sequence、错误 pair、hash/length/identity/revision 不匹配和 symlink。
    - inspect 只返回 byte_length、sha256、utf8_valid 和敏感类别，不打印正文，不创建文件。
    - 敏感扫描至少覆盖：非占位的 Authorization credential、常见 API/token 前缀、敏感环境变量赋值、private-key PEM 内容、source 文件名为 .env。空值和明确占位符可允许。不要仅因文档出现“Authorization Header”或“.env”字样就误报。
    - unrelated private content 无法由正则完备判定，文档必须如实说明；source producer 和 Controller 仍负责范围控制。
    - exit code 至少区分 success、input/schema error、integrity/security error。
    - 不访问网络，不读取 descriptor/source/artifact root 之外的任务无关文件。

    10. tests/test_artifact_recorder.py

    用 unittest、tempfile 和 subprocess/mock；fixture 全部在 /tmp 的临时目录中。不得访问网络或真实凭据。至少覆盖：

    - ASCII、中文、emoji、LF、CRLF、无末尾换行逐字节保存；
    - length mismatch、hash mismatch、invalid UTF-8；
    - 每类敏感模式拒绝，错误中不回显测试值；
    - policy 文本中的 Authorization/.env 词语不误报；
    - meta root 和 chapter root 正常；
    - absolute target、..、slash、错误 task root、错误 chapter 拒绝；
    - source、root component、payload、manifest symlink 拒绝；
    - descriptor 缺字段、未知字段、错误类型、非法 provenance/attestation 拒绝；
    - target/role/status/task/header identity mismatch 拒绝；
    - sequence 首项非 1、gap、重复、倒退拒绝；
    - attempt/revision 正常链和错误链；
    - existing pair no-overwrite 且原 bytes 不变；
    - record 成功后 verify；
    - manifest canonical bytes 被改、payload 被改、manifest/payload 孤儿、未知文件和临时残留均使 verify 失败；
    - 同 descriptor 对合法 payload orphan 的恢复；
    - 模拟第二步安装失败时普通异常清理；
    - inspect 和 verify 前后目录状态不变；
    - CLI 成功输出不含 payload 正文。

    六、公共 API、依赖和注释边界

    - 不新增依赖，不修改 Cargo。
    - Python CLI 是本任务唯一新可执行公共接口。
    - 内部函数保持最小，不建立未来数据库、签名服务、云存储或插件抽象。
    - 注释解释 no-overwrite、symlink 防护、exact-byte、recovery、canonical manifest 和递归终止的原因与不变量；不要逐行翻译代码。
    - 文档不能把未运行测试写成通过，也不能把 v2 写成已最终验收。
    - 不声称 scanner 能识别所有秘密或无关私人内容。

    七、bootstrap 写入

    基础文件和测试完成、工具通过 py_compile/unittest 后，逐个使用 record 处理 source bundle 的 001 至 009 descriptor。不得手写 payload 或 manifest。

    每次 record 后都运行 verify；上一 sequence 未通过 verify 时不得继续下一项。最终 verify 必须得到 artifact_count=9、last_sequence=9。不得创建 010-implementation 或任何后置资产；implementation 原始结果只能在本角色返回后由新的 Recorder 角色保存。

    八、验证命令

    实际执行并记录退出码：

    1. python3 -m py_compile docs/prompts/workflow/v2/tools/artifact_recorder.py docs/prompts/workflow/v2/tests/test_artifact_recorder.py
    2. python3 -m unittest discover -s docs/prompts/workflow/v2/tests -p 'test_*.py' -v
    3. python3 docs/prompts/workflow/v2/tools/artifact_recorder.py --help
    4. 对九个 bootstrap descriptor 依序执行 record；descriptor 绝对路径由交接包提供
    5. python3 docs/prompts/workflow/v2/tools/artifact_recorder.py verify --repo-root /Users/huaodong/Documents/ChatGPT/agent-harness-book --artifact-root docs/workflow-runs/workflow-v2-artifact-recorder-001 --task-id workflow-v2-artifact-recorder-001 --chapter meta
    6. 对十四个 v1 文件执行显式 shasum -a 256，并与本 Prompt 的正确值逐项比较
    7. git diff --check
    8. git status --short

    不运行 Cargo、mdBook、真实网络、真实凭据、远程 CI 或部署检查。测试产生的 fixture 必须在 /tmp，由 unittest 临时目录清理。

    九、停止条件

    遇到任一情况立即停止，不扩大范围、不覆盖、不清理用户工作树：

    - approval_status 不是 approved，批准证据不属于本 task_id，或 Prompt 被改写；
    - HEAD、工程根目录或 v1 SHA-256 与基线不一致；
    - 任一目标文件已经存在；
    - bootstrap source bundle 不完整、不是 raw export、hash/length 不独立、含 symlink、身份不匹配或敏感命中；
    - 需要修改白名单外文件；
    - 需要新增依赖、联网或真实凭据；
    - 工具或测试无法可靠运行；
    - 实际变化混入用户既有内容；
    - 发现无法安全区分的 staged 内容；
    - 需要 Git 写操作或远程操作。

    停止时保留现场，报告已经创建的精确文件和失败证据。不得 reset、revert、删除用户文件或静默修改 Prompt。

    十、严格禁止

    - git add、git commit、push、tag、branch switch、remote 修改；
    - git add . 或 git add -A；
    - 修改 v1；
    - 触碰 ch3、tutorial、book、Cargo、examples、crates、CI；
    - 预写 implementation、audit、summary、commit、commit-audit 或 acceptance 结果；
    - 把复制粘贴后的文本冒充 original bytes；
    - 自动脱敏或修改 payload；
    - 把 Recorder/verify 当作语义审计；
    - 声称未运行验证通过。

    十一、完成报告

    返回 protocol_version 1 的 implementation 结构化结果，随后严格提供：

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

    还必须单列：

    - bootstrap source provenance 与九个 manifest 路径；
    - v1 前后 SHA-256 是否逐项一致；
    - actual files 与白名单比较；
    - 网络和凭据使用情况；
    - 没有 Git 写操作；
    - ch3 仍冻结；
    - 当前结果只是待独立 post-implementation-audit 的 implementation 声明。

requires_user_confirmation: true
requested_confirmation: "请审阅计划和完整编码 Prompt；只有明确回复‘批准实施’后，Controller 才能准备原始 bootstrap source bundle 并派发唯一 coder。"
```

完整编码 Prompt如下；它与上面 `implementation_prompt.full_text` 相同，是本轮待批准的冻结实施文本。

---

# workflow-v2-artifact-recorder-001 实施 Prompt

你是本任务唯一的 Luna 类 implementation coder。你只能执行下面已经获得本轮用户明确“批准实施”的 Prompt；未收到 `approval_status: approved`、批准原文、完整 bootstrap source bundle 和本 Prompt 原文时必须停止。

## 一、目标与已确认业务逻辑

在保留 workflow/v1 完全不变的前提下，新建 breaking workflow/v2。v2 增加一个最小写权限的 Artifact Recorder 角色和一个仅依赖 Python 标准库的确定性工具，用于保存冻结的角色完整输出、自然语言正文、人工门禁原文、任务包、获批 Prompt、Controller 状态、implementation/audit/summary/commit/commit-audit 结果及配对 manifest。

Recorder 只能机械保存已冻结的 UTF-8 bytes 和元数据，不能总结、修正、补全、规范化或自动脱敏，不能修改业务文件。v2 不增加 Artifact Audit；Controller 必须复用同一工具的 `verify` 子命令做 path、length、SHA-256、pair、sequence、attempt/revision 和 identity 的确定性验证，不判断业务内容。

章节任务根目录固定为：

```text
docs/chapters/chXX/workflow/<task_id>/
```

非章节/meta 工作流任务根目录固定为：

```text
docs/workflow-runs/<task_id>/
```

本任务属于 meta，`task_id` 为 `workflow-v2-artifact-recorder-001`。

raw payload 独立保存，不添加 Markdown 包装、不做换行转换、不添加末尾换行。metadata 保存为独立 `.manifest.yaml`。manifest 使用 Python 标准库生成的 canonical JSON 文本；JSON 是 YAML 1.2 的有效子集，因此无需引入 YAML 依赖。纠错必须使用新 sequence 和递增 attempt/revision，禁止覆盖旧资产。

敏感信息、路径或校验不匹配必须 fail closed 并阻止状态推进。bootstrap coder 只能保存真实可取得的前置资产并标记 `recorded_by=bootstrap-coder`，不能伪装成历史 Recorder，也不能预写 implementation、audit、summary、commit 或 commit-audit 结果。v2 建立后，后置结果由 Artifact Recorder 角色保存。

Recorder 自己的终端结果是 payload/manifest 对和不含 payload 正文的 JSON 命令结果，不再递归记录 Recorder 输出。这是明确的递归终止边界。

## 二、事实基线

工程根目录：

```text
/Users/huaodong/Documents/ChatGPT/agent-harness-book
```

计划阶段确认：

- HEAD：`355a7d0d240e68f9e7ffc336e42913b609f5ac5e`
- branch：`main`
- `main` 相对 `origin/main` ahead 4
- `previous_commit: null`
- 工作树已脏，包含大量用户修改和未跟踪资产
- Python：`/opt/homebrew/bin/python3`，Python 3.14.6
- Cargo 可用但本任务不需要运行
- v1 当前未跟踪，属于用户既有资产

v1 正确 SHA-256：

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

实施开始时必须重新计算全部十四个值；任一不一致立即停止，不得更新基线。

## 三、实施前检查和 bootstrap 门禁

先执行并记录：

```bash
pwd
git status --short --branch
git rev-parse HEAD
python3 --version
```

再显式计算上述十四个 v1 文件的 SHA-256。确认所有允许新增文件尚不存在。

Controller/操作者必须随实施交接提供一个位于 `/tmp` 下的 bootstrap source bundle，包含九个独立 UTF-8 source 文件及九个严格 JSON descriptor：

1. `task_analysis / passed / payload_header`
2. `controller_state / awaiting_requirement_confirmation / external_descriptor`
3. `user_gate_requirement / confirmed / external_descriptor`
4. `planning / passed / payload_header`
5. `controller_state / awaiting_implementation_approval / external_descriptor`
6. `user_gate_implementation / approved / external_descriptor`
7. `approved_prompt / approved / external_descriptor`
8. `task_package / approved / external_descriptor`
9. `controller_state / implementation / external_descriptor`

descriptor 必须带独立 expected length/hash。过去原始输出才可标 `historical=true`；实施交接时生成的 task package/snapshot 必须为 false。全部 bootstrap manifest 标记 `recorded_by=bootstrap-coder`。

自动模式只接受 `platform_raw_export` 或 `bootstrap_handoff`。手动模式只接受 `manual_raw_export`，并必须声明平台 raw/download export 方法和“未使用复制粘贴、导出后未修改”的 attestation。`manual_copy`、`clipboard_copy`、`reconstructed_from_chat` 一律拒绝。

bundle 不满足条件时，在仓库写入前停止。

## 四、允许范围

只允许新增计划 YAML 中列出的：

- 一个 v2 decision；
- `docs/prompts/workflow/v2/` 的完整协议、八个角色、模板、工具和测试；
- 九个 bootstrap payload/manifest 对。

不得修改任何既有文件。特别禁止修改 v1、ch3、tutorial、book、Cargo、crates、examples、CI 和 `docs/report.md`。

用 `apply_patch` 创建决策、Prompt、模板、工具和测试。bootstrap payload/manifest 只能由新 Recorder 工具从 source bundle 写入。

## 五、实现要求

### v2 决策与协议

决策必须说明 v2 是 breaking 版本、与 v1 并存，并完整记录：

- Recorder 最小写权限；
- raw export 边界；
- 先 record/verify 后推进状态；
- no-overwrite 和 attempt/revision；
- bootstrap provenance；
- 敏感信息 fail closed；
- Controller 只做确定性验证；
- Recorder 不递归记录自身；
- 两文件采用可恢复顺序而非虚构全局原子事务；
- 无签名、无外部可信时间、无完备 secret detection。

README、Controller、manual handoff、任务包、结果模板和七个既有角色必须保留 v1 的权限与五个人工门禁，不得弱化。v2 每个角色完整响应都必须从 fenced YAML 结构头开始，并在后面保留自然语言。每次输出和用户门禁都先保存、verify，再推进。

Artifact Recorder 角色只能调用 `inspect`、`record`、`verify`，只写当前 task root；不得碰业务文件、Git 状态或 payload 内容。

### 确定性工具

`artifact_recorder.py` 仅用标准库，提供：

```text
inspect --source ...
record --repo-root ... --artifact-root ... --descriptor ...
verify --repo-root ... --artifact-root ... --task-id ... --chapter meta|chNN
```

核心契约：

- exact bytes：严格 UTF-8 检查，但保存原 bytes；
- strict descriptor，拒绝未知字段；
- expected hash/length 在写入前匹配；
- 合法 task/chapter root；
- 所有路径组件和文件拒绝 symlink；
- target 由 sequence/role/attempt/extension 唯一推导；
- manifest 是 canonical JSON-as-YAML；
- manifest 不泄露 source 绝对路径；
- role output 的 header 身份与 descriptor 匹配；
- gate/prompt/state 使用 external descriptor；
- sequence 从 1 连续追加；
- correction 使用递增 attempt 和 `revision_of`；
- 绝不覆盖；
- 同目录临时文件、flush/fsync、no-overwrite 安装、普通失败清理；
- 只允许在精确匹配时恢复 payload-only orphan；
- verify 只读并检查整个 root；
- inspect/错误输出不打印 payload 或敏感值；
- 敏感扫描识别非占位凭据、token/key assignment、private-key 内容和 `.env` source，避免仅因政策文字误报；
- 不联网，不读任务外文件。

### 测试

使用 `unittest`、`tempfile`、`subprocess/mock`，fixture 位于 `/tmp`。覆盖：

- Unicode、LF/CRLF、无末尾换行；
- hash/length、UTF-8；
- 敏感拒绝与不回显；
- 合法/非法 root、target、traversal、symlink；
- descriptor schema/provenance/attestation；
- task/role/status/header identity；
- sequence、attempt、revision；
- no-overwrite；
- canonical manifest、pair、tamper、orphan、unknown temp；
- orphan recovery 和安装失败清理；
- inspect/verify 只读；
- CLI 不输出正文。

## 六、bootstrap

工具测试通过后，按 001 至 009 依次调用 `record`。每次后运行 `verify`。上一项未验证不得继续。最终必须是：

```text
artifact_count = 9
last_sequence = 9
```

不得创建第 010 个 implementation 资产；implementation 原始结果只能在本角色返回后由新 Recorder 保存。

## 七、验证

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

另需显式复核十四个 v1 SHA-256。

不运行 Cargo、mdBook、网络、真实凭据、远程 CI 或部署。

## 八、停止与禁止

任何基线、source、hash、路径、身份、敏感信息、测试或白名单不满足即停止。不得扩大范围、覆盖文件、自动脱敏、补造历史、弱化测试或清理用户工作树。

禁止全部 Git 写操作、远程操作、v1 修改、ch3 修改和后置结果预写。

## 九、完成报告

输出 implementation 结构化结果和仓库规定的十一项报告，并单列：

- 九个 bootstrap provenance 和 manifest；
- v1 前后 SHA-256；
- actual files 与白名单；
- network/credential 使用；
- Git 写操作为零；
- ch3 继续冻结；
- 当前仅是待独立审计的 implementation 声明。

本计划阶段只进行了只读核对和 SHA-256 采集，没有修改/创建文件，也没有运行实现测试。
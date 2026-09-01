《从 0 搭建 AI Agent》技术研究报告

研究基线：2026 年 8 月 4 日

本文优先采用论文、官方文档、官方源码仓库和一线工程团队文章。需要特别说明：“loop engineering”是 2026 年才迅速流行的行业术语，尚未形成稳定的学术定义；相关学术基础实际上来自 ReAct、Reflexion、Self-Refine、测试反馈循环和控制论式 Agent 研究。


---

绪论：Agent 技术焦点变迁

0.1 总体判断

“Prompt engineering → Context engineering → Harness engineering → Loop engineering”不应被写成四代技术互相替代，而应写成Agent 工程关注点逐层外移：

1. Prompt engineering：优化一次模型调用。


2. Context engineering：优化模型在当前步骤能看到什么。


3. Harness engineering：优化模型如何获得状态、工具、权限、反馈和验证。


4. Loop engineering：优化整个系统如何反复行动、纠错、收敛和停止。



因此，新的阶段并未消灭旧阶段，而是把旧阶段包进更大的系统边界。

0.2 建议采用的时间划分

阶段	主要活跃期	核心问题	标志性突破	代表项目或实践

Prompt engineering	2020—2024 为主流焦点，持续至今	怎样用输入文本激发模型能力	Few-shot、CoT、Self-Consistency、结构化提示	GPT-3 prompting、CoT、DSPy、PromptSource
Context engineering	2023—2025 快速形成，持续至今	有限窗口内应该放什么、删什么、何时检索	RAG、长上下文、摘要压缩、分层记忆、上下文隔离	LangChain/LangGraph、MemGPT、LlamaIndex
Harness engineering	2024 年萌芽，2025—2026 成为明确工程范式	怎样把模型包进一个可靠运行时	Tool registry、状态管理、沙箱、审批、会话、验证、追踪	Codex、Claude Code、Pi、OpenHands
Loop engineering	2026 年形成行业术语，学术根源始于 2022	怎样设计能够持续行动、反馈、纠错并稳定停止的闭环	ReAct、Reflexion、Self-Refine、测试驱动循环、多代理审查	Codex/Claude Code 自动循环、Symphony、各类 reviewer loop


GPT-3 的 few-shot prompting 将“通过输入而非微调改变行为”推向主流；CoT 和 Self-Consistency 随后扩展了单次推理能力。RAG、MemGPT 和 “Lost in the Middle” 等工作则证明，扩大上下文并不等于正确利用上下文，由此推动了上下文选择、压缩和记忆管理。

2025 年后，“context engineering”开始被明确描述为对有限上下文资源的系统化管理；2026 年，“harness engineering”进一步把关注点扩展到工具、运行环境、验证、权限、状态和可观测性。OpenAI 的实践和同期论文都指出，Agent 成功与否往往取决于整个“模型—harness—环境”系统，而不只是模型本身。

0.3 技术进化树

Foundation Model
│
├── Prompt Engineering
│   ├── 指令与角色
│   ├── Few-shot
│   ├── Chain-of-Thought
│   └── Structured Output
│
├── Context Engineering
│   ├── 当前对话选择
│   ├── RAG / 搜索
│   ├── 摘要与压缩
│   ├── 长短期记忆
│   └── 上下文隔离
│
├── Harness Engineering
│   ├── Model Adapter
│   ├── Tool Registry / Executor
│   ├── Session / State
│   ├── Sandbox / Permission
│   ├── Validation / Guardrail
│   └── Trace / Observability
│
└── Loop Engineering
    ├── Plan → Act → Observe
    ├── Verify → Diagnose → Revise
    ├── Progress / Convergence Detection
    ├── Reviewer / Sub-agent
    ├── Failure Recovery
    └── Stop / Escalate / Human Approval

横向贯穿四层的能力是：评测、安全、可观测性、成本控制和人类监督。

0.4 重要论文和文章

资源	作者与年份	价值

Language Models are Few-Shot Learners	Brown 等，2020	Prompt 范式起点之一。
Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks	Lewis 等，2020	外部知识进入上下文的基础。
Chain-of-Thought Prompting Elicits Reasoning in Large Language Models	Wei 等，2022	多步推理提示。
ReAct: Synergizing Reasoning and Acting in Language Models	Yao 等，2022/2023	推理与环境行动交替。
Lost in the Middle	Liu 等，2023	长上下文并不保证有效利用。
MemGPT	Packer 等，2023	分层、虚拟化上下文管理。
Reflexion	Shinn 等，2023	语言反馈与情节记忆。
Self-Refine	Madaan 等，2023	生成—反馈—改写循环。
Plan-and-Solve Prompting	Wang 等，2023	先规划、后执行。
Effective Context Engineering for AI Agents	Anthropic，2025	工业界上下文工程总结。
Harness Engineering: Leveraging Codex in an Agent-first World	Lopopolo，2026	Agent 友好仓库和验证环境。
Harness Engineering for Coding Agent Users	Böckeler 等，2026	面向使用者的 harness 心智模型。
Agentic Harness Engineering	Lin 等，2026	基于可观测证据自动演化 harness。
What Is Loop Engineering?	IBM，2026	当前行业定义。



---

第一部分：大模型与 Agent

第一章：大模型不是 Agent

1.1 核心概念与定义

从学术角度看，Agent 是一个能够：

接收环境状态或观察；

根据目标和内部状态选择行动；

通过行动影响环境；

根据反馈调整后续策略；


的闭环决策系统。因果 Agent 定义尤其强调：系统的策略会因为自己的行动能够影响环境而发生适应。

工业界通常采用更工程化的定义：Agent 是一个由模型驱动、可以自主规划、调用工具、观察结果、调整计划并持续执行至停止条件的系统。OpenAI Agents SDK、Anthropic Agent 文档和当前 Agent 调研均把工具、状态、循环、交接、guardrail 和 tracing 视为 Agent 的组成部分。

1.2 大模型与 Agent 的本质差异

维度	大模型	Agent

本质	条件概率生成器或推理模型	目标驱动的闭环运行系统
输入输出	输入 token，输出 token 或结构化请求	观察环境、采取行动、读取反馈
状态	主要依赖当前上下文	具有会话、任务、环境和持久状态
外部能力	本身不能真正操作文件、网络或系统	通过受控工具影响环境
错误处理	通常生成一次答案	可验证、重试、回滚、降级
权限	没有真正的操作系统权限边界	Harness 实施权限、审批和隔离
停止	生成结束即停止	由目标、预算、收敛性和策略决定


“Agent = Model + Harness”是一个有效的工程简称，但需要补充：harness 必须包含状态、环境接口和执行循环。否则，模型加几个工具声明仍只是一个能提出工具请求的模型调用程序。

1.3 为什么单靠模型无法完成以下任务

1. 安全地修改代码仓库
模型可以生成补丁，但不能自行保证路径安全、执行测试、处理冲突、回滚失败修改或证明测试确实通过。


2. 在崩溃后继续长任务
模型权重不会保存“任务执行到第几步”；需要 session、事件日志、checkpoint 和幂等工具调用。


3. 执行不可逆操作
发送邮件、部署生产环境、转账、删除数据都需要身份认证、最小权限和用户审批，不能依靠系统提示词约束。


4. 保证输出满足机器接口
模型可能输出无效 JSON、错误字段或语义冲突；需要 schema、约束生成和程序化验证。


5. 获得当前事实并给出可验证证据
模型参数中的知识可能过时；需要搜索、检索、来源记录、声明—证据映射和事实验证。



这些问题分别对应 harness 的工具层、状态层、安全层、验证层和上下文层。

1.4 最小系统边界

pub trait Model {
    async fn generate(&self, request: ModelRequest)
        -> Result<ModelResponse, ModelError>;
}

pub trait Environment {
    async fn execute(&self, action: ToolCall)
        -> Result<Observation, ToolError>;
}

pub trait Policy {
    fn authorize(&self, call: &ToolCall, state: &AgentState)
        -> Decision;
}

pub struct Agent<M, E, P> {
    model: M,
    environment: E,
    policy: P,
    state: AgentState,
}

真正的 Agent 行为不是 model.generate()，而是围绕这几个接口运行的有界循环。

1.5 教学建议

本章应详讲：

“预测下一个 token”与“闭环控制系统”的区别；

模型能力和系统能力不可混为一谈；

Agent 的自治是分级的，而非有或没有；

安全边界必须由模型外部实施。


可略讲传统强化学习数学推导，但应使用状态、行动、观察、策略和奖励这些概念建立统一语言。


---

第二章：Harness 设计哲学

2.1 Codex、Claude Code、Pi 对比

维度	OpenAI Codex	Claude Code	Pi

核心语言	Rust 为主的 CLI/runtime	产品内部实现不完全公开，SDK 提供 Python/TypeScript	TypeScript monorepo
架构倾向	强类型事件协议、沙箱、审批、工具循环	生命周期 hooks、权限模式、subagent、MCP、skills	极小核心、扩展优先、低意见化
状态管理	对话项、response ID、事件和 app-server 协议	sessions 可恢复/分叉，项目 memory 和配置	JSONL session、分支、压缩和自定义事件
工具机制	内置 shell、文件、MCP 等，统一工具事件	内置工具、MCP、hooks、subagent 工具限制	Tool registry 与 extensions
安全	OS 级沙箱与审批策略分离	默认只读，写入和高风险 Bash 请求许可，可结合 sandbox/hooks	扩展可获得宿主权限，安全更多由使用者配置
指令组织	系统提示、AGENTS.md 分层规则、skills	CLAUDE.md、skills、commands、memory、hooks	system prompt、prompt templates、skills、extensions
设计哲学	让运行时和仓库变得适合 Agent	通过完整生命周期扩展和权限控制工作流	核心保持小，复杂功能由用户组合


Codex CLI 是公开的本地 coding agent，当前仓库以 Rust 实现核心运行时；其安全设计将“技术上允许做什么”的 sandbox 与“何时允许升级权限”的 approval policy 分开。

Claude Code 把 hooks、subagents、MCP、permissions、sessions、skills 和 plugins 纳入同一 Agent 生命周期。官方安全说明显示其默认采用严格只读权限，对文件修改和可能改变系统状态的 Bash 操作请求批准。

Pi 则刻意保持最小核心，通过扩展、skills、prompt templates 和自定义 provider 构建不同工作流；它的 session 以 JSONL 保存模型变化、压缩、分支摘要等事件。这种设计非常适合作为教学参考，但任意扩展代码具有宿主权限，不能直接视为安全沙箱。

2.2 极简但完整的 Harness

建议全书最终形成以下八个核心模块：

1. ModelAdapter
2. ContextBuilder
3. ToolRegistry
4. ToolExecutor
5. AgentLoop / StopPolicy
6. SessionStore
7. Policy / Approval / Sandbox
8. Validator / TraceSink

推荐依赖方向：

User Goal
   ↓
ContextBuilder → ModelAdapter
                     ↓
                Tool Request
                     ↓
Policy → Approval → Sandbox → ToolExecutor
                                ↓
                           Observation
                                ↓
                  Validator → State Commit
                                ↓
                        Continue / Stop

2.3 多语言选型

语言	优势	劣势	推荐用途

Rust	内存安全、类型系统强、异步与 CLI 生态成熟、易表达状态机	编译和泛型学习曲线较高	主 harness、沙箱控制面、session、工具运行时
C++	系统集成和既有基础设施强、性能可控	内存安全、异步和依赖管理成本高	引擎集成、已有 C++ 平台、底层执行器
Python	AI 生态最完整、原型开发最快	类型和隔离边界较弱、部署一致性较差	参考实现、评测器、检索、实验
TypeScript	Web、Node、MCP 和 UI 集成高效	CPU 密集和严格系统隔离不是强项	前端、插件层、交互式控制台


全书主线建议采用 Rust，C++ 用于系统与引擎集成附录，Python 和 TypeScript 用来展示同一协议在不同生态中的实现。


---

第二部分：从零构建极简 Agent 框架

第三章：第一个模型调用程序

3.1 模型调用不是 Agent

单次 API 请求只完成：

request → model → response

Agent 则至少需要：

goal → model → action → environment
             ↑              ↓
          updated state ← observation

3.2 统一模型接口

不要直接把业务代码绑定到某个 /chat/completions 路径，应先定义规范化接口：

#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn capabilities(&self) -> ModelCapabilities;

    async fn complete(
        &self,
        request: UnifiedRequest,
    ) -> Result<UnifiedResponse, ProviderError>;

    async fn stream(
        &self,
        request: UnifiedRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ModelEvent, ProviderError>> + Send>>,
              ProviderError>;
}

ModelCapabilities 至少记录：

pub struct ModelCapabilities {
    pub tools: bool,
    pub parallel_tools: bool,
    pub structured_output: bool,
    pub reasoning: bool,
    pub images: bool,
    pub streaming: bool,
    pub max_context_tokens: Option<u32>,
}

不要假设所谓“OpenAI-compatible API”在工具调用、流事件、usage、错误格式和 structured output 上完全兼容。

3.3 多语言调用模式

Rust：reqwest

let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;

let response = client
    .post(format!("{base_url}/responses"))
    .bearer_auth(api_key)
    .json(&request)
    .send()
    .await?
    .error_for_status()?;

reqwest::Client 应长期复用以利用连接池；流式响应可通过字节流逐块解析 SSE 或 JSON event。

C++：cpp-httplib

httplib::SSLClient client(host);
client.set_read_timeout(60);

httplib::Headers headers{
    {"Authorization", "Bearer " + api_key},
    {"Content-Type", "application/json"}
};

auto res = client.Post(path, headers, body, "application/json");
if (!res || res->status < 200 || res->status >= 300) {
    throw ProviderError(...);
}

cpp-httplib 是轻量单头文件 HTTP/1.1 库，适合教学，但生产环境还需补充异步、连接池、HTTP/2 和更完整 TLS 策略。

Python：httpx

async with httpx.AsyncClient(
    base_url=base_url,
    headers={"Authorization": f"Bearer {api_key}"},
    timeout=60,
) as client:
    async with client.stream("POST", "/responses", json=payload) as resp:
        resp.raise_for_status()
        async for line in resp.aiter_lines():
            handle_event(line)

高频调用中应复用 AsyncClient，而不是为每次请求新建连接池。

TypeScript：fetch

const response = await fetch(`${baseUrl}/responses`, {
  method: "POST",
  headers: {
    authorization: `Bearer ${apiKey}`,
    "content-type": "application/json",
  },
  body: JSON.stringify(payload),
  signal: AbortSignal.timeout(60_000),
});

if (!response.ok || !response.body) throw new Error("provider error");

for await (const chunk of response.body) {
  parser.feed(chunk);
}

Fetch 的 Response.body 是 ReadableStream，支持按块消费和背压处理。

3.4 API Key 与认证适配

pub enum AuthStrategy {
    BearerEnv { variable: String },
    HeaderEnv { header: String, variable: String },
    QueryEnv { parameter: String, variable: String },
    OAuthToken { audience: String },
    SignedRequest,
}

原则：

密钥只由 adapter 或 secret store 读取；

不进入 prompt、trace 或 observation；

日志中只记录 key ID，不记录 key；

支持 provider-specific headers；

支持密钥轮换和过期。


3.5 教学建议

详讲 HTTP、SSE、错误处理、超时、取消、流事件规范化和 provider capability；略讲特定厂商的所有字段，避免书籍迅速过时。


---

第四章：工具调用

4.1 工具模型

每个工具不应只有名字和函数，还应包含：

pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub risk: RiskClass,
    pub timeout: Duration,
    pub idempotency: Idempotency,
    pub required_scopes: Vec<Scope>,
}

执行流程：

模型提出 ToolCall
      ↓
JSON Schema 校验
      ↓
风险与权限判断
      ↓
必要时用户批准
      ↓
Sandbox 执行
      ↓
限制时间与输出大小
      ↓
脱敏、结构化 Observation
      ↓
写入 Trace / Session

4.2 常用工具分类

类型	风险点	最低要求

文件读取	敏感信息、路径逃逸	workspace root、大小限制、符号链接检查
文件修改	覆盖、竞争、部分写入	临时文件、原子替换、diff、备份
Bash	任意代码执行	无 shell 拼接、sandbox、timeout、资源限制
Git	工作区污染、推送外部	分支/worktree、远程操作审批
网络搜索	prompt injection、不可信内容	域名政策、来源标记、内容隔离
HTTP/API	凭证泄露、外部副作用	scoped token、allowlist、审批
数据库	越权、破坏性语句	只读默认、事务、查询预算


4.3 Rust 工具实现

let mut command = tokio::process::Command::new(program);
command
    .args(args)
    .current_dir(workspace)
    .env_clear()
    .envs(safe_env)
    .kill_on_drop(true);

let output = tokio::time::timeout(limit, command.output()).await??;

这里使用 Command::new(program).args(args)，而不是拼接字符串交给 /bin/sh -c。但必须强调：进程 API 不是安全沙箱。

4.4 Python 工具实现

proc = await asyncio.create_subprocess_exec(
    program,
    *args,
    cwd=workspace,
    env=safe_env,
    stdout=asyncio.subprocess.PIPE,
    stderr=asyncio.subprocess.PIPE,
)

stdout, stderr = await asyncio.wait_for(
    proc.communicate(),
    timeout=timeout_seconds,
)

避免 shell=True，并对参数、cwd、环境变量和输出大小分别限制。

4.5 Bash 沙箱方案

推荐从外到内组合：

Rootless container / microVM
  └── User namespace
       └── Read-only base filesystem
            └── Writable workspace mount
                 └── Network disabled by default
                      └── seccomp
                           └── AppArmor / SELinux
                                └── cgroup CPU / RAM / PID limits

Docker 默认 seccomp profile 会阻止一组高风险系统调用；rootless mode 和 user namespace 能进一步降低容器逃逸后的宿主权限。仅靠 Docker 并不自动安全，仍需非 root、网络策略、挂载限制和强制访问控制。

4.6 教学建议

本章应把“工具调用协议”和“安全执行工具”分开教学。JSON tool calling 很容易，真正困难的是参数验证、TOCTOU、路径规范化、权限、超时、幂等和审计。


---

第五章：最小 Agent Loop

5.1 经典模式

ReAct 把推理与行动交替组织，使 Agent 能通过外部环境取得新信息；Plan-and-Solve 先把任务拆成子任务再执行；Reflexion 将失败反馈写入情节记忆；Self-Refine 采用生成、反馈和改写循环。

5.2 推荐的最小循环

loop {
    budget.ensure_available()?;

    let context = context_builder.build(&state)?;
    let response = model.generate(context).await?;

    match response {
        ModelResponse::Final(candidate) => {
            let result = verifier.verify(&candidate, &state).await?;

            if result.accepted {
                return Ok(candidate);
            }

            state.push_verification_failure(result);

            if stop_policy.should_stop(&state) {
                return Err(AgentError::Unresolved);
            }
        }

        ModelResponse::ToolCalls(calls) => {
            for call in calls {
                let observation = tool_pipeline.execute(call, &state).await?;
                state.commit_observation(observation)?;
            }
        }
    }
}

5.3 两轮以上验证模板

不建议要求模型输出隐藏“思维链”。可要求它产生可审计的工作记录：

目标：
{goal}

当前状态与已验证事实：
{state}

未完成验收条件：
{criteria}

请执行以下协议：

1. PLAN
   给出简短、可检查的下一步计划，不超过 5 项。

2. ACT
   选择一个工具行动，或提交候选结果。

3. CHECK
   使用测试、检索结果、编译器或其他外部证据检查行动结果。
   不得仅以“我认为正确”作为证据。

4. DIAGNOSE
   若失败，列出：
   - 失败的验收条件
   - 直接证据
   - 下一轮必须改变的策略

5. REVISE
   根据诊断修改结果。

6. RECHECK
   重新运行验证。
   只有全部关键条件通过时才能输出 FINAL。

第二轮输入不应简单写“再想一遍”，而应包含上一轮的：

失败测试；

observation；

verifier report；

未满足条件；

禁止重复的无效行动。


研究表明，没有外部反馈时，模型的自我纠错并不总是可靠，甚至可能把正确答案改错。因此本书应把编译器、测试、检索、规则检查器和独立 reviewer 作为主要反馈源。

5.4 Trace 结构

pub struct TraceEvent {
    run_id: RunId,
    step_id: StepId,
    parent_step_id: Option<StepId>,
    phase: Phase,
    timestamp: DateTime<Utc>,
    input_digest: String,
    action: Option<ActionRecord>,
    observation: Option<ObservationRecord>,
    verification: Option<VerificationRecord>,
    state_delta: Option<StateDelta>,
    latency_ms: u64,
    token_usage: TokenUsage,
    stop_reason: Option<StopReason>,
}

应记录：

计划摘要；

模型和配置；

工具名、参数摘要及风险级别；

observation 的内容摘要和原始 artifact 引用；

验证结果；

状态变更；

成本、延迟和停止原因。


不应默认记录密钥、完整敏感文件或模型的隐藏思维链。


---

第六章：上下文工程管理

6.1 三类基本技术

滑动窗口

保留最近若干轮对话，简单且行为可预测，但容易遗失早期约束。

摘要压缩

把旧事件转换成结构化摘要：

目标
已完成工作
关键决策
未解决问题
不可违反的约束
证据与 artifact 引用

摘要必须保留来源 ID，防止“压缩后无法追溯”。

检索

根据当前任务选择历史事件、代码、文档和长期记忆。RAG 解决知识进入模型的问题，但检索质量、切片、排序和证据一致性仍由 harness 负责。

6.2 与 FlashAttention、MQA、GQA 的关系

FlashAttention 是 IO-aware 的精确 attention 实现，减少高带宽内存访问；MQA 让多个 query head 共享 key/value；GQA 则在多头注意力和 MQA 之间折中。这些技术降低长上下文计算或 KV cache 成本，但不决定“哪些信息值得进入上下文”。

可以这样区分：

模型/推理层：
“最多能装多少 token，处理这些 token 有多贵？”

Harness/context 层：
“这次究竟应该装哪些 token？”

“Lost in the Middle”表明，即使信息位于窗口中，模型对不同位置的信息利用能力也可能显著不同，因此简单扩大窗口不能替代上下文排序和压缩。

6.3 Rust 上下文管理

pub struct ContextManager {
    system: ImmutablePrompt,
    recent: VecDeque<Message>,
    summaries: Vec<SummaryBlock>,
    retrieved: Vec<EvidenceBlock>,
    token_budget: usize,
}

impl ContextManager {
    pub fn build(&mut self, request: &TaskRequest) -> Context {
        self.select_relevant_evidence(request);
        self.compress_until_within_budget();
        self.render_with_priority()
    }
}

建议优先级：

1. 安全政策与系统约束；


2. 当前目标和验收条件；


3. 最近 observation；


4. 与当前步骤直接相关的证据；


5. 项目长期规则；


6. 历史摘要；


7. 非关键对话。



6.4 Prompt injection 防御

把外部网页、文件、邮件明确标为“不可信数据”；

指令与数据采用不同结构字段；

不把 API 凭证放进模型上下文；

工具权限由外部 policy engine 判断；

对来自外部内容的新指令实施 taint 标记；

关键副作用要求审批；

对可疑内容限制工具集；

记录内容来源和传播路径。


OWASP 明确建议采用最小权限、应用控制凭证和高风险操作的人类批准；不能把系统提示词视为严格安全控制。

6.5 教学建议

详讲 token budget、优先级、压缩损失、来源追溯和 prompt injection；FlashAttention 等只需解释其与上下文成本的关系，不必在本书实现底层 CUDA kernel。


---

第七章：Sessions 管理

7.1 推荐模型：事件日志加快照

不要只保存一个不断覆盖的 JSON 状态，建议采用：

Append-only Event Log
          ↓
Periodic Snapshot
          ↓
Artifact Store

pub struct Session {
    id: SessionId,
    version: u64,
    status: SessionStatus,
    head_event: EventId,
    latest_snapshot: Option<SnapshotId>,
}

pub struct SessionEvent {
    session_id: SessionId,
    sequence: u64,
    event_type: EventType,
    payload: serde_json::Value,
    causation_id: Option<EventId>,
    idempotency_key: Option<String>,
    checksum: String,
}

7.2 崩溃恢复流程

1. 读取最近 snapshot
2. 验证 checksum 和 schema version
3. 重放 snapshot 之后的事件
4. 查找 started 但未 completed 的工具调用
5. 根据工具幂等性：
   - 安全重试
   - 查询外部状态
   - 回滚
   - 请求人工处理
6. 重新获得 session lease
7. 从安全检查点继续

7.3 并发会话

桌面单机版本可采用 SQLite WAL、每 session 乐观版本号和进程内锁；分布式部署则需要 PostgreSQL、租约、幂等键和消息队列。Pi 的 JSONL session 是适合教学的简单参考；LangGraph 的 durable execution 则展示了 checkpoint、恢复和 human-in-the-loop 在复杂工作流中的组合。

7.4 教学建议

本章应设置一次真实故障实验：

1. Agent 修改文件；


2. 工具执行到一半时强制终止进程；


3. 重启；


4. 从事件日志识别未完成操作；


5. 验证不会重复产生副作用。



这比只展示序列化代码更能体现 session 的价值。


---

第三部分：让 Harness 可信

第八章：Harness Engineering

8.1 工程加固层次

输入校验
  ↓
上下文构造
  ↓
约束生成
  ↓
工具参数验证
  ↓
安全执行
  ↓
结果验证
  ↓
状态原子提交
  ↓
循环与预算控制

8.2 关键措施

问题	工程措施

非法输入	长度、编码、schema、文件类型、路径校验
无效 JSON	constrained decoding、JSON Schema、重试修复
Provider 故障	指数退避、jitter、circuit breaker、fallback
重复副作用	idempotency key、外部状态查询
无限循环	步数、成本、时间、重复状态检测
Observation 爆炸	输出大小限制、摘要、artifact 外置
状态不一致	事件事务、版本号、原子提交
模型漂移	固定评测集、provider/model 版本记录
不可解释失败	trace、失败归因、replay package


Guardrails AI 提供 validators 和结构化输出控制；Outlines 等项目则把输出约束推进到生成过程。即便使用约束生成，业务语义仍必须由程序验证。

8.3 重试分类

enum RetryClass {
    Never,              // 权限拒绝、非法请求
    SameRequest,        // 短暂网络故障
    RepairRequest,      // JSON 格式错误
    Replan,             // 工具或策略失败
    ProviderFallback,   // 服务不可用
    HumanEscalation,    // 不可逆或不确定操作
}

不能对所有错误统一“再试三次”。重复一个错误计划通常只会浪费 token。


---

第九章：Loop Engineering

9.1 定义

截至 2026 年，loop engineering 通常被定义为：设计能够让 Agent 反复行动、观察、判断和迭代，直至完成用户目标的工作流，而不是由人类为每一步手写 prompt。

本书应给出更严格的定义：

> Loop engineering 是对 Agent 的状态转移、反馈信号、验证器、预算、故障恢复、收敛条件和停止策略进行系统设计的工程实践。



9.2 代表性循环比较

方法	反馈来源	记忆	主要风险

ReAct	环境 observation	当前轨迹	可能持续采取无效行动
Plan-and-Solve	初始计划和执行结果	当前上下文	错误计划可能贯穿全过程
Self-Refine	同一模型自评	当前轮次	自我评价偏差
Reflexion	环境奖励或语言反馈	情节记忆	错误反思可能被长期保留
CRITIC	外部工具反馈	当前轨迹	工具质量决定上限
测试驱动 coding loop	编译器和测试	repo/session	可能对测试过拟合
Reviewer loop	独立 Agent	审查记录	成本和意见冲突


AlphaCodium 等工作进一步表明，coding agent 的关键不是单个“神奇 prompt”，而是围绕问题理解、候选生成、测试和修复设计完整 flow。

9.3 收敛性检测

建议组合四类信号。

状态重复

signature =
hash(goal_status,
     failed_checks,
     last_tool,
     normalized_tool_args,
     artifact_digests)

最近窗口内 signature 重复超过阈值，说明循环可能没有产生新状态。

进展分数

progress =
  newly_passed_requirements
- newly_failed_requirements
- regression_penalty

连续若干轮 progress <= 0 时触发重规划或停止。

矛盾检测

维护结构化事实表：

claim_id | claim | source | confidence | valid_from | supersedes

同一事实出现互斥值时，不允许模型自行选择，必须重新检索或调用 verifier。

预算和时间

最大模型调用数；

最大工具调用数；

最大 wall-clock time；

最大 token 或费用；

单一失败类别最大重复次数。


9.4 异常降级

复杂模型不可用
  → 备用模型
  → 减少工具和上下文
  → 转为确定性程序
  → 输出部分成果和未完成项
  → 请求人类接管

降级不得悄悄改变任务语义。例如，不能把“部署生产环境”降级为“假设已经部署”。

9.5 确定性保障

LLM 本身通常不能提供传统程序意义上的绝对确定性。Harness 可以提升的是：

相同版本、配置和上下文的可重放性；

确定性的工具和验证器；

固定 schema；

固定依赖和环境；

状态变更的事务性；

验收标准的确定性；

不确定结果的拒绝或升级。


2026 年 harness 研究把每次修改、预测和后续验证结果组织成可证伪合同，并发现工具、中间件和长期记忆的结构改进可能比修改系统提示词更具迁移性。


---

第十章：输出验证

10.1 分层验证管线

Syntax
  ↓
Schema
  ↓
Semantic Rules
  ↓
Evidence / Factuality
  ↓
Execution / Tests
  ↓
Security / Policy
  ↓
Human Approval

10.2 事实性验证

建议流程：

1. 从输出提取可验证声明；


2. 对每条声明检索证据；


3. 保存来源和具体证据片段；


4. 判断 supported、contradicted 或 insufficient；


5. 只允许 supported 声明进入最终结果；


6. 对高风险领域安排人工审查。



模型 judge 可以辅助排序，但不应是唯一事实来源。

10.3 代码验证

层次	工具例子

格式与编译	rustfmt、cargo check、clang-format、编译器
静态质量	Clippy、clang-tidy、类型检查
测试	unit、integration、property、snapshot
安全扫描	Semgrep、CodeQL
依赖	cargo audit、供应链扫描
运行时	sanitizers、资源和超时测试
变更审查	diff policy、独立 reviewer


Semgrep 可在本地或 CI 中进行 SAST；CodeQL 将代码转换为可查询数据库，用于识别漏洞和错误，并能将结果输出为代码扫描告警。

10.4 Validation Report

{
  "status": "rejected",
  "checks": [
    {
      "id": "compile",
      "status": "passed",
      "evidence": "artifact://logs/cargo-check.txt"
    },
    {
      "id": "security",
      "status": "failed",
      "evidence": "artifact://reports/semgrep.sarif"
    }
  ],
  "unverified_claims": [],
  "regressions": ["test_auth_expiry"],
  "recommended_action": "revise"
}


---

第十一章：权限管理机制

11.1 权限模型

RBAC 只能回答“这个角色通常能做什么”，Agent 系统还需要结合：

用户；

当前任务；

工具；

资源；

参数；

风险；

时间；

环境；

数据敏感级别；


进行 ABAC 或 capability-based 判断。

pub struct Capability {
    subject: AgentId,
    tool: ToolName,
    resources: ResourcePattern,
    allowed_actions: Vec<Action>,
    expires_at: DateTime<Utc>,
    max_uses: Option<u32>,
}

11.2 风险级别

级别	示例	默认策略

R0	读取公开文档	自动
R1	读取项目文件、运行只读命令	受 workspace 限制自动
R2	修改文件、安装依赖	session 内批准或逐次批准
R3	网络写入、发送消息、push	明确批准
R4	生产部署、删除数据、财务操作	强认证、双重确认或禁止


Claude Code 当前采用只读默认和按操作升级许可，并允许 hooks 强制询问或拒绝；Codex 则将 OS sandbox 和审批策略组合使用。

11.3 Computer Use

Anthropic 的 Computer Use 安全设计会对截图或页面中的潜在 prompt injection 运行额外分类器，并在发现风险时引导模型请求确认。对浏览器 Agent，还应采用隔离 profile、临时凭证、受限域名和不可逆操作前确认。

关键原则：

> 模型可以提出行动，但不应自己决定自己是否拥有该行动的权限。




---

第十二章：可观测性

12.1 Trace 树

agent.run
├── context.build
│   ├── memory.retrieve
│   └── context.compress
├── model.generate
├── policy.evaluate
├── approval.wait
├── tool.execute
├── verifier.check
├── state.commit
└── agent.stop

OpenTelemetry 的 trace、span、metric 和 event 模型适合映射 Agent 执行过程；近期 GenAI semantic conventions 已覆盖模型、token usage、结束原因等字段。

12.2 核心指标

verified success rate；

task completion rate；

steps per successful run；

loop abort rate；

tool error rate；

approval request/deny rate；

retry 和 fallback 次数；

input/output/cached token；

cost per verified task；

time to first token；

总运行时间与 p95 latency；

context utilization；

crash recovery success；

regression rate。


12.3 用户应看到什么

推荐显示：

当前目标；

工作计划；

正在执行的工具；

修改 diff；

observation 摘要；

测试和验证结果；

权限申请；

当前成本和预算；

失败原因；

停止原因。


不应把“实时看到 Agent 的思考过程”理解成公开原始隐藏思维链。正确目标是：让用户看到可验证的决定、行动、证据和状态变化。

LangSmith 等平台采用 trace 记录模型调用、工具调用和各阶段决策，可作为产品界面的参考，但本书宜先用 OpenTelemetry 和自建事件模型实现最小版本。


---

第四部分：高级功能与总结

第十三章：多模型适配

13.1 统一核心加 Provider 扩展

不建议强行把所有模型压成最低公共接口。可以采用：

Unified Core
├── messages / input
├── tools
├── streaming events
├── usage
└── errors

Provider Extensions
├── reasoning controls
├── cache controls
├── computer use
├── provider routing
└── vendor-specific metadata

LiteLLM 通过 OpenAI 风格接口统一大量 provider，并提供重试、fallback、路由和成本记录；OpenRouter 重点提供跨模型和跨 provider 路由、fallback 与 BYOK。

13.2 路由策略

pub struct RoutePolicy {
    required_capabilities: ModelCapabilities,
    max_cost: Option<Money>,
    max_latency: Option<Duration>,
    data_region: Option<Region>,
    preferred_models: Vec<ModelId>,
    fallbacks: Vec<ModelId>,
}

路由依据应包括：

工具能力；

structured output；

上下文长度；

模态；

延迟；

价格；

数据合规；

历史任务成功率；

当前 provider 状态。



---

第十四章：Sub-agent 与多代理协作

14.1 经典模式

模式	适用场景

Manager–Worker	大任务拆分成相对独立子任务
Agent as Tool	主 Agent 调用专业 Agent
Handoff	当前 Agent 把控制权移交给另一 Agent
Reviewer	独立检查候选结果
Blackboard	多 Agent 共享结构化任务板
Map–Reduce	并行检索、分析、最后汇总
Debate	对高不确定问题提出不同方案


AutoGen 当前强调事件驱动、可组合的多代理应用；CrewAI 区分用于协作的 crews 和负责状态、控制流的 flows；OpenAI Agents SDK 则区分 agents-as-tools 与 handoff。

14.2 子代理协议

pub struct Delegation {
    task_id: TaskId,
    parent_run: RunId,
    objective: String,
    input_artifacts: Vec<ArtifactRef>,
    allowed_tools: Vec<ToolName>,
    token_budget: u64,
    deadline: DateTime<Utc>,
    expected_output_schema: Value,
    acceptance_criteria: Vec<Criterion>,
}

子代理默认不应继承主代理所有：

上下文；

凭证；

权限；

长期记忆；

工具；

成本预算。


Claude Code 的 subagent 设计同样强调独立 prompt、工具限制和上下文隔离。

14.3 多代理风险

token 和调用次数成倍增长；

多个 Agent 重复读取相同上下文；

结论冲突；

循环委派；

权限放大；

共享状态竞争；

reviewer 与 writer 使用同一偏差；

“大家都同意”不等于有证据。


因此，多代理应在单代理已经不能清晰解决问题时再引入。


---

第十五章：长期记忆、人机协同与插件市场

15.1 长期记忆

记忆系统应包含：

Write Policy
  什么值得记忆？

Manage Policy
  如何合并、过期、纠正和删除？

Read Policy
  何时检索，返回多少，怎样排序？

MemGPT 把上下文类比为分层内存；近期记忆调研则进一步把 Agent memory 拆成 write、manage、read 三个阶段。

每条记忆至少保存：

content
source
timestamp
confidence
scope
sensitivity
expiry
supersedes
embedding/model version

15.2 人机协同

推荐的控制点：

目标确认；

计划审阅；

高风险工具批准；

中间 artifact 编辑；

verifier 失败后的接管；

最终提交前 diff review；

生产操作双重确认。


15.3 插件市场

插件 manifest 至少描述：

name: repo-maintainer
version: 1.2.0
entrypoint: plugin.wasm
capabilities:
  - filesystem.read: workspace/**
  - filesystem.write: workspace/src/**
  - process.exec: [cargo, git]
network:
  allow: []
signature: ...

需要：

版本锁定；

签名和来源；

capability diff；

安装时权限提示；

sandbox；

kill switch；

撤销和审计；

供应链扫描；

插件数据隔离。


MCP 为 Agent 与外部工具、数据源之间提供了标准化连接方式，但远程 server 和外部内容同样可能带来工具投毒、prompt injection 和凭证风险。


---

最终框架对比

能力	本书极简 Harness	Codex	LangChain / LangGraph	AutoGPT Platform

主要目标	教学和可解释实现	生产级 coding agent	通用 Agent/工作流开发	高层可视化自动化
主语言	Rust/C++，辅 Python/TS	Rust 为主	Python/TypeScript	Python/平台化组件
Loop	自行实现、完全透明	内建 coding loop	graph/runtime 节点与边	block/workflow
Tool	强类型 registry	丰富内置工具和 MCP	大量 integrations	预制 blocks
State	event log + snapshot	session/protocol	checkpoint/durable execution	平台状态
Sandbox	教学中完整实现	OS sandbox + approval	由部署者集成	平台决定
Validation	一等模块	测试、review、环境证据	自定义节点/guardrail	workflow block
Observability	OpenTelemetry + trace event	内建日志和 UI	LangSmith 等	平台 UI
学习内部机制	最强	中等，可读部分源码	中等	较弱
开箱即用	低	高	中高	高


LangGraph 将自己定位为较低层的 Agent orchestration/runtime，支持 durable execution、人机协同和记忆；当前 AutoGPT 的重心已经转向可视化 Platform、blocks、triggers 和 marketplace，而经典 AutoGPT 部分处于实验或不再积极支持状态。


---

推荐的全书实现路线

建议每章在同一仓库增加一个可运行 milestone：

M0  model-call
M1  model-adapter
M2  tool-registry
M3  agent-loop
M4  context-manager
M5  session-store
M6  sandbox-and-policy
M7  validator
M8  loop-engineering
M9  observability
M10 subagents-and-plugins

最终目录：

agent-book/
├── crates/
│   ├── agent-core/
│   ├── model-adapters/
│   ├── tool-runtime/
│   ├── context-engine/
│   ├── session-store/
│   ├── policy-engine/
│   ├── validators/
│   └── observability/
├── examples/
│   ├── rust/
│   ├── cpp/
│   ├── python/
│   └── typescript/
├── book/
├── evals/
├── fixtures/
└── sandbox/

每个 milestone 必须包含：

可运行程序；

单元测试；

故障注入测试；

trace 示例；

安全说明；

练习题；

与前一版本的 diff；

对应 Python、TypeScript 或 C++ 参考实现。



---

技术未来展望

未来两到三年的研究重心很可能从“生成更像答案的文本”继续转向：

1. 可验证任务完成：不再只评估最终文本，而评估环境状态是否符合目标。


2. Harness 自动演化：根据失败轨迹修改工具、middleware、memory 和验证流程。


3. 模型与运行时协同训练：模型学习如何使用特定工具和 observation protocol。


4. Agent 操作系统化：统一身份、权限、状态、调度、记忆和 artifact。


5. 验证器成为核心资产：测试、仿真器、规则引擎和领域 oracle 比 prompt 更有长期价值。


6. 循环预算成为产品约束：成功率必须与 token、延迟、人工审批和风险共同优化。


7. 从 raw CoT 转向证据化执行：用行动、observation、验证结果和状态变化建立信任。


8. 多代理从“角色扮演”走向受控并发：以任务合同、隔离状态和结构化结果替代自由对话。



Codex 多 Agent、worktree、skills 和 automation 等最新实践，以及 Symphony 这样的 orchestration spec，已经表现出从单 Agent CLI 向并行监督和长时间运行系统演进的趋势。


---

写作与发布平台建议

推荐方案：GitHub + Docusaurus + Quarto

主站：Docusaurus

适合本书“交互式、多语言、逐章演进”的定位：

Markdown/MDX；

可嵌入 React 组件；

Rust/C++/Python/TypeScript 多语言 tabs；

可制作交互式 trace viewer；

可加入在线练习、架构图和版本切换。


简洁备选：mdBook

mdBook 与 Rust 技术栈高度一致，目录清晰、构建快，适合把 Git 仓库作为唯一事实源，并可通过 CI 自动构建部署。缺点是复杂交互通常需要自行开发 preprocessor 或前端扩展。

出版导出：Quarto

Quarto 可把多章内容输出为 HTML、PDF、Typst、Word 和 EPUB，并支持执行代码块、交叉引用和可复现计算，适合作为最终出版流水线。

建议的发布架构

GitHub repository
├── Docusaurus interactive website
├── mdBook lightweight mirror
├── Quarto PDF / EPUB release
├── GitHub Discussions
├── tagged source-code milestones
└── automated tests for every chapter

渠道建议：

GitHub 作为源码、issue、讨论和版本事实源；

GitHub Pages、Cloudflare Pages 或 Vercel 发布交互站；

Rust 中文社区、Rust Users Forum、Hacker News、Reddit 的 Rust/LocalLLaMA 社区发布技术版本；

知乎、掘金和微信公众号发布中文章节摘要；

完结后使用 Quarto/Typst 生成审稿 PDF 和 EPUB；

为每章附一个可下载 release tag，而不是只维护不断变化的 main。


最终平台选择建议：以 Docusaurus 作为读者界面，以 GitHub Markdown 作为内容事实源，以 Quarto 负责出版格式。
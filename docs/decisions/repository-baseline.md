# Repository Baseline Decision

Status: Accepted for P0
Scope: Repository initialization baseline

## Decision Status

这是 P0 初始化基线，用于冻结仓库治理、边界和后续演进规则。它不是永久不可修改的架构承诺。

后续重大变更必须通过新的 ADR 或明确的更新记录完成。不得静默修改已经冻结的关键决策。

当前仓库规范名称为 `agent-harness-book`。本名称是当前仓库的规范名称和候选公开仓库名；正式创建 GitHub 仓库前，仍可通过新的 ADR 修改名称。本基线不承诺任何 GitHub owner、URL、组织或 remote 地址。

## 1. 项目定位

本仓库承担两个目标：

1. 为《从 0 搭建 AI Agent》提供配套的、可运行、可测试代码；
2. 作为 Forge Studio 通用 Agent Harness 的实验性底层原型。

本仓库不是：

- Forge Studio 产品主仓库；
- Godot 修改版仓库；
- 完整生产级 Agent 平台；
- 多框架兼容层；
- 用于复制某一现有 Agent 产品内部实现的项目。

当前核心原则是：

```text
实现、测试、示例和书籍章节同步演进。
```

每个 milestone 应形成：

```text
设计
→ 实现
→ 自动化测试
→ 可运行示例
→ 书籍解释
→ 限制与安全边界
```

## 2. 仓库边界

本仓库负责：

- 通用 Agent 核心抽象；
- 模型 Provider 适配；
- Tool Runtime；
- Context Engine；
- Session 与恢复；
- Policy 与 Sandbox 边界；
- Validation 与 Evidence；
- Observability；
- 教学示例；
- 测试夹具；
- 书籍内容。

Forge Studio 仓库负责：

- Godot Bridge；
- Artifact Graph；
- SceneOperations；
- Godot 项目语义；
- 游戏资产与场景领域模型；
- Forge Studio 产品 UI；
- 产品级工作流和编排。

依赖方向冻结为：

```text
forge-studio may depend on agent-harness-book
agent-harness-book must not depend on forge-studio
```

第 16 章可以提供脱敏、裁剪或模拟的 Forge Studio 案例，但案例不能导致通用 crate 依赖 Forge Studio。Forge Studio 专属类型不得进入通用核心接口，除非后来证明其可以抽象为通用概念。

## 3. 仓库公开策略

- P0 开发初期保持私有；
- M0 验收完成后再判断是否公开；
- 本基线不创建 GitHub 仓库；
- 本基线不配置 remote；
- 本基线不修改仓库可见性。

公开仓库的最低门槛：

- M0 能编译；
- 默认测试通过；
- CI 配置有效；
- README 可以指导陌生开发者运行 M0；
- mdBook 可以构建；
- 许可证边界明确；
- 没有提交密钥；
- 默认测试不访问真实模型 API；
- 至少有一个稳定的 Mock HTTP 测试。

本基线不声称上述门槛已经满足。

## 4. 许可证策略

代码许可证冻结为：

```text
MIT OR Apache-2.0
```

适用范围：

- `crates/`；
- `examples/`；
- 测试代码；
- 脚本；
- 书籍中来自源码的代码片段。

书籍正文和原创插图冻结为：

```text
CC BY-NC-SA 4.0
```

适用范围：

- `book/` 中的原创正文；
- 原创图表；
- 原创练习；
- 原创解释性内容。

第三方材料继续遵循其原始许可证。后续必须清楚标记第三方引用。完整章节或大量外部贡献可能影响后续商业出版授权。

在贡献政策完善前，优先接受勘误、小范围改进和代码修复。P0-2 将创建标准许可证文件；本步骤不创建许可证正文。

## 5. Rust 与依赖基线

Rust 基线冻结为：

```text
Rust Edition: 2024
MSRV: 1.85
```

当前本地 Rust 版本为 1.93.0。本地工具链版本高于 MSRV，不代表代码可以忽略 MSRV。后续 CI 应至少使用 MSRV 执行 `cargo check`。提高 MSRV 必须显式记录；普通 milestone 不应无理由提高 MSRV。

依赖规则：

- 共享依赖统一进入根 `[workspace.dependencies]`；
- 禁止无理由重复声明不同版本；
- 默认禁止 Git dependency；
- 新依赖必须说明用途；
- 不为不确定的未来功能提前引入依赖；
- 不引入完整 Agent 框架作为核心实现；
- 优先使用小而清晰、可测试的依赖；
- 依赖升级必须保持默认测试通过。

## 6. CI 基线

普通 push 和 pull request 使用：

```text
Ubuntu fast gate
```

至少覆盖：

- 格式；
- 编译；
- 单元测试；
- Clippy；
- mdBook；
- Python lint；
- TypeScript type check。

`main`、版本标签或手动发布检查使用：

```text
Ubuntu
macOS
Windows
```

跨平台检查至少覆盖：

- `cargo check --workspace --all-targets`；
- `cargo test --workspace`。

不要求每次提交在三个系统运行所有文档工具。默认 CI 不得调用真实模型 API，也不得依赖真实 API Key。

跨平台故障测试不得只依赖 Unix `kill`，应优先使用可控 failpoint。GitHub Actions 只有在实际运行后才能声明通过。

## 7. API 稳定性策略

冻结：

```text
M0-M4 的公共 API 均视为实验性。
```

可以为获得正确抽象而调整接口。任何公共接口变更都必须在任务报告中说明。不应仅为假设中的兼容性保留错误设计。

M5 完成后再进行第一轮 API 稳定性评估。发布到 crates.io 不早于核心接口开始稳定。当前不得承诺语义化版本兼容性。

## 8. 密钥与安全规则

- API Key 只能来自环境变量、未提交的本地配置或 GitHub Secrets；
- 禁止把真实密钥写入源码、文档、测试夹具或日志；
- `.env` 后续必须被忽略；
- `.env.example` 只能包含变量名和说明；
- 默认测试必须使用 Mock；
- 真实网络测试必须默认忽略或手动触发；
- 错误信息不得包含 Authorization Header；
- 日志不得输出完整 API Key；
- 示例程序在缺少密钥时必须安全失败，不能 panic；
- 发现疑似密钥时必须停止扩展工作并报告。

## 9. 发布策略

- 每个 milestone 可以创建版本标签；
- P0 阶段不创建标签；
- M5 之前不急于发布 crates.io；
- 发布动作必须由明确任务触发；
- Codex 不得自行提交、推送、打 tag 或发布。

暂定标签形式：

```text
v0.1.0-m0
v0.2.0-m1
v0.3.0-m2
...
```

该形式仍可在首次发布前调整。

## 10. M0-M10 路线

| Milestone | 名称 | 可验收结果 |
|---|---|---|
| M0 | Model Call | 通过统一接口完成一次非流式模型调用 |
| M1 | Unified Protocol | 建立消息、内容块、能力、错误和响应模型 |
| M2 | Tool Runtime | 完成一次模型提出工具调用、运行工具并返回结果的闭环 |
| M3 | Agent Loop | 实现具有预算、状态和终止条件的基础 Agent 循环 |
| M4 | Context Engine | 实现上下文收集、排序、裁剪、压缩和来源追踪 |
| M5 | Sessions & Recovery | 使用事件日志和快照恢复中断任务并避免重复副作用 |
| M6 | Policy & Sandbox | 建立权限判断、审批、路径限制和命令执行边界 |
| M7 | Validation & Evidence | 对变更执行验证并产生结构化 Evidence |
| M8 | Observability | 记录统一 Trace、事件、成本、延迟和错误 |
| M9 | Loop Engineering | 实现进展判断、停滞检测、重试预算、升级和收敛控制 |
| M10 | Forge Studio Case | 将通用 Harness 用于一个简化的 Godot 变更任务 |

每个 milestone 的基本强制交付：

- 可编译实现；
- 至少一个可运行示例；
- 单元测试；
- 对应书籍内容；
- 当前限制和安全边界；
- CI 通过。

涉及状态或副作用的 milestone：

```text
M2
M5
M6
M7
M9
M10
```

这些 milestone 还必须考虑：

- 集成测试；
- 故障注入；
- 幂等性；
- 恢复或降级；
- Evidence 或 Trace。

多语言实现暂不作为每个 milestone 的强制门槛。

## 11. P0 范围

P0 包含：

- 仓库基线；
- Rust workspace；
- mdBook 骨架；
- 基础多语言工具链配置；
- CI；
- M0；
- P0 验收记录。

P0 不包含：

- Tool Runtime；
- Agent Loop；
- Session 恢复实现；
- Policy Engine 业务实现；
- Forge Studio 集成；
- Godot 集成；
- crates.io 发布；
- GitHub 仓库创建；
- 正式出版流程；
- 完整多语言参考实现。

本步骤是 P0-1，只建立仓库治理与决策基线，不执行 P0 中其他后续工作。

## 12. 尚未决定的事项

以下事项保持开放，不在本基线中替用户决定：

- 最终公开仓库名称；
- GitHub owner 和 repository URL；
- 正式作者显示名称；
- 16 章最终标题；
- 每章完整标题和排序；
- Leanpub 具体兼容方案；
- 自定义域名；
- 外部贡献者版权协议；
- 首次 crates.io 发布时间；
- 是否在 M5 后拆分独立 crate 发布；
- Forge Studio 案例的脱敏范围；
- 书籍商业出版时是否调整正文许可证策略。

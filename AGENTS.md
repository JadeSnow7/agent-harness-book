# AGENTS.md

## Repository Mission

本仓库服务于《从 0 搭建 AI Agent》的配套代码，以及 Forge Studio 通用 Agent Harness 的实验性底层原型。

仓库中的代码必须可运行、可测试、可解释。通用 Agent Harness 不得依赖 Forge Studio；Forge Studio 专属领域模型不得未经抽象直接进入通用核心接口。

## Source of Truth

发生冲突时，事实和要求的优先级如下：

1. 当前任务中的明确要求；
2. 本文件；
3. 已接受的 `docs/decisions/` 决策文档；
4. README 和章节计划；
5. 代码与测试现状。

如发现冲突：

- 不得静默选择其中一方；
- 必须在任务报告中指出冲突；
- 优先采取最小修改；
- 不得借冲突扩张任务范围。

## Before Editing

每次修改前必须：

- 检查当前工作目录；
- 检查 `git status`；
- 阅读相关决策文档和代码；
- 确认任务允许修改的文件；
- 检查是否存在用户未提交的修改；
- 不假设仓库为空；
- 不覆盖未知内容。

## Scope Discipline

- 只完成当前任务明确要求的内容；
- 不提前实现后续 milestone；
- 不借重构添加新功能；
- 不为未来假设创建复杂抽象；
- 不修改任务范围之外的文件，除非修复构建所必需；
- 如发生额外修改，必须在报告中解释原因。

## Code and Dependency Rules

- 公共 API 保持最小；
- 新依赖必须说明用途；
- 共享依赖使用根 `[workspace.dependencies]`；
- 默认禁止 Git dependency；
- 不引入完整 Agent 框架作为核心实现；
- 不为占位创建无意义公共函数；
- 不使用未验证实现冒充完成状态；
- 普通 milestone 不应无理由提高 MSRV。

## Testing Rules

- 不得删除或跳过测试来获得绿色结果；
- 默认测试不得访问真实网络；
- 测试不得依赖真实 API Key；
- 避免 `sleep` 和 timing race；
- 故障注入优先使用可控 failpoint；
- 无法运行的验证必须如实报告；
- 不得声称未实际执行的测试已经通过；
- 涉及跨平台进程故障时，不得只依赖 Unix `kill`。

## Security Rules

- 不提交 API Key 或其他真实凭据；
- 不打印敏感 Header，尤其是 Authorization Header；
- 不提交 `.env`；
- `.env.example` 只能包含变量名和说明；
- 不在 Mock fixture 中放入真实凭据；
- 不读取或发送与任务无关的用户文件；
- 示例程序缺少密钥时必须安全失败，不能 panic；
- 不擅自执行发布、推送或其他远程操作；
- 发现疑似密钥时，停止扩展工作并报告。

## Documentation Rules

- 代码、示例和书籍内容不能长期失去同步；
- 核心书籍代码应可追溯到测试源码；
- 重大公共接口变更必须更新文档；
- 当前限制和安全边界必须明确说明；
- 不得把规划中的能力写成已经实现；
- 不得伪造作者、远程地址、CI 结果、运行结果或发布状态。

## Git and External Actions

除非任务明确要求，否则禁止：

- `git commit`；
- `git push`；
- 创建或修改 remote；
- 创建或切换 branch；
- 创建 tag；
- 发布 crate；
- 创建 GitHub 仓库；
- 修改仓库可见性；
- 配置 DNS；
- 创建外部云资源。

允许进行只读 Git 检查，例如 `git status`、`git diff`、`git log`、`git branch` 和 `git remote -v`。

## Book Writing Rules

在编写、续写、审阅或重构 `book/src/` 正文之前，必须先阅读 [`docs/writing/style-guide.md`](docs/writing/style-guide.md)。作者已有正文的实际风格优先于模型默认文风；除非任务明确要求，不得把正文自动重写为通用 AI 技术文章、Reference 或产品说明。审阅应先区分技术/教学问题与风格问题，并优先报告后局部修改。

长期优先级为：当前任务明确要求 > 作者最新明确要求 > 作者稳定的人工写作模式 > Style Guide > 一般技术写作规范 > 模型默认文风。

## Completion Report

每次任务完成后必须报告：

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

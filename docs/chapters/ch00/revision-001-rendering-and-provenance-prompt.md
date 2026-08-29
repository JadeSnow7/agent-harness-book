# ch0 修订 001 Prompt

- **Revision：** 001
- **状态：** Approved
- **批准日期：** 2026-08-17
- **实施状态：** Completed
- **验收状态：** Pending review

<!-- BEGIN APPROVED PROMPT -->
任务名称：
修复 ch0 Mermaid 渲染与 Prompt 证据链

任务性质：
这是 ch0 的第一次修订，只修复两个已确认问题：

1. Mermaid 源文件已被 mdBook include，但生成页面只显示源码块，没有实际渲染图形；
2. docs/chapters/ch00/prompt.md 保存的是摘要，不是初次实施所依据的完整 Prompt。

不得借本次修订修改 ch0 的教学内容，不得进入 ch1/ch2。

一、事实基线

当前已确认：

- mdBook 版本为 0.5.4；
- 本机存在 mdBook、Node 和 npm；
- 本机不存在 mdbook-mermaid 和 mmdc；
- 仓库没有 package.json 或 Node 锁文件；
- book/book.toml 尚未配置 Mermaid；
- ch0.html 中的图被输出为 language-mermaid 代码块；
- ch0 正文、设计资产和两份 .mmd 文件已经存在；
- Python/Rust/Cargo 没有因 ch0 发生变化；
- docs/report.md 是任务开始前已经存在的未跟踪文件，不属于本任务。

二、目标

完成后必须满足：

1. ch0 的两张 Mermaid 图在 mdBook HTML 中实际显示为图形；
2. 构建产物不依赖在线 CDN；
3. 本地构建、CI 和 GitHub Pages 使用一致且固定的 Mermaid 构建工具版本；
4. 初次 ch0 Prompt 的完整原文得到保存，不再用摘要代替；
5. 本次修订 Prompt 也被独立保存；
6. Prompt、完成报告和人工验收结论彼此分离；
7. 不修改任何 Python/Rust 运行时代码。

三、技术选择

使用：

- mdbook 0.5.4；
- mdbook-mermaid 0.17.0。

选择理由：

- mdbook-mermaid 0.17.0 与 mdBook 0.5 系列对应；
- 官方安装会复制本地 Mermaid JS，并为 mdBook 配置预处理器；
- 页面不需要从 CDN 下载 Mermaid；
- 不引入 mmdc、Node package.json 或 Chromium；
- 该能力可以供后续章节的 Mermaid 图继续使用。

mdbook-mermaid 是文档构建期工具，不是 Agent Runtime 依赖。
mermaid.min.js 是随书分发的前端资产，应保留版本和许可证来源信息。

四、允许修改或新增的文件

允许修改：

- book/book.toml
- README.md
- .github/workflows/ci.yml
- .github/workflows/pages.yml
- docs/chapters/ch00/prompt.md
- docs/chapters/ch00/design.md

允许新增：

- book/mermaid.min.js
- book/mermaid-init.js
- docs/chapters/ch00/revision-001-rendering-and-provenance-prompt.md

条件性允许修改：

- book/src/assets/ch00/agent-harness-map.mmd
- book/src/assets/ch00/ai-coding-workflow.mmd

只有实际 Mermaid 渲染报告这两张 ch0 图存在语法错误时，才能进行最小语法修复。

默认禁止修改：

- book/src/ch0.md
- OUTLINE.md
- book/src/ch1.md 及后续章节
- examples/python/
- examples/rust/
- crates/
- Cargo.toml
- Cargo.lock
- fixtures/
- docs/adr/
- docs/specs/
- docs/report.md

如果修复需要修改默认禁止范围，立即停止并报告，不得自行扩大范围。

五、实施前检查

实施前必须：

1. 检查 pwd；
2. 检查 git status --short；
3. 检查 git diff；
4. 记录现有未提交变更；
5. 确认 docs/report.md 仍是任务外文件；
6. 确认当前不存在 mdbook-mermaid；
7. 确认本轮实际目标文件没有被其他未知修改覆盖。

不得清理、重置或暂存用户的其他变更。

六、安装与配置 Mermaid

如果本机尚未安装 mdbook-mermaid，执行：

cargo install mdbook-mermaid --locked --version 0.17.0

该命令需要访问网络。若执行环境要求批准，必须走权限审批，不得绕过。

然后运行：

mdbook-mermaid install book

检查该命令实际生成和修改的文件，只保留本任务批准的结果。

book/book.toml 最终应包含等价配置：

[preprocessor.mermaid]
command = "mdbook-mermaid"
after = ["links"]

[output.html]
additional-js = ["mermaid.min.js", "mermaid-init.js"]

after = ["links"] 用于保证 {{#include ...}} 先展开，再由 Mermaid 预处理器处理。

不得使用在线 CDN。
不得新增 package.json、package-lock.json 或 Chromium 依赖。

七、同步 CI 与 Pages

.github/workflows/ci.yml：

- 保留 mdBook 0.5.4 的固定安装；
- 在构建书籍前固定安装 mdbook-mermaid 0.17.0；
- 不改变 Rust、Python 或 MSRV 检查；
- 不改变其他 job 的权限和触发条件。

.github/workflows/pages.yml：

- 保留现有 Pages 权限、并发、artifact 和 deploy 结构；
- 在 mdBook 构建前固定安装 mdbook-mermaid 0.17.0；
- 不改变部署目标、分支或 Pages environment。

推荐命令：

cargo install mdbook-mermaid --locked --version 0.17.0

不得使用未固定版本的 cargo install。

八、同步 README 和设计记录

README.md：

- 将“构建书籍需要安装 mdBook”更新为同时需要：
  - mdBook 0.5.4；
  - mdbook-mermaid 0.17.0；
- 给出可复制的安装与构建命令；
- 简要说明 Mermaid 由本地 JS 渲染，不依赖 CDN；
- 不扩写成完整前端工具教程。

docs/chapters/ch00/design.md：

补充：

- Mermaid 渲染方案；
- mdbook-mermaid 和 Mermaid JS 的用途；
- 版本固定原因；
- 构建期工具与 Agent Runtime 依赖的区别；
- Mermaid JS 的来源和许可证；
- 浏览器级验收标准；
- 全局渲染会覆盖旧章节 Mermaid 图，但本轮不修改旧章节内容。

九、修复初次 Prompt 证据链

docs/chapters/ch00/prompt.md 必须保存初次 ch0 实施 Prompt 的完整原文。

要求：

1. 使用此前实际拟定的完整“ch0 实施 Prompt（待批准）”；
2. 不根据最终实现改写 Prompt；
3. 不压缩为摘要；
4. 不把完成报告写进 Prompt 原文；
5. 不伪造缺失的审批时间、审批措辞或外部记录；
6. 如果审批证据没有保存在仓库，明确标注：
   “审批证据未入库；当前文件保存的是实施前拟定的完整 Prompt 原文。”
7. 状态、实施结果和验收结论放在 Prompt 原文之外。

建议结构：

- 章节；
- Prompt 类型；
- Prompt 原文状态；
- 审批证据状态；
- 实施状态；
- 验收状态；
- BEGIN APPROVED PROMPT；
- 完整 Prompt 原文；
- END APPROVED PROMPT。

BEGIN/END 标记之间的文本必须保持原样。

十、保存本次修订 Prompt

新增：

docs/chapters/ch00/revision-001-rendering-and-provenance-prompt.md

内容必须是本次经用户批准的修订 Prompt 完整原文，不得保存摘要。

在用户明确批准之前：

- 不创建该文件；
- 不实施修改。

批准后可以在原文之外记录：

- revision：001；
- 状态：Approved；
- 批准日期；
- 实施状态；
- 验收状态。

不得把本次完成报告写入 Prompt 原文。

十一、图形和可访问性要求

两张 ch0 图必须：

- 在桌面浏览器中完整显示；
- 中文标签不被截断；
- 箭头方向清楚；
- 系统架构图与 AI Coding 工作流图承担不同认知任务；
- 状态不能只依赖颜色表达，文字标签和虚线仍要保留；
- 浅色和深色主题下均可读；
- 浏览器控制台没有 Mermaid 错误；
- 页面无需访问 CDN。

本轮不重新设计图的内容或视觉风格。
只有实际渲染暴露语法、裁切或不可读问题时，才最小修改对应 ch0 .mmd 文件。

十二、旧章节影响边界

启用 Mermaid 渲染后，ch1、ch2、ch5 的既有 Mermaid 块也会开始渲染。

必须检查：

- 是否出现 Mermaid 语法错误；
- 是否导致 mdBook 页面加载失败；
- 是否出现明显裁切；
- 浏览器控制台是否报错。

如果旧章节图存在问题：

- 记录具体章节、文件和错误；
- 不在本轮修改；
- 将其列为单独遗留问题；
- 不以“顺手修复”为由扩大范围。

十三、验证

至少执行：

1. git diff --check；
2. git status --short；
3. 检查变更文件是否严格位于允许范围；
4. 检查没有 Python/Rust/Cargo 变化；
5. 将 mdBook 构建到新的临时目录；
6. 确认 ch0.html 存在；
7. 确认生成页面加载本地 mermaid.min.js 和 mermaid-init.js；
8. 确认没有 Mermaid CDN URL；
9. 确认 ch0 两张图不再只是普通 language-mermaid 代码块；
10. 通过本地 HTTP 服务打开构建结果；
11. 在浏览器中检查两张图实际显示；
12. 检查浏览器控制台；
13. 检查浅色和深色主题；
14. 检查 ch1、ch2、ch5 的已有 Mermaid 图；
15. 检查 CI 与 Pages 都固定安装相同版本；
16. 检查 prompt.md 保存完整初次 Prompt；
17. 检查 revision-001 Prompt 保存完整本次 Prompt。

浏览器验收必须是真实页面检查，不能只搜索 HTML 字符串。

十四、完成状态边界

完成后可以声明：

- ch0 Mermaid 图已在 mdBook HTML 中实际渲染；
- 本地构建使用固定版本的 mdbook-mermaid；
- CI 和 Pages 已同步构建前置条件；
- 初次 Prompt 与修订 Prompt 已完整保存；
- 没有 Python/Rust 运行时代码变化。

不得声明：

- GitHub Actions 已通过，除非实际远程运行；
- GitHub Pages 已更新，除非实际部署并验收；
- 所有 Mermaid 图都已完成视觉精修；
- Agent Runtime 能力发生变化；
- ch1/ch2 已经开始。

十五、完成报告

完成后必须报告：

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

额外报告：

- mdbook-mermaid 与 Mermaid JS 的实际版本；
- 两张 ch0 图的浏览器验收结果；
- 是否访问任何 CDN；
- 旧章节 Mermaid 图检查结果；
- Prompt 原文是否完整；
- 是否存在未经批准的范围扩张；
- CI/Pages 只做了本地静态修改，还是已有远程运行证据。

十六、禁止事项

- 不提交、推送、切换分支或创建标签；
- 不部署 Pages；
- 不修改 remote；
- 不修改 Python/Rust/Cargo；
- 不处理 docs/report.md；
- 不删除或重写现有证据链；
- 不提前进入 ch1/ch2；
- 不使用 CDN；
- 不新增 Node package 或 Chromium；
- 不伪造审批、CI、Pages 或浏览器验收结果；
- 不通过跳过检查获得绿色结果。
<!-- END APPROVED PROMPT -->

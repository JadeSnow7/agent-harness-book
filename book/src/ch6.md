# 第 6 章：Context Engineering：模型本轮应该看到什么

**状态：设计骨架/尚未实现。** 本章定义 M4 的 ContextBuilder 和预算语义；P0 的 `SimpleContextBuilder` 只是确定性参考。

上一章的循环每次都把历史交给模型。两个工具调用时没有问题；长任务里，原始消息、工具输出、文件内容和错误会很快超过预算。更糟的是，全部保留并不等于模型会正确使用全部内容。

## 6.1 学习目标

完成 M4 后，读者应能为上下文来源排序、保护必需项、显式处理超限，并用固定 fixture 证明同一输入产生同一 ModelInput。

## 6.2 先写一个具体的 Context Builder

不要先设计“记忆系统”。先定义本轮输入的来源和顺序：系统规则、用户目标、最近动作、最近观察、与当前工具相关的项目上下文。每项带名称和来源，超过预算时从低优先级开始删除。

```rust
struct ContextItem { source: String, text: String, priority: u8 }

fn build(items: Vec<ContextItem>, max_items: usize) -> Vec<ContextItem> {
    let mut items = items;
    items.sort_by_key(|item| std::cmp::Reverse(item.priority));
    items.truncate(max_items);
    items
}
```

真实实现不能只按条目数截断，还要估算 token 或字节，并保护必需项。`SimpleContextBuilder` 已展示了有序、限额的 P0 形态，但不应被称为完整 Context Engine。

## 6.3 上下文不是数据库

对话历史、任务状态、项目事实和工具输出有不同生命周期：历史可以摘要，状态应结构化保存，项目事实需要来源，工具输出需要截断或归档。把它们都拼成字符串，会让“丢了什么”无法解释。

```text
Context Item
├── source: request | event | project | retrieval
├── priority
├── size
└── freshness / required
```

压缩也不是删除。摘要必须留下它覆盖了哪些原始事件，并允许在必要时重新检索。检索则不是“把整个仓库塞进窗口”，而是根据当前问题选择候选片段。

## 6.4 硬预算与降级

上下文无法表示时有三种策略：压缩、检索或终止。不能静默丢掉用户目标和安全规则。代码中应返回 `ContextError` 或结构化 `BudgetExhausted`，并在事件中说明降级路径。

```text
required items > budget → fail explicitly
optional items too large → summarize or drop
recent tool error → retain until observed
```

## 6.5 代码增量与验证

测试不需要模型。准备十个带优先级的 fixture，断言必需项始终出现、顺序稳定、总预算不超、同一输入得到同一结果；再准备一个“必需项本身超限”的场景，确认程序不会假装成功。P0 的事件和 `ModelInputBuilt` 可作为后续来源追踪的起点。

当前 P0 参考组合：

```text
事件历史 + Request + Tool Specs
          ↓
   ContextBuilder（有序、限额）
          ↓
       ModelInput
```

验证场景：准备带来源和优先级的 fixture，断言必需项始终出现、顺序稳定、总预算不超；再测试“必需项本身超限”，确认程序显式失败而不是静默丢弃。

## 6.6 本章将得到什么

完成 M4 实现后，系统将拥有可解释的上下文选择、预算和降级路径；它仍不是完整的长期记忆或检索平台。

下一章把“上下文里的一段历史”变成有身份的 Session、Task、Run 和 Step。

## 6.7 上下文预算把“看得到”变成了工程决策

本章在循环之上增加的不是长期记忆，而是一次 `ModelInput` 的选择规则：哪些内容必须保留，哪些可以裁剪，超限时如何显式失败。收益是输入不再由“把所有历史拼起来”决定；来源、优先级、顺序和预算可以被测试，模型每轮看到什么也终于成为可解释的运行状态。

代价在于裁剪本身可能制造错误。删掉一个看似旧、实际仍决定安全边界的事件，会让模型在缺事实时继续行动；压缩摘要若没有覆盖范围，后续恢复无法知道丢了什么；按条目数代替 token/字节估算，也会给出虚假的预算保证。Context Builder 还容易吸收数据库、检索和记忆职责，形成过早的大抽象。

| 能力 | 当前状态 |
| --- | --- |
| P0 `SimpleContextBuilder` 的有序、限额构造 | P0 参考实现，已测试 |
| 独立 M4 的来源追踪、压缩和降级路径 | 设计骨架/尚未实现 |
| durable memory、retrieval、token 精确计费 | 尚未实现 |

当前成熟度是 **Experimental**：问题模型和测试合同已清楚，P0 有最小形态，但还没有可独立复用的 Context Engine。这里有意留下的技术债是先把上下文当成纯构造函数，不引入向量数据库或跨会话记忆。下一章的必要性随之出现：即便本轮输入构造得再好，进程重启后也没有可靠身份来说明这段历史属于哪个任务、哪次尝试、哪一步。

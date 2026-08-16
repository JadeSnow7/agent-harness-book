# 第 9 章：Side Effect、ChangeSet 与 Mutation

**状态：设计骨架/尚未实现。** 本章只负责把动作变成可审查的变更提案；审批决策和执行隔离属于下一章。

前面的 `write` 和 `edit` 能完成教学任务，但它们把提议和执行放在同一个动作里。文件一旦改变，审查者只能事后看 diff。于是我们先区分三个概念：读取、动作、变更。

## 9.1 学习目标

完成 M6 的变更部分后，读者应能生成带原内容指纹、diff、原因和风险的 ChangeSet，并在原文件变化后拒绝应用。

## 9.2 读取不等于修改

`read` 是观察；`write` 是可能产生副作用的 mutation；模型输出的“请改文件”只是 Action proposal。更安全的路径是：

```text
Model Action → ChangeSet → Review/Policy → Apply → Validate
```

`ChangeSet` 可以包含目标、原内容指纹、新内容、diff、原因和风险级别。它不是把权限藏进一个新结构，而是让“将要改变什么”成为可以比较、拒绝和审计的数据。

```rust
struct Change {
    path: String,
    expected_hash: String,
    replacement: String,
}
struct ChangeSet { id: String, changes: Vec<Change>, reason: String }
```

这只是教学形状，hash 算法和持久化格式尚未冻结，标记为 `VERIFY`。

## 9.3 apply 的两个检查

应用前重新检查原内容指纹，防止模型生成 diff 后文件已被别人修改；应用时尽量原子替换，并让每个文件的结果进入事件。全部文件是否要事务化，取决于环境能力：普通文件系统通常只能做到逐文件原子，不能宣称跨文件事务。

本章只记录审批需要看到的材料，不实现审批状态。真正的 Allow、Deny、Ask、审批 id 绑定和 Sandbox 强制边界放在第 10 章。

## 9.4 验证

准备一个文件，生成 ChangeSet，验证 diff；改变原文件后 apply 必须拒绝；拒绝后文件不变；批准后只应用一次；验证失败时保存原始和最终状态。当前 M2 `write/edit` 的原子替换可作为局部参考，但它们不是 ChangeSet subsystem。

当前 Harness：

```text
ToolCall → Mutation Proposal / ChangeSet
                    ↓
              Approval + Policy
                    ↓
                Apply → Validator
```

验证场景：生成 ChangeSet 后验证 diff；改变原文件后 apply 必须拒绝；拒绝后文件不变；批准前不能调用 apply；应用后保存原始和最终状态。当前 M2 `write/edit` 只作为局部参考，不是 ChangeSet subsystem。

## 9.5 本章将得到什么

完成 M6 变更部分后，系统将把“将要改什么”变成可比较、可拒绝、可审计的数据。下一章再决定哪些能力允许真正执行，以及如何建立执行边界。

## 9.6 把副作用变成数据，审查才有落点

本章试图改变的是动作的形态：从“工具调用立即写入”变成“先生成带原内容指纹和 diff 的 ChangeSet”。收益是变更可以在执行前被比较、拒绝和记录；原文件在提案后发生变化时，指纹检查可以阻止把旧依据覆盖到新状态上。这为人工审批、Policy、验证和后续证据提供了共同对象。

代价也很清楚。生成 diff 不等于理解语义，hash 不等于解决所有并发问题，逐文件原子替换也不等于跨文件事务。恶意或错误的内容仍可能藏在一个看似合理的 ChangeSet 中；路径、symlink、编码和大文件处理都需要独立边界。AI 特有的风险是模型可以给出看似完整的修改理由，却没有真正满足目标，因此 ChangeSet 不能取代 Validation。

| 能力 | 当前状态 |
| --- | --- |
| M2 `write`/`edit` 的局部原子写入 | 已实现并验证，但不是 ChangeSet subsystem |
| `ChangeSet`、`Mutation`、指纹和 apply contract | 设计骨架/尚未实现 |
| 审批状态、跨文件事务、回滚和持久化 | 尚未实现 |

当前成熟度是 **Experimental**：本章冻结了可审查变更的方向，却没有实现或冻结 hash 格式和持久化协议。有意留下的技术债是先不把审批混进 ChangeSet；否则读者会同时学习变更表示和权限决策，无法看清读写分离本身。下一章由这个分离自然产生：有了“想改什么”，还必须回答“谁允许它真正执行”。

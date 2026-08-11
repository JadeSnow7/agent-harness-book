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

# 第 9 章：Effect、ChangeSet 与可审查的变更

**状态：已实现并验证。** M6 的 Python 实现只在内存字符串上演示变更和一次应用；它不是文件系统编辑器，也不是浏览器执行器。

## 9.1 ToolUse 不是执行结果

模型提出 ToolCall 后，Runtime 先查 ToolDescriptor。read 与 effect 是两种不同的风险；EffectIntent 描述 call、tool、domain、kind、scope、run 和参数。ChangeSet 保存 before、after、diff、reason、risk 和 expected_hash，领域数据只能通过 Projection/domain payload 承载。

~~~python
change = ChangeSet.create("note.txt", "old\n", "new\n", "fix", "low")
result = EffectApplier().apply(change, "old\n")
assert result["after"] == "new\n"
~~~

## 9.2 变更必须可审查且防陈旧

expected_hash 不匹配时拒绝应用；同一 ChangeSet 的再次应用复用记录，避免重复修改。成功路径是 before 与 expected_hash 一致；重要失败路径是陈旧状态和重复/不一致的变更身份。EffectPipeline 展示 allow 决定之后才调用 applier，但不提供真实外部 side effect。

## 9.3 验证命令、实现索引与下一步

~~~bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python \
  python3.11 -m unittest discover \
  -s examples/python/m6-effects-policy -p 'test_*.py'
~~~

源码：effects.py（ChangeSet、Projection、Mutation、EffectPipeline、EffectApplier）；测试：examples/python/m6-effects-policy/test_effects.py。M6 的收益是把“想改什么”和“改成什么”分开；代价是 diff 仍是字符串级，缺少领域适配器和 durable ledger。下一章将把工具声明、策略和审批接入真正的执行顺序。

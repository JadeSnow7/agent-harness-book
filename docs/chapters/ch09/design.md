# CH09 / M6 Effect 设计记录

ChangeSet 是通用可审查变更，Projection 承载领域字段；read/effect 由 ToolDescriptor 明确区分。EffectApplier 只接受 expected_hash 匹配的当前值，并对相同变更复用内存结果。

实现位于 tutorial/python/agent_harness/effects.py，M6 测试位于 examples/python/m6-effects-policy/test_effects.py。真实文件、浏览器、网络副作用和生产级 sandbox 都不在本章范围。

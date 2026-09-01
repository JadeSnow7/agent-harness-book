# CH15 / M10 Extension 设计记录

Manifest 是扩展的 capability 声明，Lifecycle 是可调用状态；二者都不能替代 Runtime、Policy、Validation 和 Evidence。FakeMCP/FakeSkill/FakeHook/FakePlugin 只提供确定性离线行为。

源码 tutorial/python/agent_harness/extensions.py；测试 examples/python/m10-extension-composition/test_extensions.py。生产级插件隔离、签名、版本解析和真实 MCP 连接明确不在范围内。

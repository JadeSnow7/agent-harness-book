# CH06 / M4 设计记录

M4 的最小合同是确定性上下文选择：ContextItem 携带来源、priority、freshness 和 required；ContextBuilder 以 UTF-8 字节预算产生 included/summarized/omitted 决定。必需项无法容纳时 fail closed。

源码是 tutorial/python/agent_harness/context.py，独立测试是 examples/python/m4-context-engineering/test_context.py。该实现是内存教学切片；工具 schema 成本、真实 token 估算、检索和 provider continuation 仍在边界外。与 HUSH 的关系仅是输入合同和 provider-neutral 方向对齐，不是 Runtime 集成证明。

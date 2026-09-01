# CH14 / M9 Delegation 设计记录

SubRun 需要独立 run_id、parent_run_id、目标、能力清单、预算和结构化结果。SerialDelegateExecutor 在内存中检查父子身份和结果证据；aggregate 拒绝失败 child、重复 child、空输入和不相干 parent。

源码 tutorial/python/agent_harness/delegation.py；测试 examples/python/m9-loop-delegation/test_delegation.py。Codex/Claude 等开发子代理不属于书中 SubRun。

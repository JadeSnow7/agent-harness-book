# CH12 / M8 Evidence 设计记录

Evidence 只记录可追踪材料，supporting_events 必须存在；Trace 和 Summary 是只读 projection，不改变运行状态。ReviewBundle.digest 绑定 ChangeSet、Validation 和 Evidence，旧摘要变化后 approval 不应继续适用。

源码 tutorial/python/agent_harness/evidence.py 与 _base.py；测试 examples/python/m8-evidence-observability/test_evidence.py。脱敏是教学防线，不等于生产 secrets manager 或完整 observability。

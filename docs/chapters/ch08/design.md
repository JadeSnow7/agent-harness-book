# CH08 / M5 Recovery 设计记录

本章冻结 Reserved/InProgress/Completed/Failed/Ambiguous 的区别。副作用前必须 reserve，无法确认结果时必须 Ambiguous 且停止自动重放；intent_digest 同时绑定运行身份和规范化请求内容。

源码是 tutorial/python/agent_harness/idempotency.py 与 recovery.py。验证覆盖身份不匹配、重复结果、failpoint 决策和并发声索；存储仍为内存教学实现，不是生产级 exactly-once。

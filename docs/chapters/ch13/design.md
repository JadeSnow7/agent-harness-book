# CH13 / M9 Loop 设计记录

StopPolicy 统一预算、停滞、取消和升级原因；Runner 只在 Validation 通过时产生 Completed。重复 call_id、provider error、schema error 和 policy escalation 都保留为结构化结果。

源码 tutorial/python/agent_harness/stop_policy.py、runner.py；验证为 tutorial/python/agent_harness/test_core.py。循环是内存 fake-provider 教学实现，不是生产 agent scheduler。

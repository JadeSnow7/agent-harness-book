# CH07 / M5 设计记录

事件必须带有不可混用的 Identity、连续 seq 和可追踪 event_id。EventLog 的 replay 只读，不重新调用模型或工具；Snapshot 只保存版本化状态投影。当前实现是内存教学版本，不宣称 durable recovery。

对应源码为 tutorial/python/agent_harness/identity.py、events.py、snapshots.py，验证为 examples/python/m5-session-recovery/test_session.py。Recovery 和幂等状态机在 CH08 接上这一事件边界。

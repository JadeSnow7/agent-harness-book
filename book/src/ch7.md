# 第 7 章：Session、Task、Run、Step 与事件事实

**状态：已实现并验证。** M5 只做内存事件流和可序列化 Snapshot；它不是 crash-safe 数据库。

## 7.1 身份先于历史

Session 是交互范围，Task 是目标，Run 是一次尝试，Step 是运行中的序号。tutorial/python/agent_harness/identity.py 的 Identity 将 session_id、task_id、run_id 绑定在一起；models.py 保留 Session、Task、Run、Step 的教学对象。跨 run 的事件不能进入同一 EventLog。

~~~python
identity = Identity("session-1", "task-1", "run-1")
log = EventLog(identity)
log.append(EventEnvelope(1, "run.started", identity, {"goal": "read"}))
~~~

## 7.2 EventEnvelope 是事实，replay 是只读

EventEnvelope 带 seq、event_id、schema、Identity、payload 和 terminal。EventLog 只允许连续追加，并在 terminal 后拒绝写入；events 属性返回不可变视图。replay 只校验并返回记录，绝不调用 provider 或 tool。

成功路径是顺序追加后调用 replay；重要失败路径是序号跳跃、跨身份追加和终态后追加。Snapshot.from_dict 可以恢复一个明确版本的状态，但恢复不等于重新执行。

~~~python
snapshot = Snapshot(1, "v1", {"status": "running"})
assert Snapshot.from_dict(snapshot.to_dict()) == snapshot
assert log.replay() == log.events
~~~

## 7.3 离线验证、收益与技术债

~~~bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python \
  python3.11 -m unittest discover \
  -s examples/python/m5-session-recovery -p 'test_*.py'
~~~

实现入口：identity.py、models.py、events.py、snapshots.py；测试：examples/python/m5-session-recovery/test_session.py。收益是身份和事实边界可测试；代价是事件仍在进程内，快照没有迁移工具，事件 payload 也需要上层定义 schema。下一章处理副作用执行中断后的幂等与恢复。

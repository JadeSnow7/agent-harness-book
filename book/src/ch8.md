# 第 8 章：Recovery 与幂等：不知道就不要重放

**状态：已实现并验证。** M5 的 recovery 代码展示状态机和内存 append-only ledger；它不是跨系统事务。

## 8.1 reservation 是副作用前的门

idempotency key 定位一个槽位，intent_digest 证明请求身份。IdempotencyLedger.reserve 首次返回 RESERVED 和 token；同一 key/intent 的其他声索只能得到 IN_PROGRESS，不能获得第二个执行资格。Completed 复用结果，Failed 记录已知失败，未知结果进入 Ambiguous。

~~~python
ledger = IdempotencyLedger()
claim = ledger.reserve("run-1:call-1", identity, digest(intent))
ledger.complete("run-1:call-1", {"status": "ok"}, claim.token)
assert ledger.require_result("run-1:call-1")["status"] == "ok"
~~~

## 8.2 恢复决定不是自动重试

RecoveryDecision 将已知失败映射到 retry，将执行后故障映射到 resume；UNKNOWN 或 Ambiguous 映射到 stop。相同 key 的不同 intent 会抛出 IdentityMismatch。重要失败路径是错误 token、身份/摘要不匹配和 ambiguous 后再次尝试；Ambiguous 没有自动 replay。

并发测试用 32 个线程声索同一 key，证明只有一个 RESERVED。该证据证明的是本进程锁的教学行为，不是多进程或崩溃持久化。

## 8.3 验证命令与边界

~~~bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python \
  python3.11 -m unittest discover \
  -s examples/python/m5-session-recovery -p 'test_*.py'
~~~

实现入口：idempotency.py、recovery.py、identity.py；测试：examples/python/m5-session-recovery/test_session.py。技术债是没有 JSONL 恢复、租约过期和外部事务协调；这些不能用“测试通过”替代。下一章把候选工具接到 EffectIntent、Policy 和 ChangeSet。

# 第 6 章：Context Engineering：模型本轮应该看到什么

**状态：已实现并验证。** 本章的 M4 是 Python 3.11 标准库教学实现；它解决“输入如何稳定地进入窗口”，不声称提供检索平台、长期记忆或 tokenizer。

## 6.1 从聊天历史到有预算的上下文

前一章已经能调用模型，但把所有历史、工具 schema 和结果直接拼接会导致超窗、顺序漂移和秘密泄露。M4 把输入拆成带来源和优先级的 ContextItem，再由 ContextBuilder 在明确的字节预算内选择。

ContextSource = Source 是教学别名；ContextPriority 给出可读的优先级；ContextBudget 只表达预算，不是模型真实 token 计数。

~~~python
from agent_harness import ContextBudget, ContextBuilder, ContextItem, Source

result = ContextBuilder(ContextBudget(64)).build([
    ContextItem("rule", "be safe", Source.SYSTEM, required=True),
    ContextItem("hint", "offline", Source.USER),
])
~~~

选择顺序是“必需项、priority、freshness、source、key”，因此相同输入得到相同顺序。正文的 UTF-8 字节预算和工具 schema 的 ToolDescriptor.input_schema 成本都应计入上层 request；本切片只对上下文条目计数。

## 6.2 裁剪不是静默删除

普通长条目可以在剩余预算足够时摘要；短条目或预算不足时记录 omitted。必需条目放不下直接 ContractError，而不是把失败伪装成成功。

~~~python
result = build_context(items, budget_bytes=200)
assert result.used_bytes <= result.budget_bytes
assert result.decisions  # 每个条目都有决定
~~~

成功路径是 included；重要失败路径是必需项超限和负预算。另一个边界是多字节文本：预算按 len(text.encode("utf-8"))，不会把半个字符当作一个有效输入。

## 6.3 Provider continuation 的边界

ConversationContinuation 在 protocol.py 中只保存 provider 与 opaque state。previous_response_id 可以是某个 OpenAI adapter 的策略，但不是 core 的字段语义；手工历史也必须保留 Provider 输出要求的消息项。离线 fake provider 不证明真实多轮 API 行为。

## 6.4 离线验证与实现索引

~~~bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tutorial/python \
  python3.11 -m unittest discover \
  -s examples/python/m4-context-engineering -p 'test_*.py'
~~~

实现入口：tutorial/python/agent_harness/context.py（ContextItem、ContextBudget、ContextBuilder、build_context）；测试：examples/python/m4-context-engineering/test_context.py。本章实现是内存、确定性 P0 教学切片，不是生产级上下文服务。

收益是预算、来源和降级都有可观察决定；代价是摘要规则很粗，不能替代 tokenizer 或语义压缩。下一章必须给这些消息和结果加上 Session、Task、Run、Step 身份，并回答“重放是否会重新调用模型”。

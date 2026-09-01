# CH10 / M6 Policy 设计记录

ToolDefinition 是模型侧描述，ToolDescriptor 是 Runtime 合同。输入 schema 必须先于 Policy 和 ledger；副作用必须先 reserve 再 execute；输出 schema 失败不能伪装成功。

policy.py 的 allow/deny/ask 与 ApprovalStore 是内存教学实现。Approval 通过三类摘要绑定变更、验证和证据；这与 HUSH ToolDescriptor、EffectIntent、PolicyDecision、ReviewBundle 方向对齐，但没有接入 HUSH Runtime。

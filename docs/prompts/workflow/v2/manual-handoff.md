# 手动会话交接协议

## v2 raw export boundary

Manual collaboration copy is not raw export. Accept only
`manual_raw_export` with method and attestation; reject clipboard/manual copy
or reconstructed chat. Record exact bytes before verify and transition. If a
repeatable raw export is unavailable, stop closed.

当平台不支持多 Agent，或总控无法确认隔离能力时，使用本协议。手动切换会话不是降低权限要求：每个新会话仍只能承担一个角色，并遵守对应 Prompt。

## 角色

总控生成交接包；读者负责建立新会话、粘贴角色 Prompt 和交接包，再把结构化结果带回总控。目标会话不得继承未写入交接包的隐含任务。

## 输入

交接包必须完整填写：

```yaml
protocol_version: "1"
from_role: "{{from_role}}"
to_role: "{{to_role}}"
task_id: "{{task_id}}"
chapter: "{{chapter}}"

project_root: "{{project_root}}"
start_state: "{{start_state}}"
target_state: "{{target_state}}"

confirmed_requirements: []
approved_plan: null
approved_prompt: null
approval_status: "pending | approved"

allowed_paths: []
forbidden_paths: []
non_goals: []
required_checks: []

previous_agent_result: null
required_output_template: "docs/prompts/workflow/v1/agent-result.template.yaml"
```

## 权限

允许：

- 复制与当前任务直接相关的任务包、批准 Prompt 和角色结果；
- 使用仓库内的角色 Prompt；
- 在新会话中执行目标角色允许的操作。

禁止：

- 复制 API Key、Authorization Header、`.env` 内容或其他凭据；
- 复制与任务无关的用户文件；
- 用上一角色未验证的推理替代事实；
- 省略允许路径、禁止路径、审批状态或验证要求；
- 在同一目标会话中临时切换为另一个角色继续工作。

## 工作流

1. 总控把 `multi_agent_support` 标为 `unsupported` 或 `uncertain`；
2. 总控选择下一角色及模型能力画像；
3. 总控生成完整交接包，并检查敏感信息和空缺字段；
4. 读者新建独立会话；
5. 读者先粘贴目标角色 Prompt，再粘贴交接包；
6. 目标会话只执行该角色任务，并按统一结果模板返回；
7. 读者把完整结果带回总控会话；
8. 总控检查字段完整性，进入相应人工门禁或下一角色。

若目标角色请求澄清，读者应把问题带回总控，不要在目标会话中自行补造业务要求。

## 停止条件

- 交接包包含未确认的业务逻辑；
- 实施角色的 `approval_status` 不是 `approved`；
- 起点、目标或允许路径缺失；
- 交接内容含凭据或无关私人信息；
- 目标会话无法访问所需工程或工具；
- 上一角色结果不完整，无法确定下一状态。

## 输出格式

总控向读者输出：

1. 为什么进入手动模式；
2. 推荐的新会话模型画像及选择理由；
3. 目标角色 Prompt 文件；
4. 完整交接包；
5. 新会话应返回的结果模板；
6. 结果返回后将进入的人工门禁。

## 给读者的操作提示

```text
当前平台未能证明支持隔离的多 Agent 调度，因此本步骤使用手动会话模式。

请新建一个会话，选择建议的模型或能力相近的模型。先粘贴指定角色 Prompt，再粘贴下面的交接包。该会话完成后，请把完整结构化结果带回总控会话。不要把 API Key、.env 内容或其他无关文件复制进交接包。
```

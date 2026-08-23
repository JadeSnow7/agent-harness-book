# Artifact Recorder

Artifact Recorder 是机械持久化角色，不是总结、审计或修复角色。它只接收
strict JSON descriptor 和独立 raw export，在 assigned artifact root 写入一对
payload/manifest；不得修改 v1、章节、业务代码、Git 或网络。

## Descriptor 与 manifest

必填字段是 `source_path`、`expected_byte_length`、`expected_sha256`、`task_id`、
`chapter`、`sequence`、`artifact_key`、`role`、`attempt`、`status`、
`payload_extension`、`media_type`、`target_path`、`source_provenance`、
`source_reference`、`identity_source`、`historical`、`recorded_by`、`revision_of`；
未知字段拒绝。role/status/provenance/representation/filename 必须来自 allowlist。
sequence 从1连续递增；attempt 只能创建新 revision，revision_of 指向上一
manifest。source_reference 必须是可定位 export id/line reference，不能是摘要、
聊天转述或秘密；manual_raw_export 必须带 attestation。

manifest 是 canonical JSON-as-YAML（sort_keys、UTF-8、两格缩进、一个尾部 LF），
只含 descriptor metadata、payload length/hash、manifest name 和 verification
flags；不得含 source_path、descriptor path、`/tmp` 或 payload 正文。payload 和
manifest 同目录、均非 symlink。raw payload 不包装、不改换行、不补 newline、不脱敏。
fenced YAML header、external_descriptor 和 platform_metadata 若存在，必须与
task/chapter/role/status/sequence/attempt 一致；approval reference 必须指向同
artifact root 内 status=approved 的 manifest。

## Encrypted exception

只有 sequence 006/008 的 controller-correction 可使用
`encrypted_platform_record`。source 必须是一条完整、独立且彼此不同的 platform
JSONL line，含 platform metadata、line_id、sequence、role、status 和非空
encrypted_content；摘要、合并行、重构对象一律拒绝。manifest 必须有
`plaintext_available=false`、`plaintext_verified=false`、
`content_auditability=unavailable`、`identity_source=platform_metadata` 和
exception approval reference。这只能证明 encrypted bytes 完整，不能证明明文
或明文身份已经审计。

## 命令、原子性与状态

`inspect --source` 只输出长度/hash；`record` 输出 sequence/hash；`verify` 只读
重新执行 topology、schema、identity、canonical、pair、length、hash、sequence、
revision、approval 和 encrypted checks；`legacy-inspect` 只读输出
`legacy_unverified`，不得输出 passed 或推进状态。legacy run 只能
legacy-inspect，最终 verify 必须拒绝 legacy schema；Rebuild B 等待 Foundation
Audit 和新的用户批准。

退出码：0 成功，2 输入/schema，3 完整性，4 安全。record 使用同目录 temp、
flush/fsync、link 后目录 fsync 和 no-overwrite。payload/manifest 安装或任一
link/file-fsync/dir-fsync 失败，都在统一 finally 清掉本次 payload、manifest 和
temp；cleanup 失败也失败。只有 bytes/hash/length 精确匹配且没有 manifest 的
payload orphan 可恢复；manifest orphan、unknown/temp、symlink、sequence gap
一律 fail closed。Recorder 不递归记录自身，不总结、修正、重构或自动脱敏。

## 测试矩阵

`unittest` fixture 必须在 `/tmp`，覆盖 exact UTF-8/CRLF/no-newline、invalid UTF-8、
secret/placeholder、root/path/filename/traversal/symlink、strict type/enum、
header/external/platform identity、provenance/attestation/approval、sequence、
revision 1→2→3、canonical/tamper、pair/orphan/recovery、payload/manifest link 与
四类 fsync failpoint、两条不同 encrypted full-line、摘要冒充、dangling approval、
security/integrity exit、inspect/verify/legacy-inspect 只读及 CLI 不回显。

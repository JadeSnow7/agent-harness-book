# CH11 / M7 Validation 设计记录

ValidationReport 是终态门：模型的 ModelFinished 不直接转换为 Completed。检查按稳定名称聚合，异常结构化为 validator_error，空检查集 fail closed。

源码 tutorial/python/agent_harness/validation.py；测试 examples/python/m7-validation/test_validation.py。该 validator 是确定性教学接口，不能证明真实项目状态；Evidence 引用在 CH12 增加。

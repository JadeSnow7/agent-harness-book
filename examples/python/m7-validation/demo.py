from agent_harness import Validator
print(Validator({'nonempty': lambda x: bool(x)}).validate('answer').passed)

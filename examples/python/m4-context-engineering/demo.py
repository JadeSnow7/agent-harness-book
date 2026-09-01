from agent_harness import ContextItem, Source, build_context
result = build_context([ContextItem("rule","be safe",Source.SYSTEM,10,10,True), ContextItem("hint","offline",Source.USER,1,1)], 64)
print("\n".join(item.text for item in result.items))

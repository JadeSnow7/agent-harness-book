from agent_harness import CapabilityManifest, FakeMCP, Lifecycle
m=CapabilityManifest('fake','1','demo',frozenset({'read'}),{'type':'object'}); x=FakeMCP(m, Lifecycle.ACTIVE); print(x.invoke({'ok':True}))

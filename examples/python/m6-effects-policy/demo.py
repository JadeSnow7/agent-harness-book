from agent_harness import ChangeSet, EffectApplier, EffectIntent, EffectKind, authorize
c=ChangeSet.create('file','old','new','demo','low')
intent=EffectIntent('call','file','filesystem',EffectKind.EFFECT,'file','policy','run',{})
print(authorize(intent), EffectApplier().apply(c,'old'))

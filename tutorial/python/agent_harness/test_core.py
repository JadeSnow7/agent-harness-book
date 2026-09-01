import unittest
from . import *

class CoreTests(unittest.TestCase):
 def test_canonical_is_recursive_and_stable(self):
  self.assertEqual(canonical({'b':2,'a':{'d':4,'c':3}}),'{"a":{"c":3,"d":4},"b":2}')
 def test_provider_boundary_and_runner(self):
  p=FakeProvider([ModelResponse('done')]); r=Runner(p).run('hello'); self.assertEqual(r.reason,StopReason.COMPLETED); self.assertEqual(p.calls,1)
 def test_event_snapshot_roundtrip(self):
  i=Identity('s','t','r'); l=EventLog(); l.append(EventEnvelope(1,'start',i,{})); self.assertEqual(l.replay()[0].identity,i)
  s=Snapshot(1,'v1',{'x':1}); self.assertEqual(Snapshot.from_dict({'seq':1,'schema':'v1','state':{'x':1}}),s)
 def test_change_approval_and_validation(self):
  c=ChangeSet.create('x','a','b','reason','low'); ap=Approval('a','r','c',digest(c),'v','e','me'); self.assertTrue(bind_approval(ap,'r','c',digest(c),'v','e'))
  self.assertTrue(Validator({'ok':lambda x:True}).validate(c).passed)
 def test_delegation_and_extension(self):
  spec=TaskSpec('c','p','goal',frozenset({'read'}),2); result=TaskResult(spec.task_id,'child',True,('e',),'ok'); self.assertTrue(aggregate('p',[result]).completed)
  m=CapabilityManifest('x','1','test',frozenset({'read'}),{}); x=FakeSkill(m,Lifecycle.ACTIVE); self.assertTrue(x.invoke({})['ok'])
 def test_wire_round_trip(self):
  d=ToolDescriptor('read','browser',EffectKind.READ,{'type':'object'},{'type':'string'},'fake','1'); self.assertEqual(ToolDescriptor.from_wire(d.to_wire()),d)
 def test_schema_failures(self):
  d=ToolDescriptor('x','d',EffectKind.READ,{'type':'object','required':['q']},{},'f','1'); r=ToolRegistry([ToolSpec(d,lambda x:x)]).call('c','x',{}); self.assertEqual(r.error['code'],'schema_error')
 def test_event_rebuild(self):
  i=Identity('s','t','r'); l=EventLog(); l.append(EventEnvelope(1,'a',i,{'x':1})); self.assertEqual(l.rebuild_state(),{'x':1})
 def test_stop_reasons(self):
  o=ProgressOracle(1); o.observe('x'); self.assertEqual(StopPolicy().decide(o),StopReason.BUDGET_EXHAUSTED); self.assertEqual(StopPolicy().decide(o,cancelled=True),StopReason.CANCELLED)
 def test_finish_gate(self): self.assertEqual(finish(Validator({'x':lambda v:False}).validate('v'))['status'],'Failed')
 def test_runner_effect_reserves_before_handler_and_validates_finish(self):
  seen=[]; d=ToolDescriptor('mutate','demo',EffectKind.EFFECT,{'type':'object'},{'type':'object'})
  reg=ToolRegistry([ToolSpec(d,lambda args:(seen.append(args) or {'ok':True}))])
  ledger=IdempotencyLedger(); log=EventLog(Identity('s','t','r'))
  p=FakeProvider([ModelResponse(tool_calls=(ToolCall('c','mutate',{}),)),ModelResponse('done')])
  result=Runner(p,reg,event_log=log,allow_effects=True,ledger=ledger).run('go')
  self.assertEqual(result.reason,StopReason.COMPLETED); self.assertEqual(len(seen),1)
  self.assertEqual(ledger.lookup('r:c').outcome,Outcome.COMPLETED)
 def test_runner_validation_failure_cannot_complete(self):
  p=FakeProvider([ModelResponse('done')]); result=Runner(p,validator=Validator({'no':lambda _:False})).run('go')
  self.assertEqual(result.reason,StopReason.ESCALATE); self.assertNotEqual(result.terminal['status'],'completed')
 def test_event_view_is_immutable(self):
  i=Identity('s','t','r'); log=EventLog(); log.append(EventEnvelope(1,'x',i,{}))
  with self.assertRaises(AttributeError): log.events.append(EventEnvelope(2,'x',i,{}))
 def test_runner_rejects_duplicate_call_ids_before_execution(self):
  descriptor=ToolDescriptor('read','demo',EffectKind.READ,{'type':'object'},{'type':'object'})
  registry=ToolRegistry([ToolSpec(descriptor,lambda _: {'ok':True})])
  response=ModelResponse(tool_calls=(ToolCall('same','read',{}),ToolCall('same','read',{})))
  result=Runner(FakeProvider([response]),registry).run('go')
  self.assertEqual(result.reason,StopReason.ESCALATE); self.assertEqual(registry._calls,{})
if __name__ == '__main__': unittest.main()

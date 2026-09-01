import unittest
from agent_harness import *
class EffectsTests(unittest.TestCase):
 def test_hash_and_apply_once(self):
  c=ChangeSet.create('x','a','b','test','low'); a=EffectApplier(); first=a.apply(c,'a'); self.assertEqual(first['after'],'b'); self.assertIs(a.apply(c,'ignored'),first)
  with self.assertRaises(ContractError): a.apply(ChangeSet.create('x','z','q','x','high'),'a')
 def test_ask_by_default(self): self.assertEqual(authorize(EffectIntent('c','tool','domain',EffectKind.EFFECT,'scope','pd','r',{})).decision,Decision.ASK.value)
 def test_read_allow_and_unified_diff(self):
  self.assertEqual(authorize(EffectIntent('c','read','domain',EffectKind.READ,'scope','pd','r',{})).decision,Decision.ALLOW.value)
  self.assertIn('--- x (before)',ChangeSet.create('x','a\n','b\n','r','low').diff)
 def test_stale_hash(self): self.assertRaises(ContractError,EffectApplier().apply,ChangeSet.create('x','a','b','r','l'),'z')
 def test_policy_deny_invalid(self): self.assertEqual(authorize(object()).decision,Decision.DENY.value)
 def test_effect_intent_fields(self): self.assertEqual(EffectIntent('c','t','d','effect','s','p','r',{}).scope,'s')
 def test_intent_digest_is_stable_and_sensitive_to_arguments(self):
  first=EffectIntent('c','tool','domain',EffectKind.EFFECT,'scope','policy','run',{'b':2,'a':1})
  same=EffectIntent('c','tool','domain',EffectKind.EFFECT,'scope','policy','run',{'a':1,'b':2})
  changed=EffectIntent('c','tool','domain',EffectKind.EFFECT,'scope','policy','run',{'a':1,'b':3})
  self.assertEqual(intent_digest(first),intent_digest(same))
  self.assertNotEqual(intent_digest(first),intent_digest(changed))
 def test_input_schema_precedes_policy_ledger_and_handler(self):
  seen=[]; descriptor=ToolDescriptor('mutate','demo',EffectKind.EFFECT,{'type':'object','required':['x']},{'type':'object'})
  registry=ToolRegistry([ToolSpec(descriptor,lambda args:(seen.append(args) or {}))]); ledger=IdempotencyLedger()
  provider=FakeProvider([ModelResponse(tool_calls=(ToolCall('c','mutate',{}),)),ModelResponse('done')])
  result=Runner(provider,registry,allow_effects=True,ledger=ledger).run('go')
  self.assertEqual(result.reason,StopReason.COMPLETED); self.assertEqual(seen,[]); self.assertEqual(ledger.records,[])
 def test_output_schema_error_is_not_success(self):
  descriptor=ToolDescriptor('read','demo',EffectKind.READ,{'type':'object'},{'type':'string'})
  result=ToolRegistry([ToolSpec(descriptor,lambda _: {'not':'string'})]).call('c','read',{})
  self.assertFalse(result.ok); self.assertEqual(result.error['code'],'output_schema_error')
 def test_stale_approval_is_denied(self):
  approval=Approval('a','r','c','old','v','e','human')
  intent=EffectIntent('c','edit','code',EffectKind.EFFECT,'scope','policy','r',{},'new','v','e')
  self.assertEqual(authorize(intent,approval=approval).decision,Decision.DENY.value)
if __name__ == '__main__': unittest.main()

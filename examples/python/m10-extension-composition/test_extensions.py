import unittest
from agent_harness import *
class ExtensionTests(unittest.TestCase):
 def test_lifecycle_diff_and_gate(self):
  a=CapabilityManifest('x','1','a',frozenset({'read'}),{}); b=CapabilityManifest('x','2','b',frozenset({'read','write'}),{}); self.assertEqual(capability_diff(a,b)['added'],['write'])
  x=FakePlugin(b)
  with self.assertRaises(RuntimeError): x.invoke({'api_key':'secret'})
 def test_activate_revoke_timeout_schema(self):
  m=CapabilityManifest('x','1','test',frozenset({'read'}),{}); x=FakeHook(m); x.activate(); self.assertEqual(x.invoke({'api_key':'s'})['payload']['api_key'],'[REDACTED]'); x.revoke()
  with self.assertRaises(RuntimeError): x.invoke({})
  x.activate()
  with self.assertRaises(TimeoutError): x.invoke({},0)
  with self.assertRaises(ValueError): x.invoke([],1)
 def test_manifest_invalid(self): self.assertRaises(ContractError,CapabilityManifest,'','','',frozenset(),{})
 def test_diff_removed(self):
  a=CapabilityManifest('x','1','a',frozenset({'r'}),{}); b=CapabilityManifest('x','1','a',frozenset(),{}); self.assertEqual(capability_diff(a,b)['removed'],['r'])
 def test_registered_rejects(self): self.assertRaises(RuntimeError,FakeMCP(CapabilityManifest('x','1','a',frozenset(),{})).invoke,{})
 def test_plugin_success(self):
  x=FakePlugin(CapabilityManifest('x','1','a',frozenset(),{})); x.activate(); self.assertTrue(x.invoke({})['ok'])
 def test_gateway_requires_policy_validation_and_evidence(self):
  m=CapabilityManifest('x','1','a',frozenset(),{}); x=FakeSkill(m); x.activate()
  gateway=ExtensionGateway(x,Validator({'ok':lambda value:value['ok']}),EvidenceStore())
  self.assertRaises(PermissionError,gateway.invoke,{},'ask','r','e')
  value,evidence=gateway.invoke({},'allow','r','e')
  self.assertTrue(value['ok']); self.assertEqual(evidence.evidence_id,'e')
if __name__ == '__main__': unittest.main()

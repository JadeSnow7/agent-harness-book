import unittest
from agent_harness import *
class EvidenceTests(unittest.TestCase):
 def test_projection_redacts(self):
  i=Identity('s','t','r'); e=Evidence('e','r','x',{'api_key':'secret'},(1,)); p=project_evidence(e,[EventEnvelope(1,'x',i,{})]); self.assertEqual(p.value['api_key'],'[REDACTED]')
 def test_cross_run_and_deep_redaction(self):
  i=Identity('s','t','r'); e=Evidence('e','r','x',{'nested':[{'Authorization':'Bearer abc'}]},(1,)); p=project_evidence(e,[EventEnvelope(1,'x',i,{})]); self.assertEqual(p.value['nested'][0]['Authorization'],'[REDACTED]')
  with self.assertRaises(ValueError): project_evidence(Evidence('e','other','x',{},(1,)),[EventEnvelope(1,'x',i,{})])
 def test_missing_seq_rejected(self):
  i=Identity('s','t','r'); self.assertRaises(ValueError,project_evidence,Evidence('e','r','x',{},(2,)),[EventEnvelope(1,'x',i,{})])
 def test_trace_filters_run(self):
  i=Identity('s','t','r'); j=Identity('s','t','q'); self.assertEqual(len(trace_projection('r',[EventEnvelope(1,'x',i,{}),EventEnvelope(2,'x',j,{})]).events),1)
 def test_summary_read_projection(self): self.assertEqual(summary_projection('r',[Evidence('e','r','x','ok')]).evidence_ids,('e',))
 def test_redact_string(self): self.assertEqual(redact('Bearer token and api_key=secret'),'Bearer [REDACTED] and api_key=[REDACTED]')
 def test_review_bundle_changes_when_any_input_changes(self):
  from agent_harness import ChangeSet, ReviewBundle, Validator
  change=ChangeSet.create('x','a','b','r','low'); report=Validator({'ok':lambda _:True}).validate('x')
  first=ReviewBundle.bind(change,report,[Evidence('e','r','x',{'ok':True})])
  second=ReviewBundle.bind(ChangeSet.create('x','a','c','r','low'),report,[Evidence('e','r','x',{'ok':True})])
  self.assertNotEqual(first.bundle_digest,second.bundle_digest)
 def test_store_and_trace_are_redacted(self):
  i=Identity('s','t','r'); event=EventEnvelope(1,'x',i,{'Authorization':'Bearer abc','path':'/Users/secret/file'})
  store=EvidenceStore(); safe=store.add(Evidence('e','r','x',{'token':'secret'}))
  self.assertEqual(safe.value['token'],'[REDACTED]')
  self.assertEqual(trace_projection('r',[event]).events[0].payload['Authorization'],'[REDACTED]')
if __name__ == '__main__': unittest.main()

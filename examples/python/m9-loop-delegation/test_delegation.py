import unittest
from agent_harness import *
class DelegationTests(unittest.TestCase):
 def test_stall_and_evidence_gate(self):
  o=ProgressOracle(5); o.observe('x'); o.observe('x'); o.observe('x'); self.assertEqual(StopPolicy().decide(o),StopReason.STALLED)
  with self.assertRaises(ContractError): aggregate('p',[TaskResult('c','child',True)])
 def test_serial_executor_isolates_parent(self):
  spec=TaskSpec('c','p','g',frozenset({'read'}),1); ex=SerialDelegateExecutor(lambda s,r:TaskResult(s.task_id,r.run_id,True,('e',),'ok')); out=ex.run('p',[spec]); self.assertEqual(out[0].run_id,'p.child.1')
  with self.assertRaises(ContractError): ex.run('other',[spec])
 def test_failed_child_rejected(self): self.assertRaises(ContractError,aggregate,'p',[TaskResult('c','child',False,('e',))])
 def test_multiple_serial_order(self):
  specs=[TaskSpec(str(i),'p','g',frozenset(),1) for i in range(2)]; out=SerialDelegateExecutor(lambda s,r:TaskResult(s.task_id,r.run_id,True,('e',))).run('p',specs); self.assertEqual([r.run_id for r in out],['p.child.1','p.child.2'])
 def test_parent_child_fields(self): self.assertEqual(TaskSpec('c','p','g',frozenset({'read'}),2).parent_run_id,'p')
 def test_duplicate_parent_result_rejected(self): self.assertRaises(ContractError,aggregate,'p',[TaskResult('p','p',True,('e',))])
 def test_empty_aggregate_and_capability_boundary(self):
  self.assertRaises(ContractError,aggregate,'p',[])
  spec=TaskSpec('c','p','g',frozenset({'write'}),1)
  executor=SerialDelegateExecutor(lambda s,r:TaskResult(s.task_id,r.run_id,True,('e',)),frozenset({'read'}))
  self.assertRaises(ContractError,executor.run,'p',[spec])
if __name__ == '__main__': unittest.main()

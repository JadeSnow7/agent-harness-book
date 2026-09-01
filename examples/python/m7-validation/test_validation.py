import unittest
from agent_harness import Validator, finish
class ValidationTests(unittest.TestCase):
 def test_structured_report(self):
  r=Validator({'a':lambda x:True,'b':lambda x:False}).validate('x'); self.assertFalse(r.passed); self.assertEqual([x.name for x in r.failures()],['b'])
  self.assertEqual(r.failures()[0].code,'validation_failed')
 def test_exception_is_structured_and_finish_gates(self):
  from agent_harness import finish
  r=Validator({'boom':lambda x:1/0}).validate('x'); self.assertEqual(r.failures()[0].code,'validator_error'); self.assertEqual(finish(r)['status'],'Failed')
 def test_empty_checks_fail_closed(self): self.assertFalse(Validator({}).validate('x').passed)
 def test_named_order(self): self.assertEqual([c.name for c in Validator({'z':lambda x:1,'a':lambda x:1}).validate('x').checks],['a','z'])
 def test_finish_success(self): self.assertEqual(finish(Validator({'ok':lambda x:1}).validate('x'))['status'],'Completed')
 def test_failure_payload(self): self.assertIn('code',finish(Validator({'no':lambda x:0}).validate('x'))['failures'][0])
if __name__ == '__main__': unittest.main()

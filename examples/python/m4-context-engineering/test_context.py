import unittest
from agent_harness import ContextItem, Source, build_context, ContractError
class ContextTests(unittest.TestCase):
 def test_order_omit_and_required_failure(self):
  a=ContextItem("b","b",Source.USER,1,1); b=ContextItem("a","a",Source.SYSTEM,2,2,True)
  r=build_context([a,b],1); self.assertEqual(r.items,(b,)); self.assertEqual(r.omitted,("b",))
  with self.assertRaises(ContractError): build_context([b],0)
 def test_source_trace_and_summary(self):
  r=build_context([ContextItem('long','word '*100,Source.MEMORY,1,1)],200)
  self.assertIn('long',r.summarized); self.assertEqual(r.decisions[0].action,'summarized')
 def test_utf8_bytes(self):
  r=build_context([ContextItem('x','你好',Source.USER)],4); self.assertEqual(r.omitted,('x',)); self.assertEqual(r.decisions[0].action,'omitted')
 def test_required_precedes_priority(self): self.assertEqual(build_context([ContextItem('a','a',Source.USER,99),ContextItem('b','b',Source.SYSTEM,0,0,True)],1).items[0].key,'b')
 def test_negative_budget(self): self.assertRaises(ContractError,build_context,[], -1)
 def test_tie_break(self): self.assertEqual([x.key for x in build_context([ContextItem('b','b',Source.USER),ContextItem('a','a',Source.USER)],2).items],['a','b'])
if __name__ == '__main__': unittest.main()

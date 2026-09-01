import unittest
from agent_harness import *
class SessionTests(unittest.TestCase):
 def test_log_and_ambiguous(self):
  i=Identity('s','t','r'); l=EventLog(); l.append(EventEnvelope(1,'x',i,{}))
  with self.assertRaises(ContractError): l.append(EventEnvelope(3,'bad',i,{}))
  self.assertEqual(decide(Outcome.AMBIGUOUS).action,RecoveryAction.STOP)
 def test_idempotency_identity_and_digest_mismatch(self):
  l=IdempotencyLedger(); identity=Identity('s','t','r'); l.reserve('k',identity,'d')
  with self.assertRaises(IdentityMismatch): l.reserve('k',Identity('s','t','other'),'d')
  with self.assertRaises(IdentityMismatch): l.reserve('k',identity,'changed')
 def test_append_only_terminal_and_reuse(self):
  i=Identity('s','t','r'); l=EventLog(); l.append(EventEnvelope(1,'done',i,{},True))
  with self.assertRaises(ContractError): l.append(EventEnvelope(2,'late',i,{}))
  ledger=IdempotencyLedger(); first=ledger.reserve('k',i,'d'); ledger.mark('k',Outcome.COMPLETED,{'value':1}); self.assertEqual(ledger.reserve('k',i,'d').result,{'value':1}); self.assertGreater(len(ledger.records),1)
 def test_unknown_is_permanent(self):
  l=IdempotencyLedger(); i=Identity('s','t','r'); l.reserve('k',i,'d'); l.unknown('k'); self.assertEqual(l.unknown('k').outcome,Outcome.AMBIGUOUS)
 def test_recovery_actions(self): self.assertEqual(decide(Outcome.FAILED).action,RecoveryAction.RETRY); self.assertEqual(decide(Outcome.FAILED,FailPoint.AFTER).action,RecoveryAction.RESUME)
 def test_cross_run(self):
  l=EventLog(); i=Identity('s','t','r'); l.append(EventEnvelope(1,'x',i,{})); self.assertRaises(ContractError,l.append,EventEnvelope(2,'x',Identity('s','t','q'),{}))
 def test_concurrent_reservation_has_one_executor(self):
  from concurrent.futures import ThreadPoolExecutor
  ledger=IdempotencyLedger(); identity=Identity('s','t','r')
  with ThreadPoolExecutor(8) as pool:
   claims=list(pool.map(lambda _: ledger.reserve('k',identity,'d'), range(8)))
  self.assertEqual(sum(claim.outcome is Outcome.RESERVED for claim in claims),1)
  self.assertEqual(sum(claim.outcome is Outcome.IN_PROGRESS for claim in claims),7)
if __name__ == '__main__': unittest.main()

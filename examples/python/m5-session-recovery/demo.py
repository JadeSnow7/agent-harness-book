from agent_harness import Identity, EventEnvelope, EventLog, Outcome, decide
i=Identity("s","t","r"); log=EventLog(); log.append(EventEnvelope(1,"started",i,{})); print(log.seq, decide(Outcome.AMBIGUOUS))

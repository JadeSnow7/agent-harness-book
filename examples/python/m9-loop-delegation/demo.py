from agent_harness import ProgressOracle, StopPolicy, TaskResult, aggregate
o=ProgressOracle(3); o.observe('new'); print(StopPolicy().decide(o)); print(aggregate('p',[TaskResult('c','p.child.1',True,('e',),'done')]).text)

from agent_harness import Evidence, Identity, EventEnvelope, project_evidence
i=Identity('s','t','r'); e=Evidence('e','r','api',{'Authorization':'Bearer secret'},(1,)); print(project_evidence(e,[EventEnvelope(1,'x',i,{})]).value)

Strengthened the proofs for same-term vote preservation and stale recovery completions.

Rechecked all eight formal entries: **PASS**. Tests remain **15/16 passing on every seed**, with no compilation failures or tool timeouts.

Acceptance is still **INCOMPLETE** because the protected restart test mistakes closed-channel completion for an old reply. Added a MadSim reproduction; preserved the test.

[RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) records evidence and unverified host premises. All five global safety targets remain OPEN.
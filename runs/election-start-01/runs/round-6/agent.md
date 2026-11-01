Closed this slice as **blocked** and saved [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

Latest independent logs match the current candidate: build and all eight proofs pass; every seed passes 15/16 tests. The sole failure is the protected closed-channel assertion, not a timeout. Its upstream correction is documented; candidate changes cannot repair it.

Failures and cumulative spending remain retained. Serialization, truthful I/O/readback, retention, forwarding and cancellation remain unverified host premises. Local acceptance is **INCOMPLETE**; global safety and liveness remain **OPEN**.
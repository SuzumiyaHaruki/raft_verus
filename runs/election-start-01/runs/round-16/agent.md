Check-15 confirms the same blocker; evidence is saved in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

Build and all eight proof entries pass on the unchanged candidate. Each seed passes 15/16 tests. The sole failure is the protected closed-channel assertion at `rpc_tests.rs:213`; no compilation, proof failure or tool timeout occurred.

Acceptance remains blocked pending corrected protected input. Failures and cumulative spending are preserved. Serialization, truthful I/O/readback, retention and forwarding remain host premises; all five global safety goals remain OPEN.
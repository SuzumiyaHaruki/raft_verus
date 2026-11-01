Rechecked: all eight proof entries pass; 15/16 tests pass on every seed. No compilation failures or tool timeouts.

**Blocked by the protected test:** [rpc_tests.rs:213](/home/nitro/Desktop/raft_verus/runs/election-start-01/tasks/election_start/rpc_tests.rs:213) expects a timeout after canceling the channel’s sole sender. The closed channel returns immediately. No permitted Core change can repair that assertion.

Saved the evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md). Acceptance remains incomplete pending a protected-test correction. Host serialization, truthful I/O/readback and forwarding remain unverified; global safety goals remain OPEN.
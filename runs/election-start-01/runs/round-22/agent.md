Saved check-21 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass. Each seed passes 15/16 tests; the sole functional failure remains `rpc_tests.rs:213`, expecting a timeout after the observation channel closes. No compilation/proof failures or tool timeouts occurred. Candidate sources match the checked snapshot.

Acceptance remains **BLOCKED_FIXED_TEST**. Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
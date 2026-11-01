Saved check-20 evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass; all five seeds pass 15/16 tests. The sole functional failure remains the protected closed-channel timeout assertion at `rpc_tests.rs:213`. No compilation/proof failures or tool timeouts occurred. Current candidate sources match the checked snapshot.

Acceptance remains **BLOCKED_FIXED_TEST**; repeated checks cannot resolve this contradiction. Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
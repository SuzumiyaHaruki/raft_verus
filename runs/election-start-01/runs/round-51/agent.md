Saved check-50 findings in [PROGRESS.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/PROGRESS.md) and [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

The current candidate matches the checked source. Build and all eight proof entries pass; each seed passes 15/16 tests. The sole failure remains the protected closed-channel assertion at `rpc_tests.rs:213`. All processes completed—no tool timeouts.

Acceptance remains **INCOMPLETE / BLOCKED_FIXED_TEST**; correcting the protected test is necessary. No permitted candidate repair was identified. Failures and cumulative spending remain preserved.

Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
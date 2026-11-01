Saved check-51 findings in [PROGRESS.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/PROGRESS.md) and [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

Build and all eight proof entries pass, covering receiver rules, campaigns, Choice history, authorization and four recovery entries. The checked source matches the candidate. Each seed passes 15/16 tests; the sole failure remains the protected closed-channel assertion at `rpc_tests.rs:213`. No compilation/proof failures or tool timeouts occurred.

Acceptance remains **INCOMPLETE / BLOCKED_FIXED_TEST**. No permitted candidate repair was identified. Failures and cumulative spending are preserved.

Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
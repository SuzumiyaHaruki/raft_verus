Saved check-52 findings in [PROGRESS.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/PROGRESS.md) and [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

The current candidate matches the checked source. Build and all eight proof entries pass, including receiver rules, campaigns, unified Choice history, authorization and four recovery entries. Each seed passes 15/16 tests.

Acceptance remains **INCOMPLETE / BLOCKED_FIXED_TEST**: `rpc_tests.rs:213` expects a timeout after aborting the sole sender, but the closed channel returns immediately. This is an assertion failure; no compilation/proof failures or tool timeouts occurred. No permitted candidate repair was identified.

Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
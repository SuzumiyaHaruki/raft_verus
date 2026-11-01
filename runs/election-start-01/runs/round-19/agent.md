Saved check-18 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass, including unified history, authorization and four recovery entries. All five seeds pass 15/16 tests. The sole failure remains the protected assertion expecting a timeout after the channel closes; no compilation/proof failures or tool timeouts occurred.

Acceptance remains blocked by that test contradiction. Candidate sources match the checked snapshot. Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention, cancellation and unchanged forwarding remain host premises; all five global safety properties remain OPEN.
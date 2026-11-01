Saved check-23 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass on the unchanged candidate. All five seeds pass 15/16 tests. The sole failure remains the protected assertion expecting a timeout after its sender is destroyed; this is a test contradiction, not a tool timeout. No permitted candidate repair was identified.

Acceptance remains blocked. Failures and cumulative spending are preserved. Serialization, truthful I/O/readback, retention and unchanged forwarding remain host premises; all five global safety goals remain OPEN.
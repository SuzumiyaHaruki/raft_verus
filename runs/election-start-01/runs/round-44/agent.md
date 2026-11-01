Saved check-43 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass; all five seeds remain 15/16. The sole failure is the protected assertion at `rpc_tests.rs:213`: aborting and joining the sole sender closes the channel, so receive returns immediately rather than timing out. No compilation, proof or tool-timeout failure occurred.

The current candidate matches the checked sources; the mutation audit found no additional same-term vote-changing path. Acceptance remains blocked pending correction of the protected test. Failures and cumulative spending are preserved.

Host serialization, truthful I/O/readback, file retention and unchanged forwarding remain unverified premises. All five global safety properties remain OPEN.
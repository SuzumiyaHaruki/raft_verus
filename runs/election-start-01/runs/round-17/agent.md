Saved check-16 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass. Each seed passes 15/16 tests. The remaining functional assertion expects a timeout after dropping the channel’s sole sender; reception instead returns a closed-channel error. This is a protected-test contradiction, not a compilation/proof failure or tool timeout. Acceptance remains incomplete.

Failures and cumulative spending remain retained. Serialization, truthful I/O/readback, file retention, cancellation and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
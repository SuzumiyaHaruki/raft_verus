Saved check-25 evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md. Current sources match the checked candidate.

Build and all eight proof entries pass. Each seed passes 15/16 tests. Acceptance remains blocked by `rpc_tests.rs:213`, which expects a timeout after its sole sender is destroyed; the closed-channel error returns immediately. No compilation, proof or tool-timeout failure occurred.

Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention and unchanged forwarding remain host premises. All five global safety goals remain OPEN.
Saved check-27 evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass on the unchanged candidate. Each seed passes 15/16 tests. The sole functional failure remains `rpc_tests.rs:213`: it expects a timeout after destroying the channel’s sole sender, but receives an immediate closed-channel error. Acceptance remains blocked; no compilation, proof or tool-timeout failure occurred.

Failures and cumulative spending are preserved. Serialization, truthful I/O/readback, retention and unchanged forwarding remain host premises. All five global safety goals remain OPEN.
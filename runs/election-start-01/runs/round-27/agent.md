Saved check-26 evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass on the matching candidate. Each seed passes 15/16 tests. The sole functional failure remains the protected assertion at `rpc_tests.rs:213`: a closed channel returns immediately, contradicting its expected timeout. No compilation, proof or tool-timeout failure occurred; acceptance remains blocked.

Failures and cumulative spending are preserved. Serialization, truthful I/O/readback, retention and unchanged forwarding remain host premises. All five global safety goals remain OPEN.
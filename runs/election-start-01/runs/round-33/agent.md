Saved check-32’s findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass against the current candidate. Seeds 1–5 each pass 15/16 tests. The sole failure remains `rpc_tests.rs:213`: it expects a timeout after closing the channel. This is a functional assertion failure, not a compilation/proof failure or tool timeout.

Acceptance remains blocked pending corrected protected input. Failures and cumulative spending are preserved. Serialization, truthful I/O, faithful readback/retention and unchanged forwarding remain host premises; all five global safety goals remain OPEN.
Saved check-28 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass; all five seeds pass 15/16 tests. Acceptance remains **blocked by the protected test** at `rpc_tests.rs:213`: destroying its sole sender produces an immediate closed-channel error, contradicting the expected timeout. No compilation, proof or tool-timeout failure occurred. Corrected protected input is required to proceed.

Failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retention and unchanged forwarding remain host premises; all five global safety goals remain OPEN.
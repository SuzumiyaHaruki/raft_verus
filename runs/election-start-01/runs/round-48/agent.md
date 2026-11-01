Saved check-47 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass. Each seed remains 15/16, failing only the protected closed-channel assertion at `rpc_tests.rs:213`. No compilation, proof or tool-timeout failure occurred. The current candidate matches the checked sources.

This slice is closed with acceptance **INCOMPLETE**: correcting the protected test remains necessary. Failures and cumulative spending are preserved. Host serialization, truthful I/O/readback, retention and unchanged forwarding remain premises; all five global safety properties remain OPEN.
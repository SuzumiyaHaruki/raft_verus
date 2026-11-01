Saved check-14 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md). Candidate/source identity matches; build and all eight proofs pass.

Acceptance remains **BLOCKED_FIXED_TEST**: all five seeds fail only `rpc_tests.rs:213`, which expects a closed channel to time out. This is a functional assertion failure, not a compilation, proof or tool-timeout failure. No permitted candidate repair resolves it.

Failures and cumulative spending are retained. Host serialization, truthful I/O/readback, retention and forwarding remain premises; global safety remains OPEN.
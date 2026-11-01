Saved check-31 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proofs pass, including four recovery entries. Each seed passes 15/16 tests. The sole failure remains the protected test’s closed-channel assertion, not compilation, proof failure or tool timeout. No permitted candidate repair resolves that contradiction.

Acceptance remains incomplete. Failures and cumulative spending are preserved. Serialization, truthful I/O, faithful readback/retention and unchanged forwarding remain host premises; all five global safety goals remain OPEN.
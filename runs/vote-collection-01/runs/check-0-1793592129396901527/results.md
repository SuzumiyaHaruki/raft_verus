# vote-collection scoped checks

INCOMPLETE

Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | OPEN: test_or_build_failure |
| source_boundary | PASS: same_candidate_imports |
| collection_rules | OPEN: verification_failure |
| collection_campaign | OPEN: verification_failure |
| collection_authorization | OPEN: verification_failure |
| collection_history | OPEN: verification_failure |
| collection_reply_rules | OPEN: verification_failure |
| collection_self_vote | OPEN: verification_failure |
| collection_rounds | OPEN: verification_failure |
| collection_higher_reply | OPEN: verification_failure |
| collection_provenance | OPEN: verification_failure |
| collection_nonempty | OPEN: verification_failure |
| collection_receive_confirmed | OPEN: verification_failure |
| collection_receive_pending | OPEN: verification_failure |
| collection_self_confirmed | OPEN: verification_failure |
| collection_self_pending | OPEN: verification_failure |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

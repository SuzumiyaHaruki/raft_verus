# vote-collection scoped checks

INCOMPLETE

Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | OPEN: test_timeout |
| inventory | OPEN: build_dependency_failed |
| tests_seed_1_raft_raft_vote_tests | OPEN: build_dependency_failed |
| source_boundary | PASS: same_candidate_imports |
| collection_rules | PASS: checked |
| collection_campaign | PASS: checked |
| collection_authorization | PASS: checked |
| collection_history | PASS: checked |
| collection_reply_rules | PASS: checked |
| collection_self_vote | PASS: checked |
| collection_rounds | PASS: checked |
| collection_higher_reply | PASS: checked |
| collection_provenance | OPEN: verification_failure |
| collection_nonempty | OPEN: verification_failure |
| collection_receive_confirmed | OPEN: verification_timeout |
| collection_receive_pending | OPEN: check_budget_exhausted |
| collection_self_confirmed | OPEN: check_budget_exhausted |
| collection_self_pending | OPEN: check_budget_exhausted |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

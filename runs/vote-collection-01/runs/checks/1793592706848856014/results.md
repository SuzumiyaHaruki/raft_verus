# vote-collection scoped checks

INCOMPLETE

Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | OPEN: compilation_error |
| inventory | OPEN: build_dependency_failed |
| tests_seed_1_raft_raft_vote_tests | OPEN: build_dependency_failed |
| source_boundary | PASS: same_candidate_imports |
| collection_rules | OPEN: compilation_error |
| collection_campaign | OPEN: compilation_error |
| collection_authorization | OPEN: compilation_error |
| collection_history | OPEN: compilation_error |
| collection_reply_rules | OPEN: compilation_error |
| collection_self_vote | OPEN: compilation_error |
| collection_rounds | OPEN: compilation_error |
| collection_higher_reply | OPEN: compilation_error |
| collection_provenance | OPEN: compilation_error |
| collection_nonempty | OPEN: compilation_error |
| collection_receive_confirmed | OPEN: compilation_error |
| collection_receive_pending | OPEN: compilation_error |
| collection_self_confirmed | OPEN: compilation_error |
| collection_self_pending | OPEN: compilation_error |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

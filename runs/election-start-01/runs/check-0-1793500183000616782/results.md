# election-start scoped checks

INCOMPLETE

Local choices include actual self-selections and affirmative replies; a self-choice is not I/O or majority evidence. Serialization, faithful I/O/readback, file retention and unchanged RPC forwarding remain unverified host premises. Only two bounded recovery windows are covered. All five global safety properties, full 2A and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | OPEN: test_or_build_failure |
| source_boundary | PASS: same_candidate_imports |
| election_rules | OPEN: verification_failure |
| election_campaign | OPEN: verification_failure |
| election_authorization | OPEN: verification_failure |
| election_history | OPEN: verification_failure |
| election_receive_confirmed | OPEN: verification_failure |
| election_receive_pending | OPEN: verification_failure |
| election_self_confirmed | OPEN: verification_failure |
| election_self_pending | OPEN: verification_failure |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

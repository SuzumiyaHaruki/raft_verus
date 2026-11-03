# election-recovery scoped checks

INCOMPLETE

Finite fixed-membership histories permit repeated node restarts only after a complete successful write, before or after its matching Core completion. Other nodes and old network messages continue. Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | OPEN: test_timeout |
| inventory | OPEN: build_dependency_failed |
| tests_seed_1_raft_raft_vote_tests | OPEN: build_dependency_failed |
| tests_seed_2_raft_raft_vote_tests | OPEN: build_dependency_failed |
| tests_seed_3_raft_raft_vote_tests | OPEN: build_dependency_failed |
| tests_seed_4_raft_raft_vote_tests | OPEN: build_dependency_failed |
| tests_seed_5_raft_raft_vote_tests | OPEN: build_dependency_failed |
| source_boundary | PASS: same_candidate_imports |
| recovery_rules | OPEN: verification_failure |
| recovery_campaign | OPEN: verification_failure |
| recovery_authorization | OPEN: verification_failure |
| recovery_history | OPEN: verification_failure |
| recovery_reply_rules | OPEN: verification_failure |
| recovery_self_vote | OPEN: verification_failure |
| recovery_rounds | OPEN: verification_failure |
| recovery_higher_reply | OPEN: verification_failure |
| recovery_provenance | OPEN: verification_timeout |
| recovery_nonempty | OPEN: check_budget_exhausted |
| recovery_promotion | OPEN: check_budget_exhausted |
| recovery_election_basis | OPEN: check_budget_exhausted |
| recovery_election_history | OPEN: check_budget_exhausted |
| recovery_election_nonempty | OPEN: check_budget_exhausted |
| recovery_receive_confirmed | OPEN: check_budget_exhausted |
| recovery_receive_pending | OPEN: check_budget_exhausted |
| recovery_self_confirmed | OPEN: check_budget_exhausted |
| recovery_self_pending | OPEN: check_budget_exhausted |
| recovery_restart_source | OPEN: check_budget_exhausted |
| recovery_restart_history | OPEN: check_budget_exhausted |
| recovery_restart_nonempty | OPEN: check_budget_exhausted |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

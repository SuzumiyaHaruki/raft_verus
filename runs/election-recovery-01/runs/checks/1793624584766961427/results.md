# election-recovery scoped checks

INCOMPLETE

Finite fixed-membership histories permit repeated node restarts only after a complete successful write, before or after its matching Core completion. Other nodes and old network messages continue. Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | PASS: checked |
| tests_seed_2_raft_raft_vote_tests | PASS: checked |
| tests_seed_3_raft_raft_vote_tests | PASS: checked |
| tests_seed_4_raft_raft_vote_tests | PASS: checked |
| tests_seed_5_raft_raft_vote_tests | PASS: checked |
| source_boundary | PASS: same_candidate_imports |
| recovery_rules | PASS: checked |
| recovery_campaign | PASS: checked |
| recovery_authorization | PASS: checked |
| recovery_history | PASS: checked |
| recovery_reply_rules | PASS: checked |
| recovery_self_vote | PASS: checked |
| recovery_rounds | PASS: checked |
| recovery_higher_reply | PASS: checked |
| recovery_provenance | OPEN: verification_failure |
| recovery_nonempty | OPEN: verification_failure |
| recovery_promotion | PASS: checked |
| recovery_election_basis | OPEN: verification_failure |
| recovery_election_history | OPEN: verification_failure |
| recovery_election_nonempty | OPEN: verification_failure |
| recovery_receive_confirmed | OPEN: verification_timeout |
| recovery_receive_pending | OPEN: check_budget_exhausted |
| recovery_self_confirmed | OPEN: check_budget_exhausted |
| recovery_self_pending | OPEN: check_budget_exhausted |
| recovery_restart_source | OPEN: check_budget_exhausted |
| recovery_restart_history | OPEN: check_budget_exhausted |
| recovery_restart_nonempty | OPEN: check_budget_exhausted |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

# election-recovery scoped checks

INCOMPLETE

Finite fixed-membership histories permit repeated node restarts only after a complete successful write, before or after its matching Core completion. Other nodes and old network messages continue. Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | PASS: checked |
| source_boundary | PASS: same_candidate_imports |
| recovery_rules | OPEN: verification_failure |
| recovery_campaign | OPEN: verification_failure |
| recovery_authorization | OPEN: verification_failure |
| recovery_history | OPEN: verification_failure |
| recovery_reply_rules | OPEN: verification_failure |
| recovery_self_vote | OPEN: verification_failure |
| recovery_rounds | OPEN: verification_failure |
| recovery_higher_reply | OPEN: verification_failure |
| recovery_provenance | OPEN: verification_failure |
| recovery_nonempty | OPEN: verification_failure |
| recovery_promotion | OPEN: verification_failure |
| recovery_election_basis | OPEN: verification_failure |
| recovery_election_history | OPEN: verification_failure |
| recovery_election_nonempty | OPEN: verification_failure |
| recovery_receive_confirmed | OPEN: verification_failure |
| recovery_receive_pending | OPEN: verification_failure |
| recovery_self_confirmed | OPEN: verification_failure |
| recovery_self_pending | OPEN: verification_failure |
| recovery_restart_source | OPEN: verification_failure |
| recovery_restart_history | OPEN: verification_failure |
| recovery_restart_nonempty | OPEN: verification_failure |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

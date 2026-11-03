# election-recovery scoped checks

PASS

Finite fixed-membership histories permit repeated node restarts only after a complete successful write, before or after its matching Core completion. Other nodes and old network messages continue. Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | PASS: checked |
| source_boundary | PASS: same_candidate_imports |
| recovery_rules | PASS: checked |
| recovery_campaign | PASS: checked |
| recovery_authorization | PASS: checked |
| recovery_history | PASS: checked |
| recovery_reply_rules | PASS: checked |
| recovery_self_vote | PASS: checked |
| recovery_rounds | PASS: checked |
| recovery_higher_reply | PASS: checked |
| recovery_provenance | PASS: checked |
| recovery_nonempty | PASS: checked |
| recovery_promotion | PASS: checked |
| recovery_election_basis | PASS: checked |
| recovery_election_history | PASS: checked |
| recovery_election_nonempty | PASS: checked |
| recovery_receive_confirmed | PASS: checked |
| recovery_receive_pending | PASS: checked |
| recovery_self_confirmed | PASS: checked |
| recovery_self_pending | PASS: checked |
| recovery_restart_source | PASS: checked |
| recovery_restart_history | PASS: checked |
| recovery_restart_nonempty | PASS: checked |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

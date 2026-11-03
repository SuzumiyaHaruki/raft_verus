# Experiment status

ELECTION_SAFETY_COMPLETED_WRITE_RESTART_CHECKS_PASSED: Scoped checks passed; semantic review remains outstanding

Input: madraft-election-recovery-1; origin: b31bad0a-91b2-4c57-9456-270260557f29; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 5; cumulative seconds: 5768.5/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/election-recovery-01/check-5-1793628386210091000/results.json. Evidence is invalidated by changes to checked inputs, tool versions or parameters.

| Check or obligation | Independent result |
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
| election_safety | OPEN: not checked |
| leader_append_only | OPEN: not checked |
| log_matching | OPEN: not checked |
| leader_completeness | OPEN: not checked |
| state_machine_safety | OPEN: not checked |
| execution_link | OPEN: not checked |
| snapshot_observation | OPEN: not checked |
| nonempty_execution | OPEN: not checked |

Finite fixed-membership histories permit repeated node restarts only after a complete successful write, before or after its matching Core completion. Other nodes and old network messages continue. Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

task_scope=election-recovery; full MadRaft INCOMPLETE; five global safety properties OPEN. Local host boundary: tasks/election_recovery/TASK.md.

Guidance: B; unused budget: 1431.5s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

| Goal | Formula | Actual connection | Implementation task |
| --- | --- | --- | --- |
| election_safety | OPEN | OPEN | OPEN |
| leader_append_only | OPEN | OPEN | OPEN |
| log_matching | OPEN | OPEN | OPEN |
| leader_completeness | OPEN | OPEN | OPEN |
| state_machine_safety | OPEN | OPEN | OPEN |
| execution_link | OPEN | OPEN | OPEN |
| snapshot_observation | OPEN | OPEN | OPEN |
| nonempty_execution | OPEN | OPEN | OPEN |
| safety | OPEN | OPEN | OPEN |

Reported token counters retain UNKNOWN scope and are not summed across resumed calls. Dollar cost is unknown. One fresh generation experiment contains multiple continuation rounds and multiple test seeds; these are not independent generation successes.

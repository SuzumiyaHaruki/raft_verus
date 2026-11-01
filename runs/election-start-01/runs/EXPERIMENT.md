# Experiment status

INCOMPLETE: Maximum generation rounds reached

Input: madraft-election-start-1; origin: 47a949c4-57c6-4702-9ef9-73c02b1eb939; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6-astra; effort: high; rounds: 64; cumulative seconds: 6336.7/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/election-start-01/check-64-1793506499931769054/results.json. Historical results are not reused after changes.

| Check or obligation | Independent result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | OPEN: test_or_build_failure |
| tests_seed_2_raft_raft_vote_tests | OPEN: test_or_build_failure |
| tests_seed_3_raft_raft_vote_tests | OPEN: test_or_build_failure |
| tests_seed_4_raft_raft_vote_tests | OPEN: test_or_build_failure |
| tests_seed_5_raft_raft_vote_tests | OPEN: test_or_build_failure |
| source_boundary | PASS: same_candidate_imports |
| election_rules | PASS: checked |
| election_campaign | PASS: checked |
| election_authorization | PASS: checked |
| election_history | PASS: checked |
| election_receive_confirmed | PASS: checked |
| election_receive_pending | PASS: checked |
| election_self_confirmed | PASS: checked |
| election_self_pending | PASS: checked |
| election_safety | OPEN: not checked |
| leader_append_only | OPEN: not checked |
| log_matching | OPEN: not checked |
| leader_completeness | OPEN: not checked |
| state_machine_safety | OPEN: not checked |
| execution_link | OPEN: not checked |
| snapshot_observation | OPEN: not checked |
| nonempty_execution | OPEN: not checked |

Local choices include actual self-selections and affirmative replies; a self-choice is not I/O or majority evidence. Serialization, faithful I/O/readback, file retention and unchanged RPC forwarding remain unverified host premises. Only two bounded recovery windows are covered. All five global safety properties, full 2A and conditional liveness remain OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

task_scope=election-start; full MadRaft INCOMPLETE; five global safety properties OPEN. Local host boundary: tasks/election_start/TASK.md.

Guidance: B; unused budget: 863.3s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

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

# Experiment status

ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED: Scoped checks passed; semantic review remains outstanding

Input: madraft-election-safety-1; origin: 5c98388f-dc06-462c-ac2d-b445ee397dff; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 4; cumulative seconds: 4038.5/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/election-safety-01/check-4-1793603876604771367/results.json. Evidence is invalidated by changes to checked inputs, tool versions or parameters.

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
| election_rules | PASS: checked |
| election_campaign | PASS: checked |
| election_authorization | PASS: checked |
| election_history | PASS: checked |
| election_reply_rules | PASS: checked |
| election_self_vote | PASS: checked |
| election_rounds | PASS: checked |
| election_higher_reply | PASS: checked |
| election_provenance | PASS: checked |
| election_nonempty | PASS: checked |
| election_promotion | PASS: checked |
| election_election_basis | PASS: checked |
| election_election_history | PASS: checked |
| election_election_nonempty | PASS: checked |
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

Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, finite executions from Followers, without node restart. Four local recovery windows are separate. RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

task_scope=election-safety; full MadRaft INCOMPLETE; five global safety properties OPEN. Local host boundary: tasks/election_safety/TASK.md.

Guidance: B; unused budget: 3161.5s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

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

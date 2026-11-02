# Experiment status

VOTE_COLLECTION_CHECKS_PASSED: Scoped checks passed; semantic review remains outstanding

Input: madraft-vote-collection-1; origin: 702e400d-2199-44ae-83d1-a3a7255766d6; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 2; cumulative seconds: 1667.9/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/vote-collection-01/check-2-1793593711227080022/results.json. Evidence is invalidated by changes to checked inputs, tool versions or parameters.

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
| collection_rules | PASS: checked |
| collection_campaign | PASS: checked |
| collection_authorization | PASS: checked |
| collection_history | PASS: checked |
| collection_reply_rules | PASS: checked |
| collection_self_vote | PASS: checked |
| collection_rounds | PASS: checked |
| collection_higher_reply | PASS: checked |
| collection_provenance | PASS: checked |
| collection_nonempty | PASS: checked |
| collection_receive_confirmed | PASS: checked |
| collection_receive_pending | PASS: checked |
| collection_self_confirmed | PASS: checked |
| collection_self_pending | PASS: checked |
| election_safety | OPEN: not checked |
| leader_append_only | OPEN: not checked |
| log_matching | OPEN: not checked |
| leader_completeness | OPEN: not checked |
| state_machine_safety | OPEN: not checked |
| execution_link | OPEN: not checked |
| snapshot_observation | OPEN: not checked |
| nonempty_execution | OPEN: not checked |

Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

task_scope=vote-collection; full MadRaft INCOMPLETE; five global safety properties OPEN. Local host boundary: tasks/vote_collection/TASK.md.

Guidance: B; unused budget: 5532.1s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

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

# Experiment status

INCOMPLETE: Insufficient budget for useful work and final checks

Input: madraft-joint-proof-4; origin: 58954752-c6bd-4062-88a5-b32602141a44; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 6; cumulative seconds: 3000.1/3600.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/calls-A-01/check-6-1793334379571675997/results.json. Historical results are not reused after changes.

| Check or obligation | Independent result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_tests | PASS: checked |
| tests_seed_1_kvraft_tests | PASS: checked |
| tests_seed_1_shard_ctrler_tests | PASS: checked |
| tests_seed_1_shardkv_tests | PASS: checked |
| tests_seed_1_test_support | PASS: checked |
| tests_seed_2_raft_tests | PASS: checked |
| tests_seed_2_kvraft_tests | PASS: checked |
| tests_seed_2_shard_ctrler_tests | PASS: checked |
| tests_seed_2_shardkv_tests | PASS: checked |
| tests_seed_2_test_support | PASS: checked |
| tests_seed_3_raft_tests | PASS: checked |
| tests_seed_3_kvraft_tests | PASS: checked |
| tests_seed_3_shard_ctrler_tests | PASS: checked |
| tests_seed_3_shardkv_tests | PASS: checked |
| tests_seed_3_test_support | PASS: checked |
| tests_seed_4_raft_tests | PASS: checked |
| tests_seed_4_kvraft_tests | PASS: checked |
| tests_seed_4_shard_ctrler_tests | PASS: checked |
| tests_seed_4_shardkv_tests | PASS: checked |
| tests_seed_4_test_support | PASS: checked |
| tests_seed_5_raft_tests | PASS: checked |
| tests_seed_5_kvraft_tests | PASS: checked |
| tests_seed_5_shard_ctrler_tests | PASS: checked |
| tests_seed_5_shardkv_tests | PASS: checked |
| tests_seed_5_test_support | PASS: checked |
| foundation | PASS: checked |
| definitions | PASS: checked |
| execution_link | OPEN: missing_entry_or_dependency |
| election_safety | OPEN: missing_entry_or_dependency |
| leader_append_only | OPEN: missing_entry_or_dependency |
| log_matching | OPEN: missing_entry_or_dependency |
| leader_completeness | OPEN: missing_entry_or_dependency |
| state_machine_safety | OPEN: missing_entry_or_dependency |
| snapshot_observation | OPEN: missing_entry_or_dependency |
| nonempty_execution | OPEN: missing_entry_or_dependency |
| linkage_review | OPEN: missing_implementation_entry |
| connection_initialization | OPEN: connection_source_not_imported |
| connection_transitions | OPEN: connection_source_not_imported |
| connection_outputs | OPEN: connection_source_not_imported |
| connection_recovery | OPEN: connection_source_not_imported |
| connection_snapshots | OPEN: connection_source_not_imported |
| safety | OPEN: missing_entry_or_dependency |
| proof | PASS: checked |

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

Guidance: A; unused budget: 599.9s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

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

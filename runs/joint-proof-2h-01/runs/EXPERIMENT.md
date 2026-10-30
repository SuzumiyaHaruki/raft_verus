# Experiment status

INCOMPLETE: Insufficient budget for useful work and final checks

Input: madraft-joint-proof-5; origin: b86194d0-7e39-415b-94d0-40a123eb4b82; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6-astra; effort: high; rounds: 12; cumulative seconds: 6668.7/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/joint-proof-2h-01/check-12-1793351241131800847/results.json. Historical results are not reused after changes.

| Check or obligation | Independent result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_tests | OPEN: test_or_build_failure |
| tests_seed_1_kvraft_tests | OPEN: test_or_build_failure |
| tests_seed_1_shard_ctrler_tests | OPEN: test_or_build_failure |
| tests_seed_1_shardkv_tests | OPEN: test_or_build_failure |
| tests_seed_1_test_support | PASS: checked |
| tests_seed_2_raft_tests | OPEN: test_or_build_failure |
| tests_seed_2_kvraft_tests | OPEN: test_or_build_failure |
| tests_seed_2_shard_ctrler_tests | OPEN: test_or_build_failure |
| tests_seed_2_shardkv_tests | OPEN: test_or_build_failure |
| tests_seed_2_test_support | PASS: checked |
| tests_seed_3_raft_tests | OPEN: test_or_build_failure |
| tests_seed_3_kvraft_tests | OPEN: test_or_build_failure |
| tests_seed_3_shard_ctrler_tests | OPEN: test_or_build_failure |
| tests_seed_3_shardkv_tests | OPEN: test_or_build_failure |
| tests_seed_3_test_support | PASS: checked |
| tests_seed_4_raft_tests | OPEN: test_or_build_failure |
| tests_seed_4_kvraft_tests | OPEN: test_or_build_failure |
| tests_seed_4_shard_ctrler_tests | OPEN: test_or_build_failure |
| tests_seed_4_shardkv_tests | OPEN: test_or_build_failure |
| tests_seed_4_test_support | PASS: checked |
| tests_seed_5_raft_tests | OPEN: test_or_build_failure |
| tests_seed_5_kvraft_tests | OPEN: test_or_build_failure |
| tests_seed_5_shard_ctrler_tests | OPEN: test_or_build_failure |
| tests_seed_5_shardkv_tests | OPEN: test_or_build_failure |
| tests_seed_5_test_support | PASS: checked |
| current_dependency_1 | PASS: checked |
| current_dependency_2 | PASS: checked |
| current_entry | PASS: checked |
| current_parent | PASS: checked |
| foundation | PASS: checked |
| definitions | PASS: checked |
| execution_link | PASS: checked |
| election_safety | OPEN: compilation_error |
| leader_append_only | OPEN: compilation_error |
| log_matching | OPEN: compilation_error |
| leader_completeness | OPEN: compilation_error |
| state_machine_safety | OPEN: compilation_error |
| snapshot_observation | OPEN: compilation_error |
| nonempty_execution | OPEN: compilation_error |
| linkage_review | PASS: limited_source_checks_only |
| connection_initialization_unit | PASS: checked |
| connection_initialization | OPEN: outer_paths_open |
| connection_transitions_unit | PASS: checked |
| connection_transitions | OPEN: outer_paths_open |
| connection_outputs_unit | PASS: checked |
| connection_outputs | OPEN: outer_paths_open |
| connection_recovery_unit | PASS: checked |
| connection_recovery | OPEN: outer_paths_open |
| connection_snapshots_unit | OPEN: missing_entry_or_dependency |
| connection_snapshots | OPEN: missing_entry_or_dependency |
| safety | OPEN: compilation_error |
| proof | PASS: checked |

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

Guidance: B; unused budget: 531.3s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

| Goal | Formula | Actual connection | Implementation task |
| --- | --- | --- | --- |
| election_safety | OPEN | OPEN | OPEN |
| leader_append_only | OPEN | OPEN | OPEN |
| log_matching | OPEN | OPEN | OPEN |
| leader_completeness | OPEN | OPEN | OPEN |
| state_machine_safety | OPEN | OPEN | OPEN |
| execution_link | PASS | OPEN | OPEN |
| snapshot_observation | OPEN | OPEN | OPEN |
| nonempty_execution | OPEN | OPEN | OPEN |
| safety | OPEN | OPEN | OPEN |

Reported token counters retain UNKNOWN scope and are not summed across resumed calls. Dollar cost is unknown. One fresh generation experiment contains multiple continuation rounds and multiple test seeds; these are not independent generation successes.

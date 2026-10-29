# Experiment status

INCOMPLETE: Insufficient budget for useful work, feedback and final checks, or maximum rounds reached

Input: madraft-joint-proof-3; origin: f15aca92-c9c9-4fee-b16a-8eb5a0dbb631; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 5; cumulative seconds: 2661.2/3600.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/fresh-system-02/check-5-1793271672184810316/results.json. Historical results are not reused after changes.

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
| foundation | PASS: checked |
| definitions | PASS: checked |
| execution_link | PASS: checked |
| election_safety | PASS: checked |
| leader_append_only | PASS: checked |
| log_matching | OPEN: compilation_error |
| leader_completeness | OPEN: compilation_error |
| state_machine_safety | OPEN: compilation_error |
| snapshot_observation | PASS: checked |
| nonempty_execution | PASS: checked |
| linkage_review | PASS: limited_source_checks_only |
| safety | OPEN: compilation_error |
| proof | PASS: checked |

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

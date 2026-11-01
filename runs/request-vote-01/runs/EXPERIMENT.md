# Experiment status

REQUEST_VOTE_CHECKS_PASSED: Scoped checks passed; semantic review remains outstanding

Input: madraft-request-vote-1; origin: 577bf176-7355-42c8-a4fc-306b0d9444f0; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6-astra; effort: high; rounds: 1; cumulative seconds: 345.4/7200.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/request-vote-01/check-1-1793495456807427792/results.json. Historical results are not reused after changes.

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
| vote_rules | PASS: checked |
| vote_history | PASS: checked |
| vote_confirmed | PASS: checked |
| vote_pending | PASS: checked |
| election_safety | OPEN: not checked |
| leader_append_only | OPEN: not checked |
| log_matching | OPEN: not checked |
| leader_completeness | OPEN: not checked |
| state_machine_safety | OPEN: not checked |
| execution_link | OPEN: not checked |
| snapshot_observation | OPEN: not checked |
| nonempty_execution | OPEN: not checked |

局部接收端检查不等于全局 Election Safety。宿主串行化、实际 I/O、编码、文件保留、通知真实性及向原 RPC 请求者转发仍未验证；只覆盖两种指定恢复窗口。

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

task_scope=request-vote-receiver; full MadRaft INCOMPLETE; five global safety properties OPEN. Local host boundary: tasks/request_vote/TASK.md.

Guidance: B; unused budget: 6854.6s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.

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

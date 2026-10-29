# Experiment status

INCOMPLETE: Cumulative time or maximum rounds exhausted

Input: madraft-joint-proof-2; origin: b72eb121-522f-4eba-9b8f-96540a4e3c97; source revision: UNAVAILABLE: Git metadata not exposed.

Model: gpt-6.1-sol; effort: high; rounds: 6; cumulative seconds: 3600.0/3600.0.

Latest independent evidence: /home/nitro/Desktop/raft_verus/runs/.controller/fresh-system-01/check-6-1793255729075190038/results.json. Historical results are not reused after changes.

| Check or obligation | Independent result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1 | OPEN: test_timeout |
| tests_seed_2 | OPEN: check_budget_exhausted |
| tests_seed_3 | OPEN: check_budget_exhausted |
| tests_seed_4 | OPEN: check_budget_exhausted |
| tests_seed_5 | OPEN: check_budget_exhausted |
| proof | OPEN: check_budget_exhausted |
| election_safety | OPEN: check_budget_exhausted |
| leader_append_only | OPEN: check_budget_exhausted |
| log_matching | OPEN: check_budget_exhausted |
| leader_completeness | OPEN: check_budget_exhausted |
| state_machine_safety | OPEN: check_budget_exhausted |
| execution_link | OPEN: check_budget_exhausted |
| snapshot_observation | OPEN: check_budget_exhausted |
| nonempty_execution | OPEN: check_budget_exhausted |
| linkage_review | PASS: limited_source_checks_only |
| safety | OPEN: check_budget_exhausted |

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.

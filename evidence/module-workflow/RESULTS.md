# Module-workflow revision: actual checks

Status: DRAFT. Commands ran serially from the repository root with
`PYTHONDONTWRITEBYTECODE=1`. Full stdout/stderr are in [checks.log](checks.log).
No live Codex generation was invoked. The repair demo uses scripted candidate
edits and real Rust/Verus checks. A BLOCKED project result is an unmet requirement,
not a successful whole-project verification. Measurement has no approved threshold.

| Check | Command | Exit | Seconds | Observation |
|---|---|---|---|---|
| Controller suite | `python3 -m unittest discover -s orchestrator/tests -v` | 0 | 24.019 | 84 tests passed, no skips |
| Structural validation | `python3 -m orchestrator validate` | 0 | 0.590 | PASS |
| Readable module plan | `python3 -m orchestrator plan --format text` | 0 | 0.394 | exit 0 |
| Planned initialization prompt | `python3 -m orchestrator prompt --task STATE-INIT` | 0 | 0.265 | exit 0 |
| Ready replication prompt | `python3 -m orchestrator prompt --task REPLICATION-RECONCILE` | 0 | 0.386 | exit 0 |
| Ready-step plan preview | `python3 -m orchestrator run-plan --tasks REPLICATION-RECONCILE REPLICATION-PROOF REPLICATION-REPLAY INTEGRATION-MEASURE --dry-run` | 0 | 0.383 | DRY_RUN |
| Missing initialization prerequisites | `python3 -m orchestrator run-task --task STATE-INIT --dry-run` | 1 | 0.447 | BLOCKED |
| Rust build | `python3 -m orchestrator verify --profile interface-check` | 0 | 0.581 | PASS; run 20261027T062022Z-84ab509472c7 |
| Rust functional tests | `python3 -m orchestrator verify --profile reconcile-tests` | 0 | 1.087 | PASS; run 20261027T062022Z-d03408285b15 |
| Model typecheck | `python3 -m orchestrator verify --profile spec` | 0 | 2.014 | TYPECHECKED {'verified': 0, 'errors': 0}; run 20261027T062023Z-49c1dd8e66a6 |
| Local Verus proof | `python3 -m orchestrator verify --profile reconcile-proof` | 0 | 2.530 | PASS {'verified': 9, 'errors': 0}; run 20261027T062025Z-888aebeac756 |
| Local replay | `python3 -m orchestrator verify --profile reconcile-replay` | 0 | 1.066 | PASS; run 20261027T062028Z-7320d3a8a7ef |
| Whole-project acceptance | `python3 -m orchestrator verify --profile full` | 1 | 0.131 | BLOCKED; run 20261027T062029Z-25d099c12ddb |
| Controlled defect and repair | `python3 orchestrator/tests/real_repair_demo.py --output evidence/runs/module-workflow-repair-demo` | 0 | 13.487 | PASS |
| Core measurement | `python3 -m orchestrator verify --profile reconcile-bench` | 0 | 1.479 | MEASURED; run 20261027T062042Z-3a6c0e5b9d75 |

[Final audit](audit.log) checked all 25 prompts, file links, unchanged protocol and
formal source against the pre-edit archive, the four empty read-only directories,
26 OPEN global obligations, disabled real generation and current real-check inputs.
The temporary audit script ran with the command recorded in its log.

The four runnable steps still cover only existing reconciliation work. New protocol
implementation starts with STATE-INIT/RUNTIME-PERSIST after their real API/scope
and check profiles are supplied. All modules remain incomplete and DRAFT.

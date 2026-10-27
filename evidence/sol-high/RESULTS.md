# Coupling and explicit model experiment checks

Status: DRAFT. No live Codex generation, account-access test or actual Sol/High
response was run. Fixtures check argument forwarding. Local Rust/Verus checks
execute installed tools. Commands ran serially with PYTHONDONTWRITEBYTECODE=1.
Full command output: [checks.log](checks.log).

| Check | Command | Exit | Seconds | Observation |
|---|---|---|---|---|
| Controller suite | `python3 -m unittest discover -s orchestrator/tests -v` | 0 | 16.493 | 88 tests passed, no skips |
| Validation | `python3 -m orchestrator validate` | 0 | 0.370 | PASS |
| Explicit-model plan | `python3 -m orchestrator --model gpt-6.1-sol --reasoning-effort high plan --format text` | 0 | 0.313 | exit 0 |
| Explicit-model dry-run | `python3 -m orchestrator --model gpt-6.1-sol --reasoning-effort high run-plan --tasks REPLICATION-RECONCILE REPLICATION-PROOF REPLICATION-REPLAY INTEGRATION-MEASURE --dry-run` | 0 | 0.384 | DRY_RUN |
| Local Rust tests | `python3 -m orchestrator verify --profile reconcile-tests` | 0 | 1.172 | PASS; 20261027T063208Z-7c3c4c6af425 |
| Local Verus proof | `python3 -m orchestrator verify --profile reconcile-proof` | 0 | 2.073 | PASS; 20261027T063209Z-4a09c03172ad |

Final consistency check: real generation remains disabled; default model settings were not changed; saved Rust/Verus checks still match current inputs; edited Markdown links exist; model, assumptions, properties, proof checklist, I/O and TCB documents are unchanged.

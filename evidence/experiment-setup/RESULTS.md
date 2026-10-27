# Sol/High experiment setup and review placement

Status: DRAFT. Actual local checks on 2026-10-27. No live model call.

The project defaults are `gpt-6.1-sol` / `high`. `./experiment.sh` previews the
four ready steps; `./experiment.sh run` starts them explicitly, without an
approval file. Human review happens after code delivery. No external approval
was installed or marked approved. The superseded temporary DRAFT was deleted.

| Actual command/check | Result |
|---|---|
| `bash -n experiment.sh` | PASS |
| `python3 -m orchestrator validate` | PASS, structural checks only |
| `python3 -m orchestrator plan --format text` | PASS; configured model/effort, four ready and 21 planned steps |
| `/home/nitro/Desktop/raft_verus/experiment.sh` from `/tmp` | DRY_RUN, zero agent calls; exact four selected steps and Sol/High defaults |
| `python3 -m unittest discover -s orchestrator/tests -v` | 91 tests passed, no skips; includes normal CLI task/plan/resume without approval, actual fixture subprocess model forwarding, disabled start/resume, protected paths and real-Verus integrations |
| `python3 -m orchestrator doctor` | Codex CLI READY; isolated Rust/Cargo 1.92, Verus 0.2026.01.23.1650a05 and Z3 4.12.5 AVAILABLE; overall PARTIAL because compatible temporal tooling is missing |
| `python3 crates/raft-verified/check.py test` | 15 real Rust tests passed: 2 interface, 11 reconciliation, 2 replay cases |
| `python3 -m orchestrator verify --profile reconcile-proof` | Real Verus: 9 verified / 0 errors; LOCAL-RECONCILE only |
| `python3 orchestrator/tests/real_repair_demo.py --output evidence/runs/experiment-setup-repair-demo` | PASS; two SIMULATED edits, real Rust regressions and Verus reject the defect then accept the repair; Verus 8/1 then 9/0 |
| `python3 -m orchestrator verify --profile full` | BLOCKED, exit 1: missing full proof entrypoint; no full-proof invocation or acceptance |

Current raw output is in [final-tests.log](final-tests.log) and
[final-checks.log](final-checks.log). The new repair evidence is
[the demo summary](../runs/experiment-setup-repair-demo/summary.json).
The original tool/15-test observations are retained in [checks.log](checks.log);
the Rust/Verus code and tool configuration were not subsequently changed.

The first approval-removal test pass exposed a missing local `settings` variable
after factoring the existing execution controls (18 errors, recorded in
[revision-tests.log](revision-tests.log)). The variable was restored and the
complete suite passed. Earlier launcher/approval checks in `checks.log` precede
the user's clarification and are superseded by the current implementation.

The final audit in [final-checks.log](final-checks.log) covers the simplified
launcher's dispatch, unchanged Raft source/formal inputs, English prompts and
local Markdown links. Historical evidence and read-only directories are preserved.
No global Codex config or system toolchain was replaced. Backup before this setup:
`/tmp/raft-experiment-config-original-6n833o01/authored.tar.gz`.

Live account access and a real Sol/High experiment remain NOT_RUN. Replay and
measurement code did not change; no new performance claim was made. All 26 global
Raft obligations, full-node integration, liveness/refinement and compatible temporal
tooling remain open. STATE-INIT is the next new implementation assignment and still
needs concrete source scope and real checks. Specifications and technical decisions
remain DRAFT; execution is not semantic approval or permission to merge/publish.

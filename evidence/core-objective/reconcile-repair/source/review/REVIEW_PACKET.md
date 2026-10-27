# Raft workflow review packet

Status: DRAFT. Updated 2026-10-27. Human review follows code delivery, for discussion
and revision with the user. No Raft semantic decision or correctness claim is approved.

IDS reference update: `input/instructions.md` and `input/roadmap.md` now require
joint incremental code/proof work and evidence-guided redesign; source S14 in
`docs/sources.md` records the paper and adaptation limits. This changes task
guidance only. The four-step launcher below remains a helper experiment; it still
does not build a Raft node. No model, Rust source or controller behavior changed.
Validation, nine module tests and all 25 English prompts passed; actual output is
appended to [the check log](../evidence/experiment-setup/final-checks.log). Rust/Verus were not rerun for this documentation-only update.

## Current changes

| File | Change |
|---|---|
| `workflow.toml` | Default `gpt-6.1-sol` / `high`; explicit execution enabled; bounded calls/timeouts |
| `experiment.sh` | Preview/run/resume/report using the existing Python controller and four fixed steps |
| `orchestrator/scheduler.py`, `cli.py` | Removed runtime approval validation and `--approval-file`; retained candidate separation, scope checks, independent verification and execution controls |
| `orchestrator/wire/` | Deleted unused approval schema/example; retained agent/check report schemas |
| `orchestrator/tests/` | Start/resume without approval; disabled execution still blocks both; fixtures remain labeled SIMULATED |
| `AGENTS.md`, `README.md`, `docs/workflow.md`, `input/instructions.md` | Commands and review after delivery; English LLM material |
| `input/trust.md`, `review/OPEN_DECISIONS.md` | Removed approval storage from the checker description and revised ADR-10; no protocol assumption changed |

Existing mutation/evidence bindings remain: plan output now uses `input_bindings`,
task artifacts use `input_bindings.json`, and plan status is `DRAFT`. No new hashes,
signatures, frozen contracts or review framework were added. Historical evidence
remains readable. No external approval was installed or marked approved.

The user reviews delivered candidates afterward. Execution does not merge them
into the source checkout. Scope or semantic conflicts stop with findings; they do
not open an approval/signature interaction inside the program.

## Commands and scope

```sh
cd /home/nitro/Desktop/raft_verus
./experiment.sh                 # Preview; zero agent calls
./experiment.sh run             # Start the configured four-step experiment
./experiment.sh resume RUN_ID   # Continue its saved candidate
./experiment.sh report RUN_ID   # Read saved results
```

The selected steps are REPLICATION-RECONCILE, REPLICATION-PROOF,
REPLICATION-REPLAY and INTEGRATION-MEASURE. The first three allow three calls each,
600 seconds/call and 2,400 seconds/task; measurement allows two calls,
300 seconds/call and 1,200 seconds/task. These are execution limits, not a cost cap.
Model overrides, direct Python commands and check meanings are in
[the workflow guide](../docs/workflow.md#configured-solhigh-experiment).

## Actual checks

| Check | Result |
|---|---|
| Controller suite | 91 tests passed, no skips; actual offline subprocesses, normal CLI task/plan/resume without approval, and installed-Verus integrations |
| Validation, plan and launcher | PASS; preview from outside the repository uses Sol/High and launches zero agents |
| Rust tests | 15 passed: 2 interface, 11 reconciliation, 2 replay cases |
| Local Verus | 9 verified / 0 errors; LOCAL-RECONCILE only |
| Controlled defect/repair | Two scripted edits; real Rust tests and Verus reject truncation, then pass after repair; Verus 8/1 then 9/0 |
| Tools | Codex CLI help/version, isolated Rust/Cargo 1.92, Verus 0.2026.01.23.1650a05 and Z3 4.12.5 available; compatible temporal tooling missing |
| Full verification | BLOCKED before verifier launch: complete proof entrypoint is missing |
| Audit | English prompts, local links and launcher dispatch checked; 23 Rust/formal files unchanged; read-only directories preserved |

[Current results and raw logs](../evidence/experiment-setup/RESULTS.md) include the
initial missing-variable regression and its repair. Scripted generation is
SIMULATED; actual Rust/Verus checking is REAL_LOCAL. No live Codex call, recursive
agent invocation, merge, push or publication occurred. Live model/account access
remains NOT_RUN. The system Rust toolchain and global Codex settings are unchanged.

## Proof gaps and next work

The executable `reconcile_log`, tests, replay and benchmark use the same source.
Its local proof establishes correspondence to `merged` and prefix preservation;
it does not establish whole AppendEntries or cluster log matching. Future callers
must validate terms/predecessors, committed-prefix compatibility, index conversion
and durable response timing. Vec allocation can still abort on OOM.

All eight modules are INCOMPLETE and all 26 global obligations remain OPEN. Node
state/dispatch, elections, leader/follower replication, commitment/delivery,
recovery and real adapters are unfinished. Shared invariants, node/cluster
refinement and conditional liveness must connect the coupled runtime modules.
Local tests/proofs cannot substitute for those arguments.

**Next new implementation task: STATE-INIT**, followed by RUNTIME-PERSIST. Supply
concrete source scope and actual initialization/coordinator checks before activating
them. REPLICATION-RECONCILE is the next already-runnable local step. Passing the
four-step experiment completes neither replication nor Raft.

[Pending technical decisions](OPEN_DECISIONS.md) and [proof obligations](../input/proofs.md)
remain DRAFT/OPEN. Compatible temporal tooling and hostile-process isolation are
missing. No performance threshold or complete safety/liveness claim is approved.
The unchanged core benchmark has measurement data only; no new performance run
or throughput claim was made in this setup.

Earlier delivery evidence remains in [module-workflow](../evidence/module-workflow/RESULTS.md),
[model-control checks](../evidence/sol-high/RESULTS.md) and
[the original slice revision](../evidence/revision/commands.json). Its old approval
requirements and controller counts are historical. Stop here for review of the
actual files and results.

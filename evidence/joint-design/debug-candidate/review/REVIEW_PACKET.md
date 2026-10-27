# From-zero experiment correction

Status: DRAFT. Delivered 2026-10-27 for review after delivery. The experimental input now contains no working Raft operation or supplied induction strategy. No real Codex task, recursive agent, merge, push, publication, tool download or system-toolchain change occurred.

## Corrected experiment boundary

Maintenance prepares requirements, shared types and interfaces, formal targets, independent oracles, replay/measurement harnesses and the existing Python controller. The experiment generates executable behavior and proof strategies in one persistent candidate. Later steps extend that run's own code. This is generation from specified interfaces, not unconstrained generation from blank files. Supplied specifications, types, views and tests must be disclosed in any experiment report.

Previously, candidate copying included working reconciliation, persistence and progress implementations, and initialization required the existing operation checks to pass. That was an extension experiment. The active source now has failing executable seeds and empty progress proof bodies. Previous working implementations are preserved only as excluded SIMULATED fixtures. The original reject-all constructor is unchanged; it was not replaced with the scripted answer.

| Main files | Actual change |
|---|---|
| [Agent.md](../Agent.md), [input/starting-point.md](../input/starting-point.md), [input/instructions.md](../input/instructions.md) | Maintenance/experiment responsibilities, supplied input and generation boundaries, one continuously developed candidate |
| [orchestrator/workspace.py](../orchestrator/workspace.py) | Excludes all of `orchestrator/tests`, including scripts and solution data, from candidate copies and input manifests; preserves existing evidence/maintenance exclusions |
| [src/reconcile.rs](../crates/raft-verified/src/reconcile.rs), [src/persistence.rs](../crates/raft-verified/src/persistence.rs), [progress_lemmas.rs](../crates/raft-proofs/progress_lemmas.rs) | Incorrect error-only executable seeds and empty proof bodies; no assume/admit, trusted body or skipped verification |
| [src/types.rs](../crates/raft-verified/src/types.rs), local proof entrypoints and representation imports | Shared data vocabulary separated from reconciliation behavior; initialization/persistence proofs no longer import a future unfinished algorithm |
| [workflow.toml](../workflow.toml) | Initialization checks all eight unchanged initialization cases and its exact proof/replay; reconciliation checks its 13 operation/replay cases. Previously completed tasks still regress through the existing controller |
| [offline solutions](../orchestrator/tests/fixtures/solutions/README.md), [real_repair_demo.py](../orchestrator/tests/real_repair_demo.py) | Prior executable/proof examples retained as explicit fake-process edits; optional progress continuation demonstration. No solution loader is invoked by real launch/resume |
| [test_starting_point.py](../orchestrator/tests/test_starting_point.py), related controller tests | Exclusion, independent fresh copies, English prompt boundary, actual partial generation and missing-progress-proof regressions |

The unused legacy OperationSequence helper and its two tests are preserved in `orchestrator/tests/fixtures/solutions/legacy_lib.rs`; no Node implementation called it. It is removed from the supplied experimental API, rather than provided as another solved operation. All existing independent Rust integration tests and replay/measurement inputs are unchanged. The candidate model and initialization/persistence/progress specifications are unchanged. `reconcile_view.rs` only changes its type import. Protected proof consumers retain their contracts and checks, importing the relocated types instead of an unrelated algorithm. See the [preservation and exclusion audit](../evidence/from-zero/starting-point-audit.json).

## Actual checks and evidence

All generation below is **SIMULATED**, performed only by the bundled fake subprocess. Rust/Verus/replay checks actually ran and are **REAL_LOCAL**. There is no evidence of real-model capability. No fresh candidate contains the solution directory; scripts explicitly inject answers after candidate creation only for these offline demonstrations.

| Check | Observed result |
|---|---|
| [Controller suite](../evidence/from-zero/controller-final.log) | **115 tests passed, no skips**, including actual relocated-tool checking, cumulative continuation/regression checks and five new starting-point tests |
| [Active scaffold](../evidence/from-zero/seed-checks/summary.json) | Build PASS; initialization and reconciliation tests FAIL as expected. Verus: initialization 18 verified / 1 error, reconciliation 7 / 1, persistence 24 / 3, progress 19 / 5. Failures are real missing postconditions/proofs, not compiler failures |
| Model in that scaffold | TYPECHECKED, 0 verified; no Raft theorem discharged |
| [Initialization defect and repair](../evidence/from-zero/initialization-demo/summary.json) | Wrong initial term rejected by actual tests, replay and Verus; repaired scripted constructor passes all eight cases, replay and proof (21 / 0). Future persistence/reconciliation remain unfinished |
| [Shared candidate continuation](../evidence/from-zero/continuation-demo/summary.json) | Four scripted calls: initialization, wrong-kind persistence defect, repair, progress proof. Wrong-kind defect rejected by tests/replay and Verus (26 / 1). Repair passes six persistence cases, 14-event scenario and proof (27 / 0); supplied-by-fixture progress strategy then passes (24 / 0). Completed initialization/persistence checks rerun |
| [Reconciliation defect and repair](../evidence/from-zero/reconciliation-demo/summary.json) | Matching-tail truncation rejected by actual tests and Verus (8 / 1); repaired scripted implementation passes 13 cases and proof (9 / 0) |
| Missing proof strategy regression | With a scripted correct coordinator but empty progress bodies, actual progress verification fails; adding the scripted strategy makes it pass. Included in the 115-test suite |
| Measurements | Initialization/coordinator demonstrations emit actual repeated release measurements after required correctness checks. No comparable reference or performance target is configured; verdict remains MEASURED/INCONCLUSIVE |
| [Validation](../evidence/from-zero/validation.json), [default preview](../evidence/from-zero/default-preview.json), [combined preview](../evidence/from-zero/combined-preview.json) | PASS / DRY_RUN, zero live agent calls; 26 English task prompts; all 26 global obligations OPEN |

Verifier function counts include dependencies and derives; they are not counts of proved Raft properties. Each selected proof crate is checked in full with the existing no-cheating configuration. Separating types does not claim that excluded, future operation bodies are proved. A successful initialization or persistence candidate can still fail the all-library test command because reconciliation has not been generated. Whole-project acceptance remains false.

The earlier persistence-only demonstration and initial controller log are retained under `evidence/from-zero`; the final continuation demonstration above is the reviewed multi-step result. The earlier persistence-only run overlapped controller validation, so its timings should not be used as a comparable performance reference.

## Preservation and limits

The pre-edit authored-file backup is `/tmp/raft-from-zero-92mb2iaz/authored.tar.gz`. [The previous review packet](../evidence/from-zero/previous-review.md) and all earlier evidence remain available. The read-only `.git`, `.agents`, `.codex` and `.aws` directories were untouched. [The maintained-file diff](../evidence/from-zero/changes.patch) records this correction; no local Git reset or source solution overlay occurred.

Fresh runs copy the current scaffold; resume retains the recorded candidate, budgets and history. No automatic integration occurs. If someone later manually fills the source checkout with implementations, a new copy becomes an extension experiment and must be reported as such. Historical runs retain their actual starting materials and are not retroactively labeled from-zero. Existing changed-input checks prevent silent reuse across changed specifications/configuration.

Candidate exclusions prevent accidental answer inclusion. They do not provide OS isolation against an agent deliberately reading other same-user paths. Requirements instruct the experiment agent to stay inside the candidate and not retrieve external fixtures or prior implementations. Authentication, checker protection, scope restrictions, process limits and irreversible-operation safeguards remain in place.

All Raft modules remain incomplete. Election, both-side replication, commitment, recovery, integrated cluster behavior and global safety/liveness/refinement are still open. Supplied DRAFT semantic choices remain pending in [OPEN_DECISIONS.md](OPEN_DECISIONS.md), including persistence failure behavior, host durability, legal callers, recovery incarnations, full Effects integration and fairness/resource assumptions. This correction approves no new Raft behavior or assumptions. Empty auxiliary lemma declarations may be discharged automatically by SMT; report this as automation on supplied targets, not an AI-authored proof strategy.

## Exact commands and next task

Run these from the repository root; demonstration output directories must be new:

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m unittest discover -s orchestrator/tests -v
python3 orchestrator/tests/real_repair_demo.py --task STATE-INIT --output /tmp/raft-zero-init
python3 orchestrator/tests/real_repair_demo.py --task PROGRESS-FOUNDATION --output /tmp/raft-zero-continuation
python3 orchestrator/tests/real_repair_demo.py --task REPLICATION-RECONCILE --output /tmp/raft-zero-reconciliation
./experiment.sh
python3 -m orchestrator run-plan --tasks PROGRESS-FOUNDATION --dry-run
```

The next real task is STATE-INIT: generate the missing constructor and proof from the unchanged seed. `./experiment.sh run` explicitly launches that task with the configured `gpt-6.1-sol` / `high`; it does not build all of Raft. For the selected ready continuation, the following live commands remain available but were **not run** during this maintenance:

```sh
python3 -m orchestrator run-plan --tasks PROGRESS-FOUNDATION --execute-codex
python3 -m orchestrator resume --run-id RUN_ID --execute-codex
```

This plan generates initialization, persistence and local progress in the same candidate. Later protocol steps still need their runnable independent checks. Delivery stops for human review; no approval file or interactive approval checkpoint has been added.

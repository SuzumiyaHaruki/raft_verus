# Joint implementation and specification generation

Status: DRAFT. Delivered 2026-10-27 for review after delivery. This revision lets the experiment generate Rust behavior, implementation-layer Verus specifications, representation/invariants and proofs together, while independent callers still require the intended observable behavior. No real Codex task, recursive agent, merge, push, publication, download or system-toolchain replacement occurred.

## Changes and cleanup

| Files | Delivered behavior |
|---|---|
| [starting-point.md](../input/starting-point.md), module assignments, [Agent.md](../Agent.md), [workflow.toml](../workflow.toml) | Distinguish supplied observable requirements from generated implementation design. STATE-INIT and RUNTIME-PERSIST may change Node representation, local predicates and proofs; previously completed scopes/checks remain part of coupled repair. |
| [node_state.rs](../crates/raft-verified/src/node_state.rs), [contracts.rs](../crates/raft-verified/src/contracts.rs), operation files | Editable layout, `view`, `wf`, observation adapters and deliberately vacuous local-contract seeds. The constructor still rejects all inputs; persistence/reconciliation remain unfinished. No maintenance solution is installed in the active source. |
| [state.rs](../crates/raft-verified/src/state.rs), [observation.rs](../crates/raft-proofs/observation.rs), local proof entrypoints | Protected `Observation` vocabulary replaces a fixed internal Node layout. Independent executable callers connect observation adapters to the generated view, require initialization to establish `wf`, and require handlers to preserve it and the original behavioral relations. |
| Rust tests and examples | Observe actual state through the checked interface; preserve all eight initialization and six persistence cases, return-value expectations and replay inputs. Boundary exhaustion uses the disclosed representation adapter. Internal auxiliary fields need not be byte-for-byte unchanged when observations are unchanged. |
| [scheduler.py](../orchestrator/scheduler.py), report schema, verifier/feedback | Ordinary local-specification edits use changed_files/summary; protected requirement/assumption changes remain review proposals. Candidate type/mode/visibility errors route to source repair with proof NOT_RUN. Genuine proof failures and missing tools remain distinct. |
| [test_joint_design.py](../orchestrator/tests/test_joint_design.py), [real_repair_demo.py](../orchestrator/tests/real_repair_demo.py) | Actual checking of an alternate layout with a nontrivial serial invariant; vacuous-contract and observation/invariant negatives; scripted continuous generation, rejection and repair. |

Removed the unused 107-line `legacy_lib.rs` token fixture, the prescribed `waiting_prefix` proof skeleton, and the obsolete `review/REVISION_PLAN.md`. Replaced stale task restrictions and duplicate historical traceability/review tables instead of appending another policy layer. Global obligations remain maintained in [input/proofs.md](../input/proofs.md). No new configuration framework, hash mechanism, frozen contract, approval file or runtime review checkpoint was added.

The [audit](../evidence/joint-design/audit.json) confirms the model, requirements, assumptions, TCB, global obligations and saved replay inputs are unchanged. The three affected behavioral specification files differ only by replacing the concrete Node type with Observation; their conditions are unchanged. Tests changed their observation interface, so they are not claimed byte-identical. All 26 prompts are English; seven steps are ready and 19 remain planned.

## Actual results

Generation in the demonstration is **SIMULATED**, using only the bundled fake subprocess. Rust/Verus/replay checks are **REAL_LOCAL**. These results test the checking workflow, not real-model capability.

| Command/check | Observed result and evidence |
|---|---|
| `python3 -m unittest discover -s orchestrator/tests -v` | **119 passed, no skips**, including four new joint-design tests and existing budget/history/regression/toolchain checks. [Log](../evidence/joint-design/controller.log). |
| Final module/workspace regression after context/document updates | **18 passed**. [Log](../evidence/joint-design/prompt-regression.log). |
| Alternate representation and local-specification tests | Real initialization/persistence tests, proofs and replay pass with nested state and `issued == observation.serial`. False invariants and weakened observation contracts fail actual verification; source visibility errors are classified as repairable, proof NOT_RUN. [Focused log](../evidence/joint-design/design-tests.log). |
| Active scaffold | Build PASS; node tests FAIL as intended. Verus initialization **23 verified / 1 error**, persistence **29 / 3**, progress **26 / 2**, reconciliation **7 / 1**. Model TYPECHECKED, **0 verified**. [Reports and raw logs](../evidence/joint-design/seed-checks/summary.json). |
| Continuous alternate-layout demonstration | **PASS**, five scripted calls in one candidate: wrong initialization/local predicate, repaired initialization, wrong-kind persistence, repair, then local progress. [Summary](../evidence/joint-design/redesign-demo/summary.json). |
| Independent initialization rejection | Wrong term with a `true` generated contract lets the constructor's own contract verify, but the protected constructor target fails: **26 / 1**; runtime tests/replay also reject it. Repair passes eight cases, replay and proof **27 / 0**. |
| Persistence rejection and repair | Wrong-kind defect fails tests, replay and proof **32 / 1**. Repair passes six cases, the 14-event trace and proof **33 / 0**. Prior initialization checks rerun. |
| Local progress | Generated fixture proof passes **30 / 0**; completed initialization/persistence checks rerun. This proves only the existing conditional coordinator theorem, not Raft liveness. |
| Measurements | Actual repeated release measurements run after correctness checks, serially after the full controller suite. MEASURED/INCONCLUSIVE: no comparable reference or performance target configured. Raw measurement paths are in the demonstration summary. |
| Validation and previews | [Validation](../evidence/joint-design/validation.json) PASS; [default](../evidence/joint-design/default-preview.json) and [continuation](../evidence/joint-design/combined-preview.json) previews DRY_RUN, zero agent calls. |

Counts include dependencies and derives, not distinct Raft properties. Each selected proof crate is checked in full with `--no-cheating`. Earlier exploratory visibility/compiler failures and intermediate test failures remain under `evidence/joint-design`; the final passing logs are identified above. Final context additions only include the protected observation consumer in the Read list; executable/check behavior is unchanged from the demonstrated run.

## Remaining limits and decisions

The observation vocabulary, public method signatures and borrowed observation interface remain supplied constraints. The generated design is flexible within that boundary. Public fields in the example layouts support this Verus release's open specifications; this revision does not claim private-field encapsulation or arbitrary API synthesis.

`from_observation` is a boundary-test representation constructor, not recovery or a legal Raft transition. It must represent every observation and establish `wf`, so `wf` concerns representation consistency. Reachable-state Raft invariants need separate initialization/preservation proofs. Review this adapter boundary and specification adequacy; a machine proof does not establish that the supplied requirements fully capture intended Raft.

All 26 global obligations remain OPEN. Complete elections, both-side replication, commitment/delivery, recovery, integrated effects, node/cluster refinement, global safety and conditional liveness remain unfinished. The temporal-library integration and host-fairness/resource transfer are still open. [OPEN_DECISIONS.md](OPEN_DECISIONS.md) retains draft storage-failure, legal-caller, incarnation/recovery, effect/send, resource, TCB and isolation choices. This revision approves none of them.

Candidate exclusions prevent accidental answer inclusion, not hostile same-user access. Offline solutions are absent from real candidate copies; generated successes remain only in isolated demonstration candidates. Existing process, scope, checker and irreversible-operation protections remain. Verus/Rust/SMT and host trust remain explicit. Human review follows this delivery.

## Preservation and exact next commands

The pre-edit authored-file backup is `/tmp/raft-joint-design-71tnuk2r/authored.tar.gz`; the [previous review](../evidence/joint-design/previous-review.md), historical evidence and read-only `.git`, `.agents`, `.codex`, `.aws` are preserved. [Changed files](../evidence/joint-design/changed-files.json) and the [source diff](../evidence/joint-design/changes.patch) describe this revision without touching Git metadata.

From the repository root, with a new demonstration output directory:

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m unittest discover -s orchestrator/tests -v
python3 orchestrator/tests/real_repair_demo.py --task PROGRESS-FOUNDATION --redesign --output /tmp/raft-joint-design-demo
./experiment.sh
python3 -m orchestrator run-plan --tasks PROGRESS-FOUNDATION --dry-run
```

The next real task is STATE-INIT: generate the constructor, its representation/local contracts and proof, then independently check them. The following explicit live commands are available but were **not run** during maintenance:

```sh
./experiment.sh run
python3 -m orchestrator run-plan --tasks PROGRESS-FOUNDATION --execute-codex
python3 -m orchestrator resume --run-id RUN_ID --execute-codex
```

The first selects STATE-INIT with configured `gpt-6.1-sol` / `high`; the second continues initialization, persistence and local progress in one candidate. Neither produces a full Raft implementation. Resume keeps the recorded design and cumulative budgets; it does not preload a solution. Delivery stops here for human review.

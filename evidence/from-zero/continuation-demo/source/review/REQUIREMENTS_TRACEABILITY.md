# Requirements traceability — DRAFT

## Current incremental revision

The user's later clarification replaces the bootstrap runtime approval-file rule: human review follows code delivery. Local execution/resume requires an explicit launch but no approval artifact. `scheduler.py`, `cli.py`, `experiment.sh` and the revised offline tests implement that instruction. Historical R-08 below records the original behavior; it is no longer the current execution requirement.

The subsequent user-requested module organization is implemented by eight module documents in `input/`, named-step selection in `materials.py`/`scheduler.py`, `plan --format text`, and `test_modules.py`. Old T files were removed and mapped in the roadmap. Step results cannot imply module completion. No semantic or global proof target changed; current checks are linked from the review packet.

Source: Desktop `CODEX_RAFT_WORKFLOW_REVISION_REQUIREMENTS.md`, 310 lines, read in full. The prior bootstrap-only restriction is replaced only for this revision's authorized scope. Semantic decisions and all 26 global obligations remain open. Actual check outcomes and remaining limits are in [the review packet](REVIEW_PACKET.md).

| Revision sections | Delivered paths and meaning |
|---|---|
| 2–5: readable assignments and separation | `AGENTS.md`, `README.md`, the eight module documents in `input/`, `input/roadmap.md`, `input/model.md`, `input/proofs.md`; complete module responsibilities, bounded internal steps and preserved global proof dependencies |
| 6: bounded feedback and resume | `orchestrator/scheduler.py`, `feedback.py`, `cli.py`; one saved candidate, classified reproduction data and stale-input rechecking |
| 7: local versus full acceptance | `verifier_runner.py`, `workflow.toml`; real typed outcomes, nonempty test selection, actual Verus function attribution and protected proof consumer |
| 8–10: real slice and checking layers | `crates/raft-verified/src/reconcile.rs`, `tests/`, `examples/`, `check.py`; same executable body, explicit replay and repeated core measurements |
| 10: formal correspondence | `crates/raft-spec/src/reconcile_view.rs`, `crates/raft-proofs/reconcile.rs`, `reconcile_lemmas.rs`; unchanged `merged` semantics, protected view/target and proved corollaries |
| 10: failure and repair | `orchestrator/tests/real_repair_demo.py`; fixture-generated truncate-tail defect and repair, independent real Rust/Verus checks |
| 9, 11: impact and controller regressions | `workflow.toml` scoped inputs, `orchestrator/tests/test_revision.py` and existing suites; changed source/checker invalidation, typecheck/test/proof/measurement and resume cases |
| 12: delivery and review | `review/REVIEW_PACKET.md`, `review/OPEN_DECISIONS.md`, `docs/workflow.md`, `evidence/revision/commands.json`; actual results, pending decisions and later authorized commands |

## Historical bootstrap traceability

The following tables describe the original bootstrap delivery, not the new local slice's capabilities or its current test counts. Preserve their historical evidence.


Statuses describe delivery depth, not mathematical conclusions. Original source: desktop `CODEX_RAFT_WORKFLOW_REQUIREMENTS.md` v0.1, 789 lines, read in full. No prior implementation required migration; four empty read-only directories were preserved. Bootstrap execution results below are historical; current layout and persistent-tool checks are recorded separately in the review packet.

## Deliverables

| ID | Actual paths | Status and evidence |
|---|---|---|
| D1 | [input/requirements.md](../input/requirements.md), [input/model.md](../input/model.md), [docs/workflow.md](../docs/workflow.md) | DONE: scope, refinement-boundary diagram and staged workflow |
| D2 | [input/](../input/), [orchestrator/wire/](../orchestrator/wire/), [review/OPEN_DECISIONS.md](../review/OPEN_DECISIONS.md) | DONE: DRAFT contract, 14 assumptions, 8 TCB entries, 10 open decisions; Markdown cross-reference checks |
| D3 | [crates/raft-spec/src/lib.rs](../crates/raft-spec/src/lib.rs), [input/model.md](../input/model.md) | PARTIAL: actual transitions/property definitions; real Verus TYPECHECKED (0 verified); explicit temporal-environment/interface-error/refinement gaps |
| D4 | [input/proofs.md](../input/proofs.md) | DONE: 26 OPEN obligations, 49 coverage entries and DAG; no completed proofs |
| D5 | [orchestrator/](../orchestrator/), [workflow.toml](../workflow.toml) | DONE (local mode): installable CLI, serial tasks and independent checking; formal isolation/full Raft acceptance BLOCKED |
| D6 | [orchestrator/tests/](../orchestrator/tests/), [crates/proof-smoke/](../crates/proof-smoke/), [evidence/bootstrap/](../evidence/bootstrap/) | DONE: 53 offline tests and 2 Rust tests passed; real positive smoke 2 verified, negative rejected, model TYPECHECKED; future Raft tests planned but NOT_RUN |
| D7 | [review/REVIEW_PACKET.md](../review/REVIEW_PACKET.md), [review/OPEN_DECISIONS.md](../review/OPEN_DECISIONS.md) | DONE: decisions, risks, actual records, commands and stopping statement |

## Bootstrap acceptance items

| ID | Path | Status / boundary |
|---|---|---|
| R-01 | [README.md](../README.md) | DONE: actual files and executable commands |
| R-02 | [review/REVIEW_PACKET.md](../review/REVIEW_PACKET.md) | DONE: all seven deliverable groups present; PARTIAL/BLOCKED items disclosed individually |
| R-03 | [input/model.md](../input/model.md) | PARTIAL: model/property/assumption traceability; complete Verus encoding of environmental premises remains open |
| R-04 | [input/proofs.md](../input/proofs.md) | DONE: includes safety, liveness, refinement, cluster composition and liveness preservation |
| R-05 | [orchestrator/setup.cfg](../orchestrator/setup.cfg) | DONE: actual local offline installation and validate/plan/dry-run/report execution |
| R-06 | [orchestrator/tests/](../orchestrator/tests/) | DONE: real subprocess handling, JSONL, schemas, budgets, timeouts and permission errors |
| R-07 | [orchestrator/verifier_runner.py](../orchestrator/verifier_runner.py) | DONE: generation reports separated from independent evidence; empty targets/agent claims do not pass |
| R-08 | [orchestrator/scheduler.py](../orchestrator/scheduler.py) | DONE: missing/example/wrong-scope/mismatched-digest approval rejected; real bootstrap generation disabled |
| R-09 | [evidence/bootstrap/checks.json](../evidence/bootstrap/checks.json) | DONE: actual, SIMULATED and unrun results distinguished; no fabricated tool versions |
| R-10 | [crates/proof-smoke/](../crates/proof-smoke/) | DONE: real smoke results matched expectations; model TYPECHECKED; full Raft acceptance BLOCKED and coverage obligations OPEN |
| R-11 | [review/OPEN_DECISIONS.md](../review/OPEN_DECISIONS.md) | DONE: 10 recommendations with rationale, alternatives, impacts and risks |
| R-12 | [review/REVIEW_PACKET.md](../review/REVIEW_PACKET.md) | DONE: no real Codex generation, recursive CLI, automatic approval/merge/release |

## Top-level targets

These are target definitions, not proved theorems; all remain OPEN. Conclusions are in [properties.md](../input/properties.md); complete premises are in [assumptions.md](../input/assumptions.md).

| ID | Verus symbol | Obligation | Status / evidence |
|---|---|---|---|
| S-ELECTION | [lib.rs::election_safety](../crates/raft-spec/src/lib.rs) | PO-S-ELECTION | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| S-APPEND | [lib.rs::leader_append_only](../crates/raft-spec/src/lib.rs) | PO-S-APPEND | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| S-LOG | [lib.rs::log_matching](../crates/raft-spec/src/lib.rs) | PO-S-LOG | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| S-COMPLETE | [lib.rs::leader_completeness](../crates/raft-spec/src/lib.rs) | PO-S-COMPLETE | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| S-APPLY | [lib.rs::application_safety](../crates/raft-spec/src/lib.rs) | PO-S-APPLY | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| S-INTERFACE | [lib.rs::interface_safety](../crates/raft-spec/src/lib.rs) | PO-S-INTERFACE | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| L-ELECTION | [lib.rs::l_election](../crates/raft-spec/src/lib.rs) | PO-L-ELECTION | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| L-COMMIT | [lib.rs::l_commit](../crates/raft-spec/src/lib.rs) | PO-L-COMMIT | OPEN; no theorem-proof evidence (model only TYPECHECKED) |
| L-DELIVER | [lib.rs::l_deliver](../crates/raft-spec/src/lib.rs) | PO-L-DELIVER | OPEN; no theorem-proof evidence (model only TYPECHECKED) |

## Individual proof obligations

[The proof checklist](../input/proofs.md) contains all 26 obligations, dependencies, assumptions, TCB references, coverage and the dependency graph. Every obligation remains OPEN. There is no completed-proof evidence.

## Offline test coverage

| Requirement group | Test files |
|---|---|
| Process failures, permissions, cancellation, descendants, long input, large output and shell characters | [test_process.py](../orchestrator/tests/test_process.py) |
| JSONL/schema/missing reports/unknown events/report-directory symlinks/false success/checker output | [test_adapters.py](../orchestrator/tests/test_adapters.py) |
| Modes/deletion/rename/untracked files/symlinks/path traversal/nested reserved names | [test_workspace.py](../orchestrator/tests/test_workspace.py), [test_policy.py](../orchestrator/tests/test_policy.py) |
| Approval/DAG/budgets/repeated failures/stale evidence/default no-call/builds cannot close proofs | [test_scheduler.py](../orchestrator/tests/test_scheduler.py) |
| Assumption/TCB references, top-level targets, coverage, unproved CHECKED status, safety relying on liveness | [test_registry.py](../orchestrator/tests/test_registry.py) |

At bootstrap, deterministic Raft simulation, property tests, fault injection, protocol mutations and reference-implementation differential tests were NOT_RUN; see [proofs.md](../input/proofs.md).

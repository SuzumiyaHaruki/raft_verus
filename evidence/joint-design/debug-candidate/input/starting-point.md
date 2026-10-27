# Experiment starting point

Status: DRAFT. The experiment studies AI generation of executable Rust Raft behavior and its Verus proofs, starting without an implemented Raft operation. This is synthesis from supplied requirements and interfaces, not unconstrained synthesis from a blank file.

## Supplied inputs

English requirements and module assignments, the candidate formal model, exact local contracts and protected proof consumers, shared data types and API signatures, independent tests, host replay drivers, measurement harnesses, Cargo configuration and the Python controller are supplied. Record these inputs when describing results. Tests and host drivers call the library; they are not alternative Raft implementations. Structural derives and representation functions are part of the supplied interface/specification.

## Missing work

`node.rs`, `reconcile.rs` and `persistence.rs` contain intentionally incorrect error-only seeds. They compile, but do not implement their contracts. `progress_lemmas.rs` supplies target signatures with empty proof bodies and no induction strategy. Other editable lemma declarations also have empty bodies; Verus may discharge simple supplied targets automatically, which is not evidence of an AI-authored proof strategy. No seeds use trusted bodies, assumptions or skipped verification. Missing behavior and unproved targets must fail their independent checks.

Generate the selected executable body and its proof in the same source. Subsequent steps and repairs build on the candidate produced during this run. Resume retains that candidate and its history; it must not restore seeds or import a demonstration solution. Earlier completed tasks are rechecked when later changes can affect them. Future unimplemented tasks are not prerequisites for completing the current bounded assignment.

## Evidence and scope

Offline controller tests and scripted solutions are maintenance material and are omitted from candidate copies, along with historical evidence and maintenance instructions. Do not search outside the candidate for answers, historical implementations or fixture code. Scripted generation is SIMULATED even when its Rust/Verus checks are real. These local directory boundaries prevent accidental answer inclusion; they are not hostile-process OS isolation.

A new run copies the checked-out seed source; a resume continues the recorded generated candidate. If the source checkout is manually populated with an implementation, a new copy is an extension experiment and must be described as such. No automatic reset, solution overlay or source merge occurs. Completion concerns selected tasks only. Full Raft safety, conditional liveness and implementation/cluster refinement remain open until their actual proofs are checked.

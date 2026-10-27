# Conditional liveness and implementation progress

Status: DRAFT. Module: INCOMPLETE; temporal premises and end-to-end progress proofs remain open. Liveness is mandatory for the final Raft result.

## Responsibility and connections

Establish eventual stable leadership, commitment of eligible retried client requests and delivery to continuously participating quorum nodes. Transfer those abstract results to the actual event-driven implementation, including terminating local calls, external waits, scheduler fairness and finite resources. Use the completed election/replication/commitment/runtime paths and their safety invariants.

Read [assumptions.md](assumptions.md), [properties.md](properties.md), [proofs.md](proofs.md), `valid_trace`, fairness helpers, `l_election`, `l_commit` and `l_deliver` in the model. Encode existing premises transparently; never assume a stable leader, successful vote or eventual commitment as an environment premise. Tool/temporal-library compatibility is unresolved. New premises or weaker targets need review; finite tests are not substitutes for these proofs.

## Module completion

Actual Verus entrypoints establish all three conditional-liveness targets and PO-LIVE-REFINEMENT under explicit non-circular premises. Safety proofs do not inherit A-L assumptions. Higher-term interference, old-message draining, persistence, request service, internal stuttering and resource bounds are accounted for. Finite post-fault scenarios check engineering behavior separately. One election window lemma does not finish election liveness, much less this module.

## Step: Establish coordinator progress foundation

Stage: READY after RUNTIME-PERSIST. Generate the missing proof bodies in `crates/raft-proofs/progress_lemmas.rs` against the protected `progress.rs` consumer and `raft-spec/src/progress.rs` target. The supplied target requires eventual release of the pending reply when a matching successful persistence callback receives service after finitely many rejected or unsupported events. `execution` relates successive observations by the independent transition relation; the checked actual `Node::step` preserves its generated representation invariant and establishes that relation through its view; `fair_completion` constrains event service and noninterference, never assumes the output. The host still owes actual durability before a successful notification. Identify the first failed induction step when revising the proof; do not strengthen premises or edit the target.

Run `python3 crates/raft-verified/check.py progress-proof` or `python3 -m orchestrator verify --profile progress-proof`, then the persistence tests and replay on the shared initialized candidate. Use the installed Verus/vstd combination with unbounded `nat` traces and quantifiers; choose the needed auxiliary lemmas and induction strategy within the editable file; no waiting-prefix lemma skeleton is prescribed; no external temporal library is needed for this local encoding. The optional verus-tla library remains unavailable and uncalibrated. Check the generated coordinator handlers; this task does not prove termination of all future handlers or fair scheduling by an actual host.

Retain global L-ELECTION, L-COMMIT, L-DELIVER and PO-LIVE-REFINEMENT. Finite counter/capacity sufficiency, competing protocol events, higher-term interference and transfer of fairness through real scheduling remain OPEN. Invalid events may delay service only finitely in this theorem; continuing storage failure or missing successful completion does not satisfy its premises.

## Step: Establish persistent leader progress

Stage: PLANNED. Requires integrated election paths, fault scenarios and safety invariants.

First formalize the existing admissible trace conditions and prove the bounded exclusive-election-window lemma. Account for eligible logs, durable votes, unique quorum members, higher-term traffic and old messages. Then derive persistent leadership from the permitted continuing election/service opportunities; do not stop at the bounded window result. Split proof work into explicit subgoals and bounded reviewed scopes before any agent call that cannot fit the current budget.

Deliver trace definitions, election lemmas, the conditional l_election theorem and finite scenarios stating their post-fault window. Real temporal proof/scenario profiles and prerequisite invariants are missing. No runnable proof is claimed. Record incomplete subgoals and tool limitations rather than marking the module done.

## Step: Establish request commitment progress

Stage: PLANNED. Requires persistent-leader progress and commit/log safety.

Prove that every eligible persistently retried and fairly served Client request has at least one matching entry eventually committed. Link admission, durable append, replication retry and current-term majority progress. ProposalAccepted and internal Noop are not witnesses. Keep request identity/content binding and finite-resource premises explicit; do not assume the desired future acceptance or commitment.

Deliver the dependent lemmas and l_commit proof entrypoint, plus finite fair-service regressions against the actual core. Activate separately scoped lemma tasks and real proof profiles once prerequisites exist. Today both are missing; completion requires actual proof of the stated request theorem, not one successful client trace.

## Step: Establish delivery progress

Stage: PLANNED. Requires leadership/commit progress and the delivery coordinator.

Prove eventual ordered Deliver observations for committed entries at each continuously participating quorum node under the stated conditions. Separate delivery from host application and exclude permanently failed nodes as the contract specifies. Derive replication catch-up and fair delivery service, respecting finite index/resources.

Deliver l_deliver and supporting proofs plus finite post-fault service regressions. Actual theorem/scenario profiles are missing. Do not infer a theorem from a timeout test; failure to prove or finish a finite run needs a precise diagnosis.

## Step: Transfer progress to executable code

Stage: PLANNED. Requires abstract progress, dispatcher correspondence and cluster composition.

Prove termination of each local operation, account for every external wait, exclude infinite internal stuttering, and transfer required fairness through the concrete host/event boundary. Derive or explicitly constrain representation/capacity budgets for each promised request. Safe resource exhaustion alone does not establish liveness.

Deliver implementation-progress arguments and checked refinement entrypoints over the same Rust logic used at runtime. Missing scheduling/refinement/resource proofs and independent profiles keep this step planned. The final integration review must include these proofs; no complete-Raft claim is allowed without them.

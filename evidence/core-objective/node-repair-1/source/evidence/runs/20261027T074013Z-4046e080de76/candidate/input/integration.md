# Integrated Raft validation and performance

Status: DRAFT. Module: INCOMPLETE; local reconciliation checks exist, while a
complete node, real adapters and whole-project proofs are absent.

## Responsibility and connections

Connect [state](state.md), [runtime](runtime.md), [elections](elections.md),
[replication](replication.md), [commitment](commitment.md), [recovery](recovery.md)
and [progress](progress.md) into a usable Rust library and a continuously running
multi-node example. Tests, proofs, fault scenarios and measurements exercise the
same core. A host adapter implements the declared I/O contract; a simulator does
not establish that an actual network/storage/timer adapter satisfies it.

## Module completion and whole-project acceptance

The integrated library runs repeated elections, proposals, replication, commitment,
delivery and recovery over multiple terms. Real-adapter and deterministic-fault
tests pass. Actual final-candidate Verus checks cover every executable/event/error
path, global safety, all conditional liveness and implementation/cluster refinement
in [proofs.md](proofs.md). DRAFT semantics and TCB decisions require human review.
The existing `full` profile stays incomplete until its genuine prerequisites exist.
No count of passed local steps can replace these conditions.

Measurements distinguish core operation, proposal acceptance, commitment and delivery.
Record code/build/tools, workload, repetitions, sizes, batching/concurrency, machine,
resource use and timing boundary. Do not invent thresholds or compare different
durability modes as equivalent. An optimization reruns affected tests and proofs.

## Step: Measure reconciliation cost

Stage: READY after the checked local operation and its replay regression.

Run the existing shared-operation benchmark at several log lengths and matching,
conflicting and missing-suffix workloads. Retain all repeated samples and units,
aggregation, command size, batching/concurrency, machine/build metadata and input
preparation/allocation boundaries. Run serially without competing heavy checks.
Read `crates/raft-verified/examples/measure_reconcile.rs`, `check.py` and
[the workflow guide](../docs/workflow.md) before interpreting the result.

Edit only `review/measurements/reconciliation.md`; core and harness are protected
in this task. Run `python3 -m orchestrator verify --profile reconcile-bench`
(or `python3 crates/raft-verified/check.py bench` for direct reproduction). Require
nonempty repeated data on the same code checked for correctness. Missing data
means incomplete measurement; no approved reference/threshold means INCONCLUSIVE
performance, not a pass/regression verdict. Explain noise and repeat comparable
measurements before optimizing. This is a core microbenchmark, not cluster throughput.

## Step: Connect and test real host adapters

Stage: PLANNED. Requires the integrated dispatcher and reviewed host contract.

Implement concrete transport identity/codec handling, durable storage/recovery and
timer delivery for a runnable multi-node example. State external device/library
trust explicitly and test ordering/durability/error behavior against io.md. Keep
the protocol in the shared core; do not hide its coordinator in an unverified host.

Deliver adapters, repeated client/protocol execution example and real integration
checks including restart and delayed completions. Adapter choice and any TCB
expansion remain pending ADR-07 review; no adapters or real profiles exist today.
A passing simulator cannot complete this step.

## Step: Compose global safety proofs

Stage: PLANNED. Requires integrated handlers and recovery scenarios.

Build the inductive invariant and prove initialization and preservation for all
model transitions. Connect all concrete event/error paths and I/O waits to that
model, compose nodes with the network/storage environment, then derive historical
election safety, append-only leadership, log matching, leader completeness,
interface safety and application safety with their declared assumptions.

Deliver real invariant/refinement/composition lemmas and global safety entrypoints.
Use the existing obligation/dependency map; never replace a missing induction
argument with a helper proof or function-name match. These source entrypoints and
profiles are absent. Plan individual dependent lemmas with bounded editable scopes
before execution. Local proofs remain useful but do not close this step by themselves.

## Step: Measure integrated workloads

Stage: PLANNED. Requires real adapters and reproducible fault-free/recovery workloads.

Measure repeated client workloads across actual nodes. Report proposal acceptance,
commit and delivery latency/throughput separately, with comparable durability modes,
log/command sizes, batch/concurrency settings and machine/resource observations.
Save all samples; report noise and absent thresholds honestly. Keep fault recovery
measurements separate from steady operation and exclude setup explicitly where appropriate.

Deliver cluster measurement harness/results and actual local profiles once the
runtime/adapters exist. They are missing today. Any justified optimization is a
separate scoped change with affected correctness regressions; measurements cannot
establish protocol safety or liveness.

## Step: Review complete Raft coverage

Stage: PLANNED. Requires all module implementation, safety, progress and adapter work.

Run the entire functional/fault/integration suite, affected measurements and actual
full Verus coverage on the final unchanged candidate. Reconcile executable entrypoints
with every event/error/model branch and all global obligations. Review remaining
assumptions, TCB, semantic decisions and tool limitations. Record actual commands,
results and outstanding gaps in review/REVIEW_PACKET.md.

The current command `python3 -m orchestrator verify --profile full` reports BLOCKED;
the planned full proof entrypoint and complete coverage are absent. Do not treat
that as success or enable this step prematurely. Human review of a complete evidence
package is distinct from a selected-step plan finishing. No merge, push or publication
is implied by this task or any check result.

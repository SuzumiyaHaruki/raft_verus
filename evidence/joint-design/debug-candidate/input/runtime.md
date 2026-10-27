# Event processing and I/O coordination

Status: DRAFT. Module: INCOMPLETE; interfaces and independent checks exist; coordinator, dispatcher and protocol behavior await generation.

## Responsibility and connections

Make the Raft node run continuously as an event-driven library. The host submits messages, timer expirations, proposals, completions and recovery inputs through `Event`; the library returns `Effect` values. The host performs I/O and returns completion events. The library owns protocol ordering and pending work; the host owns devices and scheduling under [io.md](io.md).

Dispatch to [elections](elections.md), [replication](replication.md), [commitment](commitment.md) and [recovery](recovery.md) over the shared [state](state.md). No new thread/runtime framework is required. Repeated calls must form a valid node execution, including calls while persistence is pending. Read `write`, `storage_durable`, `storage_ack`, `flush_message`, `storage_error`, `stale_completion`, `step` and `next` in the model and existing token/completion types in the executable library. Public API and error semantics remain DRAFT.

## Module completion

Every Event variant has an implemented path, including invalid inputs and delayed, duplicate, wrong-kind and old-incarnation completions. Required writes precede positive replies; unknown persistence outcomes cannot report success. Timers and pending work survive role changes correctly or are explicitly invalidated. Functional/fault checks execute the actual dispatcher. Local correspondence and I/O proofs compose into PO-IO, PO-REFINE-STEP and PO-COMPOSITION; the progress module must also establish termination and fairness transfer. Dispatching one happy-path message is insufficient for module completion.

## Step: Coordinate persistence and completions

Stage: READY after STATE-INIT in the same saved candidate. Implement and prove `Node::submit_persist`, `Node::complete_persist` and `Node::step` in `crates/raft-verified/src/persistence.rs`, together with `crates/raft-proofs/persistence_lemmas.rs`. The supplied bodies are intentionally incomplete error-only seeds; no coordinator solution is included. Read `state.rs`, `io.rs` and the protected `raft-spec/src/persistence.rs` contracts and `raft-proofs/persistence.rs` consumers first. Ordinary attempts cannot change shared representation, independent targets/tests, the driver or checker configuration; propose a representation extension explicitly if it exceeds this scope.

There is one pending write. Submission moves the proposed term/vote/log image into `Node.hard`, allocates `(incarnation, serial)` without wrapping, retains an optional dependent protocol reply, and returns only the persistence token. The host snapshots `hard` under that token before notification; submission is not durable success. While pending, reject new writes with Busy and preserve the current image/reply. Completion compares node incarnation, full token and Persist kind before consuming anything. Success returns the retained reply exactly once; duplicate, stale, unknown and wrong-kind callbacks preserve state and release nothing. NotPerformed returns StorageFailure, Unknown returns RecoveryRequired; both enter the current model's conservative fail-stop policy and release no reply. Recovery and a direct known-failure retry path are not implemented. These draft API choices are recorded under ADR-01/07, not newly approved semantics.

Future election handlers must establish legal term/vote changes and reply authority; replication handlers must establish log/commit-prefix legality before calling `submit_persist`. Coordinator targets intentionally accept arbitrary supplied images: they prove storage ordering, not that such an update is legal Raft. A successful host callback must certify the entire snapshot is recoverable. The local `storage_projection`, `submit_projection` and `completion_projection` connect the represented fields and successful path to `write`/`storage_ack`; full error/recovery, outbox/send and cluster refinement remain open. The returned reply still needs a separately identified Send operation in the future dispatcher.

Run `python3 crates/raft-verified/check.py test --test persistence`, `python3 crates/raft-verified/check.py persist-proof` and `python3 crates/raft-verified/check.py persist-replay`. The six independent tests cover withheld replies, Busy, identity freshness, all matching dimensions, fail-stop outcomes, exhaustion and unsupported events. The saved `examples/persistence.trace` drives the actual `step` method, with separate host disk changes and delayed/reordered/duplicate callbacks; it is a coordinator/completion scenario, not crash recovery or a cluster fault test. `python3 crates/raft-verified/check.py persist-bench` measures in-memory submission/completion only, after correctness checks pass.

`python3 -m orchestrator run-plan --tasks RUNTIME-PERSIST --dry-run` previews STATE-INIT then this step. An explicitly executed plan carries the checked constructor into this task in the same candidate, without copying or merging. Keep all global Raft properties open. Runtime completion still requires every handler integrated through the growing dispatcher.

## Step: Integrate the node dispatcher

Stage: PLANNED. Requires election, replication, commitment and recovery handlers.

Connect every existing Event variant to the appropriate state transition and ordered effects. Document dispatch behavior while work is pending and after role changes. Drive the same node repeatedly from a host queue; do not write a second implementation in the simulator. Exercise proposals, messages, both timers, completion outcomes and recovery over multiple terms. Retain explicit error paths.

Deliver the public node entrypoint, an executable host-loop example and tests for all event/error branches. Add local step/refinement entrypoints with meaningful state invariants, then check the integrated node and dependent proofs. RUNTIME-PERSIST must first generate a partial `Node::step` that handles persistence completions and explicitly rejects every unsupported Event variant. Its return type will need a reviewed extension for general effects; all-event handlers and independent profiles are still missing, so this remains PLANNED. Missing handlers cannot be replaced with success stubs.

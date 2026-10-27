# Event processing and I/O coordination

Status: DRAFT. Module: INCOMPLETE; event/effect types and token helpers exist,
but no node dispatcher or persistence coordinator is implemented.

## Responsibility and connections

Make the Raft node run continuously as an event-driven library. The host submits
messages, timer expirations, proposals, completions and recovery inputs through
`Event`; the library returns `Effect` values. The host performs I/O and returns
completion events. The library owns protocol ordering and pending work; the host
owns devices and scheduling under [io.md](io.md).

Dispatch to [elections](elections.md), [replication](replication.md),
[commitment](commitment.md) and [recovery](recovery.md) over the shared
[state](state.md). No new thread/runtime framework is required. Repeated calls
must form a valid node execution, including calls while persistence is pending.
Read `write`, `storage_durable`, `storage_ack`, `flush_message`, `storage_error`,
`stale_completion`, `step` and `next` in the model and existing token/completion
types in the executable library. Public API and error semantics remain DRAFT.

## Module completion

Every Event variant has an implemented path, including invalid inputs and delayed,
duplicate, wrong-kind and old-incarnation completions. Required writes precede
positive replies; unknown persistence outcomes cannot report success. Timers and
pending work survive role changes correctly or are explicitly invalidated.
Functional/fault checks execute the actual dispatcher. Local correspondence and
I/O proofs compose into PO-IO, PO-REFINE-STEP and PO-COMPOSITION; the progress module
must also establish termination and fairness transfer. Dispatching one happy-path
message is insufficient for module completion.

## Step: Coordinate persistence and completions

Stage: PLANNED. Requires initialized node state.

Implement pending writes and completion matching using existing OperationId,
CompletionKind and CompletionOutcome. Allocate fresh tokens; separate a durable
write from the notification that releases its dependent reply. Route Success,
NotPerformed and Unknown according to the existing I/O contract; do not infer
that a lost acknowledgment means a write did not happen. Keep the permitted
pending-operation policy explicit; ambiguity belongs to ADR-01/07.

Deliver the coordinator, tests of duplicate/wrong/stale tokens and uncertain
outcomes, and a local storage/completion correspondence proof. Save a trace that
crashes between durable storage and callback. Supply actual source scope,
coordinator tests and Verus profiles before activation; these are missing today.
Passing those checks completes this step only. Runtime completion still requires
all handlers integrated through the dispatcher.

## Step: Integrate the node dispatcher

Stage: PLANNED. Requires election, replication, commitment and recovery handlers.

Connect every existing Event variant to the appropriate state transition and
ordered effects. Document dispatch behavior while work is pending and after role
changes. Drive the same node repeatedly from a host queue; do not write a second
implementation in the simulator. Exercise proposals, messages, both timers,
completion outcomes and recovery over multiple terms. Retain explicit error paths.

Deliver the public node entrypoint, an executable host-loop example and tests for
all event/error branches. Add local step/refinement entrypoints with meaningful
state invariants, then check the integrated node and dependent proofs. The
entrypoint signature, handlers and independent profiles are not present yet;
this remains PLANNED. Missing handlers cannot be replaced with success stubs.

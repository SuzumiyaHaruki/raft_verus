# Failure handling and recovery

Status: DRAFT. Module: INCOMPLETE. Full crash recovery awaits generation; the supplied MadSim host and independent fault scenarios exercise the actual public event handler. The older reconciliation replay remains a local helper scenario.

## Responsibility and connections

Preserve protocol and host-interface guarantees across crashes, restarts, delayed completions, stale messages and network faults. Reconstruct [state](state.md) from durable input, restore [runtime](runtime.md) coordination safely, and exercise the actual [election](elections.md), [replication](replication.md) and [delivery](commitment.md) paths. Read `crash`, `recover`, storage/application steps and `stale_completion` in the model and [io.md](io.md).

## Module completion

Recovery uses actual durable images/cursors, invalidates obsolete tokens/timers and never revives volatile acknowledgments as durable facts. Check crashes before a write, after durability before callback, and after application before ack. The deterministic cluster driver supports message loss/duplication/reordering, partitions, timer events, crashes/restarts and I/O completions with explicit traces and state observations. Local recovery/I/O proofs and cluster composition are required; passing scenarios alone does not discharge PO-RECOVERY or PO-COMPOSITION.

## Step: Restore a crashed node

Stage: PLANNED. Requires node state and the persistence coordinator.

Implement Event::Recover from host-provided DurableState, a valid application cursor and a fresh incarnation under the draft storage contract. Reset volatile role/progress, validate index/cursor domains and reject old-incarnation completions. Account for a completed write or application with a lost callback; never reconstruct success solely from the last observed callback. Keep invalid recovery input and resource exhaustion explicit.

Deliver shared recovery code, crash-boundary/cursor/token tests and a local `recover` correspondence proof. Define source scope and real recovery check profiles before execution. The active MADRAFT-RECOVERY step now provides a scope and public checks over the shared Node; this earlier route remains planned. Resolve any mismatch in durable-image or incarnation semantics under ADR-03/06/07.

## Step: Exercise cluster fault histories

Stage: PLANNED. Requires the integrated node dispatcher.

Drive several actual nodes using a deterministic event queue. Add explicit message, timer, crash/recovery and completion events; save both seed and full trace with relevant node states. Cover competing elections, partition/healing, divergent logs, duplicate/stale RPCs and delayed storage/application callbacks over multiple terms. Minimize a failing trace when practical and retain it as a regression.

Check historical per-term elections and committed/delivered observations. Temporary differences in uncommitted logs are legal; old leaders in different terms during a partition are not automatically an election-safety violation. For progress scenarios, state when failures cease and the later communication/timer/service/resource bounds. Finite scenarios expose engineering failures and never prove infinite-trace liveness.

Deliver the driver, reproducible regression traces and independent scenario profiles against the real node, not a second Raft implementation. The active MADRAFT-RECOVERY step now supplies the real MadSim driver and profiles; this earlier route is not the default. Oracle changes are reviewed work; ordinary repairs cannot rewrite expected results. Attribute failures provisionally to core, adapter, driver or expectation and retain the evidence.

## Step: Generate recovery and exercise the complete core

Stage: READY as MADRAFT-RECOVERY after MADRAFT-REPLICATION. Generate recovery in the same handle_event implementation. The host destroys the old node, calls this run's generated Node::new, reads the actual saved durable image/application cursor and supplies a fresh incarnation. Restore protocol state, discard volatile peer/vote/pending work, persist the incarnation with the image before dependent outputs, and reject old callbacks/timers. Unknown writes require recovery from actual storage; they are not evidence that nothing was written.

The host implements the disclosed atomic whole-image store and application-plus-cursor transaction. It does not choose protocol outcomes. Exercise crashes before writing, after durable writing but before notification, and after application but before acknowledgment. Preserve election, commit and application histories across restarts. Check the final node log and actual applied Client contents, not just successful process exit.

Generate recovery contracts, token/state reconstruction invariants and their connections to prior election/log/commit proofs. Re-run core-all-tests, cluster-tests, cluster-replay, interface-check and core-proof. The fixed-seed replay compares complete actual event histories; tests do not prove temporal liveness or universal safety. Preserve the first failing trace and diagnose whether the core, host or expectation is responsible.

Completion of the selected development steps produces a locally checked candidate, not automatic five-property acceptance. Deliver a property-by-property proof status and the remaining PO-RECOVERY/PO-IO/PO-REFINE-STEP/PO-COMPOSITION dependencies. The safety-only aggregate remains BLOCKED until its independent complete-execution consumers exist; full requires all safety and conditional-liveness goals, including concrete progress refinement. Do not fabricate a final proof file or relabel local checks to make either aggregate pass.

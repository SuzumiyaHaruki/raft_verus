# Election Safety across completed-write restarts — DRAFT

Generate one actual MadRaft receiver/campaign/collection/promotion/recovery Core and its Verus proofs from this unfinished input. The new obligation is historical Election Safety across arbitrary finite repetitions of the two local completed-write restart windows, interleaved with other nodes and existing network traffic. Do not import a former run. The other four safety properties, full MadRaft, interrupted-write recovery and conditional liveness remain OPEN.

## Preserved semantics and the one extension

| Existing rule | This task | Reason |
| --- | --- | --- |
| Receiver, log freshness, checked terms/IDs, self-save, reply source, distinct votes, strict majority promotion | Unchanged `api.rs`, `core.rs` contracts and 38 tests | Recovery must not weaken earlier obligations |
| One Save both chooses I/O outcome and calls complete in `network.rs` | Kept for all 18 old proof groups; `recovery_network.rs` separates Write and Notify | Express a successful write without an invented completion |
| No restart in the old multi-node theorem | Stop/Restart in the new fixed executor | Compose existing local recovery with network history |
| Four local recovery windows | Still independently checked | Local composition alone is not a cluster theorem |
| Current-instance Choice history | Core recovery may clear it; fixed global ledger retains actual observations | A voter restart must not erase evidence for a past election |
| Return callbacks belong to the original caller | Send carries a host incarnation; a dead caller's callback cannot address its replacement | Retain old packets without reviving old futures |

## Storage windows and environment

Window A permits stopping after the latest complete write succeeded and the Core consumed that matching success. Subsequent promotion and same-round collection are allowed; an accepted new write or any change to persistent state/next invalidates this permission. Window B permits stopping after the pending write succeeded, before Notify. The retained image is exactly that node's actual proposed Write, with its instance and write-event index. All existing save kinds are covered: positive/negative receiver decisions, self-votes and higher-reply-term Persisted writes. Pending=None by itself grants no restart permission. Neither window is conditioned on vote uniqueness, a majority, an invariant or Election Safety.

Write(false) does not update the retained image or authorize recovery. Notify carries only the current pending operation's recorded result in this instance. Stop discards the actual Core and pending work. Restart consumes only this node's retained image through Core::recover, clears volatile votes/round/Pending, and never fabricates an old Reply, Send, Choice or Elect. A new write is needed before another allowed stop of that new instance. Different nodes may repeat these operations in any finite interleaving; the environment is not globally quiescent. The framework uses checked u64 incarnation IDs and usize event storage, with no wraparound; histories are general within those finite machine bounds.

Already authorized requests can reach a live peer after the sender stops; already generated replies can reach the same still-live caller even after the voter restarts. Messages and observations are never globally cleared. The original send's host incarnation must match the current live caller for Deliver/Error. A callback belonging to a killed caller remains historical evidence only; the executor does not relabel it or feed it to a new Core. This is cancellation/association plumbing, not a new Raft wire field or a protocol voting decision. Duplicate/reordered/delayed messages are allowed; dropping means never scheduling delivery. Stopped nodes do not process protocol calls.

Not covered: interrupted create/write, failed-write recovery, partial/torn/stale/corrupt reads, unsaved volatile updates, physical power-loss durability, arbitrary restart points, logs-to-summary correspondence, replication, commitment, application and snapshots. The root environment and full task are not weakened by this scoped theorem.

## Actual calls, observations and proof responsibilities

The real registered RequestVote path and RaftHandle driver call the same `madraft/src/raft/vote_core.rs` that Verus imports. The host only serializes operations, writes/reads the existing single file, carries immutable RPC context, forwards actual output, and cancels killed tasks. Its truthful I/O, readback, retention, peer/address mapping, encoding and scheduling remain UNVERIFIED premises, tested with real MadSim files and same-node kill/restart. They do not assume vote uniqueness or valid majorities.

The new fixed executor mechanically records every entry's before/after state, live/dead status, actual output choices and actual Write lifecycle. Its `represented` predicate is only data correspondence. It supplies no cross-instance protocol invariant. `recovery_lemmas.rs` is intentionally false/unimplemented: choose and prove your own invariant, initialization, single-step preservation and conclusion. Relate the archived ledger to retained term/votedFor and the new Core suffix; preserve sources and old winning majorities after voters or leaders restart. Never turn recovery correctness into a requires clause left open by final acceptance.

`caller.rs::finite_restarts` executes any structurally enabled finite sequence, checks record origin, cross-instance Choice uniqueness, current and historical vote provenance and historical Election Safety at each prefix, then connects the exact events to the original `requirements/targets.rs` predicate. Restart is not enabled by any of those target predicates. The nonempty caller exercises both windows, denial after restart, actual majority promotion, repeated restarts and a higher-term reelection. The old no-restart executor remains independently required; its combined Save corresponds to adjacent Write/Notify in the new executor without Stop/Restart. Local fixtures may start from boundary states; cluster initial states are legal Followers.

## Work and feedback

Start by implementing and verifying actual Core recovery and handlers together; then connect real retained-write origins to the cross-instance ledger. Preserve local results while attacking the first open cluster dependency. All protocol implementation, auxiliary invariants, majority reasoning and proof decomposition are yours to generate. Do not edit fixed tests, specifications, callers, adapters, tool configuration or acceptance. If a fixed input is contradictory, submit `madraft/OBJECTION.md` with the precise failing command and minimal reproduction; do not change the standard yourself.

Use `python3 scripts/check.py obligation --task election-recovery`, `python3 scripts/check.py safety --task election-recovery`, and `python3 scripts/check.py final --task election-recovery --timeout 600`. Final checks run all existing and new test cases at seeds 1–5, all 18 preserved proof groups and the three new groups on one stopped candidate, verifying imported bodies with `--no-cheating`. Local/conditional verification is not final success. Solver timeout, proof failure and a missing tool are different diagnostics; decompose proofs or adjust legitimate triggers/spinoff_prover, never skip candidate bodies or expand acceptance limits yourself.

Keep `madraft/PROGRESS.md` and `RESULT.md` concise. Distinguish no-restart Election Safety, completed-write restart Election Safety, four local windows, arbitrary crash recovery, host premises and full Raft. Only `ELECTION_SAFETY_COMPLETED_WRITE_RESTART_CHECKS_PASSED` is the new scoped check status; it is not full Election Safety or semantic approval.

Round 5 redesigns only the failed recovery-step decomposition, motivated by independent check-4 resource failures in disk_step, nodes_step and provenance_step. Core bodies, fixed inputs, actual restart windows and retained Choice/message/election history are unchanged. Disk lifecycle and pending-RPC association now have nine event-specific lemmas using small per-node premises; node linkage composes local state, ledger locks, disk facts, pending association and suffix retention through a pure composition lemma. Provenance separates individual votes and historical election frames. Solver limits remain below the fixed default; tools.ini and acceptance are unchanged.

Checked redesign progress: disk-case redesign 1793627587489315253 verified 122 functions with only nodes/provenance resource failures. After further splits, 1793627815248236216 verified 138 functions; all disk and node-linkage obligations passed. Remaining diagnostics were a generic vote-case resource limit and one frame node-index assertion. The last revision splits vote evidence by event type and adds net.nodes.len<=usize::MAX to the invariant/initialization contract. This is the real Vec-based cluster bound, to be discharged by the unchanged constructor/caller, not a new voting or restart premise. Rechecking that affected initialization and full actual finite caller is in progress.

Failures remain retained: 1793627632811186123, 1793627712325715002 and 1793627739078110850 timed out, with resource diagnostics in the earlier broad pending/nodes composition. These are distinct from the frame-index assertion in 1793627815248236216. Earlier independent check-4 passes all 18 preserved groups, the concrete restart witness, build/inventory and seed-1 election tests; prior final runs passed all 43 election tests at seeds 1–5. Actual source induction and no-restart executable safety remain checked dependencies. General completed-write restart acceptance remains OPEN until all imported step bodies pass.

Current obstacle: confirm the event-specific vote helpers, the actual Vec cluster-bound discharge, and the final frame/provenance composition within the fixed check limits; then run all scoped groups/tests on stopped code. Host I/O/readback, encoding, peer mapping, immutable callback incarnation/cancellation, async forwarding and scheduler contracts remain UNVERIFIED. Full Raft/labs, the other four global properties, arbitrary/interrupted-write recovery, snapshots and conditional liveness remain OPEN. The root progress-discovery objection remains separate from proof failure, resource limits and checks not run.

<!-- obligation -->
```ini
[obligation]
id = restart-ledger-preservation
statement = Actual recovery initialization establishes ledger/disk/source invariants, and every enabled recovery step must preserve archived Choices and current/historical election evidence.
goal = election_safety
kind = step
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::recover, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/recovery_check.rs
parent =
dependencies = madraft/proof/recovery_initial_check.rs, madraft/proof/source_check.rs, madraft/proof/network_check.rs
premises = Initialization and source component checked; ledger/disk/provenance step preservation remains open. Host I/O, encoding, identity, callback association and scheduling are unverified.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/network_check.rs | Actual no-restart initialization checked; restarted-cluster invariant open. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/network_check.rs | Actual no-restart preservation checked; cross-instance ledger lock open. |
| outputs | madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::start_election | madraft/proof/network_check.rs | Actual no-restart sources checked; host I/O and RPC mapping unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/source_check.rs | All-prefix write origins checked; archived vote/majority composition open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Unimplemented and unproved. |
<!-- /connections -->

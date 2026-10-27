# Node state and initialization

Status: DRAFT. Module: INCOMPLETE; initial-state types and independent checks exist; constructor generation is pending.

## Responsibility and connections

Represent a fixed-membership Raft node throughout its lifetime: follower, candidate and leader roles; durable term, vote and log; volatile commit/delivery positions; election votes, peer replication progress and pending operations. This state is shared by [elections](elections.md), [replication](replication.md) and [commitment](commitment.md). The [runtime](runtime.md) serializes events; [recovery](recovery.md) reconstructs state after a crash.

The observable-state vocabulary is in `crates/raft-verified/src/state.rs`. It includes identity, membership, durable image, role, commit/delivery/application positions, incarnation, serial, deadline and pending persistence information. The implementation's Node layout, view and representation invariant live in editable `node_state.rs`; different internal organizations can expose the same observation. Peer progress and recovery must extend this same implementation with reviewed observable requirements. The model's `fresh_node`, `initial`, `init` and `well_formed` remain independent targets.

## Module completion

Fresh construction, role-specific fields and recovered construction have explicit bounds and representations. Every handler uses this state; no parallel test-only node exists. Tests cover legal/invalid membership, initial state and index/resource boundaries. Actual proofs establish initialization and representation and supply the invariant required by handlers. PO-INIT, PO-REPRESENTATION, PO-REFINE-INIT and index/token lemmas in [proofs.md](proofs.md) remain open until checked.

## Step: Initialize node state

Stage: READY for generation, not implemented or proved in the source checkout.

Generate `Node::new(id: u64, members: Vec<u64>) -> Result<Node, InitError>` together with its implementation-layer contracts and proof. May edit `src/node.rs`, `src/node_state.rs` and `src/contracts.rs`; choose the internal layout, view, representation invariant, helper decomposition and local predicates. The same Rust source is compiled and verified. The current reject-all seed is deliberately incomplete and cannot pass this task. The candidate contains no offline solution; do not retrieve one from outside it.

Accept every nonempty, duplicate-free membership containing the local ID, regardless of input order, including one node and `u64::MAX` IDs. Preserve the member order and values. Reject empty lists, duplicate peers and absent IDs with `InvalidMembership`. Initialize term/positions/incarnation/serial to zero, no vote, empty log, Follower role, deadline 1, no pending write and recovery_required false, matching the existing `fresh_node` fields. This constructor is only for a genuinely fresh identity/storage image; never use it to bypass recovery or reuse incarnation zero. Allocation follows Rust Vec's existing abort behavior; no resource-liveness claim is made. API recommendations remain DRAFT under ADR-01/02/03 for review after delivery.

The protected `raft-spec/src/node_init.rs` expresses `init_result` on observable state and requires success for every valid input. `raft-proofs/node_init.rs` calls the actual constructor and independently requires the resulting Node to satisfy `wf` and that target through its generated view. `raft-proofs/observation.rs` checks actual `observe`/`from_observation` against the same view. These callers cannot be edited during ordinary generation. A true local contract or false invariant does not discharge them. This is initial-state correspondence only, not whole-node/cluster refinement.

Develop implementation, local specification and proof together. Explain how the representation and invariant connect to the independent target; revise them when an approach stalls. No duplicate-scan algorithm or loop invariant is prescribed. Run all eight initialization cases and the local bridge. Distinguish code defects, missing proof facts, mismatched local contracts and solver limits before repairing. The controller additionally reruns completed tasks during continuation.

```sh
python3 crates/raft-verified/check.py build
python3 crates/raft-verified/check.py test --test node_init
python3 crates/raft-verified/check.py node-replay
python3 crates/raft-verified/check.py node-proof
python3 crates/raft-verified/check.py node-bench
```

The controller also typechecks the candidate model; that closes no theorem. It measures initialization only after required correctness checks pass. The replay constructs independent initial states, not a cluster fault experiment. Report results, design/complexity and remaining gaps using the existing agent report. Passing closes only LOCAL-NODE-INIT on this candidate. The next selected step RUNTIME-PERSIST uses this same Node with supplied pending-write observations and independent tests/proof/replay. A combined plan carries this checked constructor forward without requiring a merge first.

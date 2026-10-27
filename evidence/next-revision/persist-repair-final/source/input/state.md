# Node state and initialization

Status: DRAFT. Module: INCOMPLETE; initial-state types and independent checks exist; constructor generation is pending.

## Responsibility and connections

Represent a fixed-membership Raft node throughout its lifetime: follower, candidate and leader roles; durable term, vote and log; volatile commit/delivery positions; election votes, peer replication progress and pending operations. This state is shared by [elections](elections.md), [replication](replication.md) and [commitment](commitment.md). The [runtime](runtime.md) serializes events; [recovery](recovery.md) reconstructs state after a crash.

The shared types are in `crates/raft-verified/src/state.rs`, re-exported by the library. `Node` currently represents identity, membership, hard state, role, commit/delivery/application positions, incarnation, serial and deadline. Pending I/O, peer progress and recovery will extend this same type. They are not represented by fictitious constant fields in a claimed complete refinement. The model's `fresh_node`, `initial`, `init` and `well_formed` remain unchanged.

## Module completion

Fresh construction, role-specific fields and recovered construction have explicit bounds and representations. Every handler uses this state; no parallel test-only node exists. Tests cover legal/invalid membership, initial state and index/resource boundaries. Actual proofs establish initialization and representation and supply the invariant required by handlers. PO-INIT, PO-REPRESENTATION, PO-REFINE-INIT and index/token lemmas in [proofs.md](proofs.md) remain open until checked.

## Step: Initialize node state

Stage: READY for generation, not implemented or proved in the source checkout.

Implement `Node::new(id: u64, members: Vec<u64>) -> Result<Node, InitError>` in `crates/raft-verified/src/node.rs`, with its Verus annotations and loop proofs in the same file. The current reject-all seed is deliberately incomplete and cannot pass this task. Do not copy the offline controller fixture as generated work.

Accept every nonempty, duplicate-free membership containing the local ID, regardless of input order, including one node and `u64::MAX` IDs. Preserve the member order and values. Reject empty lists, duplicate peers and absent IDs with `InvalidMembership`. Initialize term/positions/incarnation/serial to zero, no vote, empty log, Follower role and deadline 1, matching the existing `fresh_node` fields. This constructor is only for a genuinely fresh identity/storage image; never use it to bypass recovery or reuse incarnation zero. Allocation follows Rust Vec's existing abort behavior; no resource-liveness claim is made. API recommendations remain DRAFT under ADR-01/02/03 for review after delivery.

The protected `crates/raft-spec/src/node_init.rs` specifies valid membership and `init_result`. It demands success for every valid input, so rejecting all inputs cannot prove the target vacuously. `crates/raft-proofs/node_init.rs` calls the exact constructor source compiled by Cargo; weakening its annotation or strengthening its precondition cannot satisfy the independent caller. This checks only the represented initial fields, not whole-node/cluster refinement or runtime behavior. No semantic model, oracle or checking-policy edit is part of this task.

Develop code and proof together. A simple duplicate scan is sufficient initially; use invariants to relate the scanned prefix to membership validity. Compare test failures and failed proof goals, retain failed approaches in the report, and repair within this file. Check all existing executable regressions as well.

```sh
python3 crates/raft-verified/check.py build
python3 crates/raft-verified/check.py test
python3 crates/raft-verified/check.py node-replay
python3 crates/raft-verified/check.py node-proof
python3 crates/raft-verified/check.py proof
python3 crates/raft-verified/check.py node-bench
```

The controller also typechecks the candidate model; that closes no theorem. It measures initialization only after required correctness checks pass. The replay constructs independent initial states, not a cluster fault experiment. Report results, design/complexity and remaining gaps using the existing agent report. Passing closes only LOCAL-NODE-INIT on this candidate. Next development extends the same Node with pending writes and completion handling in RUNTIME-PERSIST; its currently missing oracles and representation must be supplied before execution.

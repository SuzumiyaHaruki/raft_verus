# Raft requirements

Status: DRAFT. Candidate contract `RAFT-V1`, version `0.1.0`; no human approval or complete Raft proof.

The eventual target is a standalone Rust Raft library with fixed membership and a single-threaded event-driven core: elections, heartbeats, RequestVote/AppendEntries and responses, multi-entry replication, conflict truncation, commitment, ordered delivery and crash recovery under explicit stable-storage semantics. Membership is a parameterized nonempty finite set `M`; quorum `Q` satisfies `Q ⊆ M ∧ 2|Q| > |M|`. Recommend supporting one node. Tests with three or five nodes do not replace a proof for arbitrary finite size.

Development is organized around complete Raft modules, implemented through bounded checked steps inside each module. The repository contains a candidate model, interface types, a real log-suffix reconciliation operation and a Python checking/repair workflow. This operation is a building block for follower AppendEntries; it is not a complete Raft node or handler. No unimplemented `step` reports success. Tests, replay, measurement and Verus checking must use the same executable logic. The roadmap in [roadmap.md](roadmap.md) retains the complete safety and conditional-liveness goal.

Excluded: dynamic membership, joint consensus, snapshots, log compaction, ReadIndex, lease reads, leader transfer, pre-vote, performance pipelining/optimized batching and database service. Multi-entry AppendEntries is basic functionality, not parallel pipelining. Without compaction, bounded space under unlimited execution is not promised.

Non-Byzantine crash recovery, message loss, finite or infinite delay, duplication, reordering and partitions are allowed. Safety does not require faults to stop; liveness has separate eventual-stability conditions. Forged messages, false durability reports and arbitrary disk corruption are outside the safety environment.

The model uses natural numbers and unbounded sequences. Recommend checked `u64` terms, indices and operation IDs with explicit resource errors; incarnation freshness requires a durable counter under the storage contract. Safe stopping with finite representations does not imply liveness. Prove the finite resources needed for each target request or restrict the theorem to execution prefixes that do not exhaust representation/capacity. Fixed-capacity v1 makes no unconditional infinite-input liveness promise.

State library delivery consistency, host application safety and full client linearizability separately. Delivery may replay after recovery; external effects are not exactly-once. See the [I/O contract](io.md) and [open decisions](../review/OPEN_DECISIONS.md).

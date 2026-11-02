Implemented one authoritative Core for receiver, campaign, matching persistence completion, and real-reply collection. The original host calls these functions directly. Initial targeted RPC test is running. Cross-node initialization imports and calls the fixed actual Network constructor now, before local proof completion. Full source preservation remains OPEN.

Current obstacle: verifying private representation and history invariants, then cross-node provenance preservation. No fixed-input objection identified. No prior candidate imported.

<!-- obligation -->
```ini
[obligation]
id = actual-vote-initialization
statement = Actual Core and Network initialization establish boot and empty source obligations, with receiver calls through the same Core.
goal = execution_link
kind = initialization
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::complete
entry = madraft/proof/vote_initial.rs
parent =
dependencies =
premises = Network source preservation and async host fidelity remain open.
tests = raft::raft::vote_tests::rpc_rules_and_file
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/vote_initial.rs | Establish constructor and fixed network initialization. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect | madraft/proof/vote_initial.rs | Arbitrary message preservation remains open. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/vote_initial.rs | Host completion and RPC association are public premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | Two retained-image windows; arbitrary crashes remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot | madraft/proof/implementation.rs | All snapshot obligations remain OPEN. |
<!-- /connections -->

Checks: task-specific seed-1 raft::raft::vote_tests passed 29/29 on the executable implementation. Initial unscoped checks failed because their snapshot omitted task inputs; scoped feedback reached real code. Verus diagnosed enum-derived equality and mutation of invariant-bearing fields; equality is now executable pattern matching and voter insertion moves the Vec through verified mem::swap. Final raw proof diagnostics are in verus-initial.log; no formal PASS is claimed. Failed approach: borrowing voters directly into Vec::push is rejected by the type-invariant unwind rule. Next obstacle: establish the private invariant at all mutator returns, then strengthen network_inv with pending/send/reply correspondence and prove network_step rather than assume it. General source preservation is explicitly unfinished; global safety and liveness remain OPEN.

Final concrete Verus blocker: wf is required at individual field updates as well as returns; constructors and intermediate updates do not yet discharge it. The direct source-linked entry therefore reports verification errors, not a checked initialization result. Repair requires coherent whole-Core reconstruction or an invariant-friendly representation, and proof of finite-set cardinality/domain facts. No exception, assume, admit or skipped consensus body was introduced.

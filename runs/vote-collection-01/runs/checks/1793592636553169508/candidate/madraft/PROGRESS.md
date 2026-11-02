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

Implemented the shared receiver, campaign, completion, collection and strict-majority promotion bodies; no prior candidate imported. First check pending. Historical network induction and host premises remain OPEN.

<!-- obligation -->
```ini
[obligation]
id = core-initialization-and-handlers
statement = Actual constructor and handlers satisfy the fixed state and output contracts.
goal = election_safety
kind = initialization
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/core_connection.rs
parent =
dependencies =
premises = Host serialization and faithful I/O remain unverified; historical induction remains open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/core_connection.rs | Check actual initialization contracts. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/core_connection.rs | Network invariant preservation remains open. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/core_connection.rs | Faithful host completion and forwarding unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | Four local recovery compositions remain open. |
| snapshots | madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Snapshot and complete Raft behavior remain open. |
<!-- /connections -->

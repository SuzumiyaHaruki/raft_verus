Implemented receiver, campaign, completion, collection, executable distinct votes and strict-majority promotion in one Core. Seed-1 selected election tests passed in the first obligation run. The latest core_connection proof passed Verus with all imported Core bodies and the strengthened Choice invariant (runs/checks/1793600706590672605/proof.log). No earlier candidate imported.

<!-- obligation -->
```ini
[obligation]
id = historical-network-provenance
statement = Network initialization and each actual entry preserve vote provenance and historical election witnesses.
goal = election_safety
kind = step
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/network_connection.rs
parent =
dependencies = madraft/proof/core_connection.rs
premises = Host serialization and faithful I/O remain unverified; historical induction remains open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/core_connection.rs | Constructor and all Core bodies checked; deployment mapping unverified. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/core_connection.rs | Network invariant preservation remains open. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/core_connection.rs | Faithful host completion and forwarding unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | Four local recovery compositions remain open. |
| snapshots | madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Snapshot and complete Raft behavior remain open. |
<!-- /connections -->

Failed approaches: derived enum equality is unsupported by Verus; replaced it with pattern checks. Mutable field borrows conflicted with type invariants; stage voter vectors locally. The first recovery-history bound incorrectly excluded a first same-term choice after recovering an empty vote; repaired it to constrain only an existing recovered vote. Next obstacle: construct and preserve the network invariant linking actual delivered replies and append-only election frames to historical Choice witnesses and majority intersection. The supplied false network placeholder remains an explicit failed proof, not a safety premise.

Round 2 resumes after independent checks passed all 38 seed-1 tests, all local contracts, promotion, mixed history and four local recovery windows. The first missing dependency is network initialization/provenance. Host premises: membership/address mapping, original RPC association, serialized operations, truthful submitted I/O completion/readback, retained simulated files, encoding, cancellation and forwarding. No majority, vote uniqueness or election safety is a host premise.

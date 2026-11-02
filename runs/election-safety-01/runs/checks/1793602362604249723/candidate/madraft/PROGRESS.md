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
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/network_connection.rs | Network initialization and preservation checked; async host contracts unverified. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/core_connection.rs | Faithful host completion and forwarding unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | All four local windows passed independent round-1 checks; arbitrary cluster restart remains open. |
| snapshots | madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Snapshot and complete Raft behavior remain open. |
<!-- /connections -->

Round 2 replaced the false network placeholder with an inductive invariant over local state, host Pending correspondence, actual sends and replies, delivered vote sources, Choice witnesses and append-only election frames. Initialization, local transition lemmas, state/pending preservation, message preservation, vote-source preservation, frame preservation and historical majority-intersection conclusion all pass together in madraft/proof/network_connection.rs (runs/checks/1793601499869902204/proof.log). This imports the actual Core and fixed network executor; no elections or majorities filter enabled steps. Historical frames retain their own winning sets and Choice witnesses across demotion and tally clearing. The fixed vocabulary bridge retains the original Event::Elect target for an arbitrary fixed group.

Failed approach: one monolithic preservation proof timed out; split it by state/pending, messages, votes and frames. The first split exposed missing explicit instantiation of old sent/reply evidence; adding those facts discharged the complete unit. Earlier enum/type-invariant and recovery-bound repairs remain in the same candidate. Independent round-1 results passed all 38 seed-1 tests, local rules, promotion, mixed history and all four recovery windows. Final task-wide results, remaining seeds and semantic host coverage still require the stopped-candidate checker.

Host premises: fixed membership/address mapping, original genuine RPC association, serialized Core operations, truthful submitted I/O completion/readback, retained simulated files, encoding, cancellation and forwarding; physical durability and interrupted/failed-write recovery remain OPEN. No majority, vote uniqueness or Election Safety is assumed by the host. Complete Raft, snapshots, other four global properties and liveness remain OPEN. Candidate reports and sources are frozen before final.

Receiver, campaign and real-reply collection share one Core. Round 2 repaired the private invariant by moving a Data value through each complete transition and restoring Core once coherent, proved finite member-set cardinality, and established pending-send consistency including nonhalted state. Protocol rules and the supplied async host remain unchanged. The fixed Network now has an initialized and preserved source invariant covering pending proposals, actual successful sends, actual receive/complete replies and retained delivery indices; effective voters have explicit earlier self-save or remote-reply witnesses.

Checks: round2-proof3.log verifies the actual Core, receiver caller and Network initialization (54 verified, zero errors). round2-safety.log passed every local group and all four recovery callers before source repair. round2-source4.log verifies the fixed arbitrary finite-message provenance caller (63 verified, zero errors). The refactored seed-1 vote suite passed 29/29. The complete final run passed all 22 required checks, including all five seeds, general provenance, both nonempty witnesses and all four recovery callers. Its aggregate was INCOMPLETE solely because this progress note and auxiliary diagnostic files changed during the run; the clean final repeat now returned VOTE_COLLECTION_CHECKS_PASSED (runs/checks/1793593391348242067/results.json), with no concurrent changes. The standalone source composition independently passed 58 verified, zero errors. No fixed-input objection identified.

Failed approach: field-by-field Core updates violated the type invariant at intermediate states. The replacement stages the same transition privately; a recursive finite-set bound discharges cardinality, and a pending-implies-nonhalted fact discharges self-save completion. An invariant containing only the source conclusion lacked pending/send/reply origin facts; those facts now support preservation, including higher old replies, failed saves, duplicate delivery and noncurrent terms. Source evidence is retained when nodes change term.

<!-- obligation -->
```ini
[obligation]
id = actual-vote-source-step
statement = The actual fixed Network constructor and enabled execute calls establish and preserve finite-message provenance for effective voters.
goal = execution_link
kind = step
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect
entry = madraft/proof/vote_source_entry.rs
parent =
dependencies = madraft/proof/vote_initial.rs
premises = Faithful async host RPC association, I/O, readback, serial wiring, encoding, cancellation and forwarding remain unverified; cluster restart and global Raft goals are open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/vote_source_entry.rs | Actual fixed Network constructor calls Core::new; deployment mapping remains trusted. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::collect | madraft/proof/vote_source_entry.rs | Fixed Network execute preserves provenance; async serial wiring is unverified. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/vote_source_entry.rs | Matching completion authorizes exact recorded output; external I/O is a host premise. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_initial.rs | Four fixed callers prove local retained-image windows; arbitrary cluster crashes remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot | madraft/proof/implementation.rs | Snapshot creation, installation, decoding and recovery remain OPEN. |
<!-- /connections -->

Boundary: the finite-message proof has no node restart. Local recovery covers only confirmed and written-but-uncompleted receive/self-save records with faithful retained-image readback. It does not prove external I/O or real-disk durability. Majority reasoning, leader promotion, all five global safety targets, replication, snapshots, application correctness and conditional liveness remain OPEN. Machine verification still requires semantic review of the actual host boundary.

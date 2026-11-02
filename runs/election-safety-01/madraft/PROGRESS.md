Round 4 final completed successfully: ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED in runs/checks/1793603270321300948/results.json. Build, inventory, source boundary, all 38 tests at each of seeds 1–5 (190 executions), and all 18 fixed proof callers pass. This includes actual initialization, local rules and mixed history, genuine vote-to-Choice provenance, promotion, historical-election basis/uniqueness under the original target projection, nonempty witnesses and all four local recovery windows. No concurrent candidate changes, changed fixed standards or evidence mismatches were reported. This is scoped machine evidence pending semantic review; it establishes neither host contracts nor complete Raft. Only report updates and the CLI exit-code objection were written after final completed; proof and executable sources remain unchanged.

<!-- obligation -->
```ini
[obligation]
id = historical-network-provenance
statement = Discharged: fixed finite-prefix provenance and historical election callers pass within their existing verification limits.
goal = election_safety
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/network_connection.rs
parent =
dependencies = madraft/proof/core_connection.rs, madraft/proof/vote_lemmas.rs
premises = All fixed final callers pass; host serialization and faithful I/O remain unverified and semantic review remains pending.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/core_connection.rs | Actual constructor checked; deployment mapping unverified. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/network_connection.rs | Fixed actual network initialization, preservation and historical conclusion callers all pass; host paths unverified. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/network_connection.rs | Actual sent/reply/completion/delivery provenance proved; host completion and forwarding unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | All four local windows pass in this final; arbitrary cluster restart OPEN. |
| snapshots | madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Snapshot and complete Raft behavior OPEN; implementation entry absent. |
<!-- /connections -->

One shared executable Core implements receive, campaign, completion, distinct vote collection and strict-majority promotion. Its history invariant establishes same-node/same-term Choice uniqueness and the recovery-origin bound. Network preservation separately checks local state/Pending, actual sends and replies, delivered vote sources and immutable election frames. Each historical election retains its winning set and Choice witnesses after demotion, failure, tally invalidation and subsequent terms. Majority intersection and actual Choice uniqueness yield historical uniqueness, and the fixed observer bridge uses the original targets::Event::Elect vocabulary for an arbitrary fixed group. No desired property filters enabled executions.

Round 4 isolates structure_step, messages_step, votes_step, frames_step and network_conclusion with Verus spinoff_prover and exports the already-proved node-count preservation from network_step to avoid expanding step_after for loop bounds. The diagnostic mirror of the unmodified fixed provenance caller passes with all imported bodies and --no-cheating: runs/checks/1793603165846652701/proof.log, 79 verified, 0 errors. The mirror enables exactly the provenance configuration; it is supplementary evidence and does not replace fixed acceptance. Previous opaque-invariant-only optimization failed to resolve independent timeouts. Protocol behavior and all fixed inputs are unchanged. Final checks completed with sources and reports frozen; the resulting task-wide report is authoritative. The previous first dependency is repaired. The CLI nevertheless returned exit status 1 despite its success label; madraft/OBJECTION.md gives a minimal reproduction and the unchanged fixed-code cause.

Host premises remain UNVERIFIED: membership/address mapping, genuine original RPC association, serialized Core operations, truthful submitted I/O completion/readback, file retention, encoding, cancellation and forwarding. Exact host paths include RaftHandle::vote_call, start_election, send_vote_request, deliver_vote_result and request_vote, and vote_host::save/load. None assumes majorities, vote uniqueness or Election Safety. The network theorem excludes restart; four local recovery windows are separate. Physical durability, interrupted/failed-write recovery, arbitrary repeated cluster restart, real log-summary correspondence, complete Raft Election Safety, the other four global properties, snapshots, full MadRaft and conditional liveness remain OPEN. Rust/Verus/vstd/SMT are trusted. Cost/token accounting remains unknown.

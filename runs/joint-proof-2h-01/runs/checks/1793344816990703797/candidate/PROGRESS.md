# Progress

Round 1 selects the real fresh-state constructor and its first RequestVote call. `RaftHandle::new` now validates membership before indexing peers and calls `State::new`; `Raft::request_vote` calls the shared verified core. The core rejects invalid candidate IDs and malformed last-term claims, advances terms monotonically, checks log freshness, and prevents changing an existing vote within a term. No term arithmetic is needed in this segment. Initial observations cover the actual empty log only; extending that observation to nonempty logs is OPEN.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The real State::new constructor establishes well-formed empty-log state and enables RequestVote with exact term/vote/reply correspondence.
goal = execution_link
kind = initialization
functions = madraft/src/raft/state.rs::new, madraft/src/raft/state.rs::request_vote, madraft/src/raft/raft.rs::new, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/initialization.rs
parent = madraft/proof/vote_correspondence.rs
dependencies =
premises = Async constructor and RPC wrapper composition, durable vote release, recovery, message provenance, and cluster induction remain open. Observation currently applies only to the implemented empty-log segment.
tests = raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/state.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/initialization.rs | Async constructor, restore, deployment group identity remain open. |
| transitions | madraft/src/raft/state.rs::request_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_correspondence.rs | Actual wrapper preconditions and send provenance remain open. |
| outputs | madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_correspondence.rs | Core reply matches request; persistence and actual reply release remain open. |
| recovery | madraft/src/raft/raft.rs::restore | madraft/proof/recovery.rs | No recovery implementation/proof yet. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths and decoding remain open. |
<!-- /connections -->

Checks: first obligation check pending. Failed approach: none yet; inherited election test fails because no timer elects a leader. Next obstacle: check core contracts, then establish actual storage submission/completion and serialized reply release before adding elections. All five global safety targets and complete execution coverage remain OPEN.

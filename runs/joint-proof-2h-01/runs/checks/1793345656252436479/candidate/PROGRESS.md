# Progress

Round 1 selects the real fresh-state constructor and its first RequestVote call. `RaftHandle::new` now validates membership before indexing peers and calls `State::new`; `Raft::request_vote` calls the shared verified core. The core rejects invalid candidate IDs and malformed last-term claims, advances terms monotonically, checks log freshness, and prevents changing an existing vote within a term. No term arithmetic is needed in this segment. Initial observations cover the actual empty log only; extending that observation to nonempty logs is OPEN.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The real RPC caller uses a checked phase machine to initialize state, bind each RequestVote to an immutable storage operation, and release only the matching pending reply.
goal = execution_link
kind = caller
functions = madraft/src/raft/protocol.rs::new, madraft/src/raft/protocol.rs::begin_vote, madraft/src/raft/protocol.rs::complete_vote, madraft/src/raft/raft.rs::new, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/vote_operation.rs
parent = madraft/proof/vote_operation_history.rs
dependencies = madraft/proof/initialization.rs
premises = Async constructor and RPC wrapper composition, durable vote release, recovery, message provenance, and cluster induction remain open. Observation currently applies only to the implemented empty-log segment.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/state.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/initialization.rs | Async constructor, restore, deployment group identity remain open. |
| transitions | madraft/src/raft/state.rs::request_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_correspondence.rs | Actual wrapper preconditions and send provenance remain open. |
| outputs | madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_correspondence.rs | Core reply matches request; persistence and actual reply release remain open. |
| recovery | madraft/src/raft/raft.rs::restore | madraft/proof/recovery.rs | Validated vote-image recovery and conditional same-term rejection; durable-history linkage remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths and decoding remain open. |
<!-- /connections -->

Checks: first obligation check passed Cargo build and both nominated Verus units (11 verified in initialization); initial_election_2a fails as expected with no leader. The second obligation check passed build, both proof units, and vote_persists_before_reply_and_restores; initial_election_2a still fails with no leader. The recovery unit separately passed. The third obligation check passed build, the actual RPC vote/storage/recovery test, initialization (11 verified) and the vote-history parent (15 verified); initial_election_2a still fails. Combined progress.rs also passed. Evidence: runs/checks/1793345105383611124/results.json and runs/checks/1793345157515802882/results.json. These local results do not establish any global target. Failed approach: the inherited truncate/write skeleton cannot support available recovery across its gap; this vote-only increment rejects interrupted images and leaves recoverable storage generations as future work. The inherited election test continues to fail because no timer elects a leader. Next obstacle: prove storage submission/completion and actual async reply-release composition, then add checked election initiation and vote collection. All five global safety targets and complete execution coverage remain OPEN.

Storage increment: the real RequestVote wrapper holds an async operation gate across mutation, state-file write and reply release. Vote-only images include deployment member identity and term/vote. Truncated or undecodable state halts recovery rather than clearing votes. This is a conservative incomplete recovery strategy: it sacrifices restart availability in the create/write gap; log and snapshot persistence and the formal async-storage correspondence remain OPEN. No multi-file atomicity is assumed.

Handoff: no global implementation adapter or nonempty cluster witness is supplied yet. The local VoteHistory quantifies over every RequestVote input and proves stable same-term votes across repeated calls; it excludes crashes and non-vote transitions and is explicitly not the final runtime relation. Next required connection is the actual async request operation: identify its submitted image, prove completion matches that image, connect the released reply to the originating delivered request, and carry the durable vote relation through restart. Then add checked election timeout and reply-collection paths. No rollback or earlier candidate was used.

Round 2: preserving the checked State and local history results while replacing the real caller state with a private Machine. begin_vote issues an owned immutable WriteVote and retains request/reply context; the async host writes that operation image and calls complete_vote only after successful I/O. The checked completion matches both operation number and full image before releasing the retained reply. Failure retains the pending operation and releases nothing. Recovery version/deployment checks move into verified Machine::recover. The filesystem completion guarantee and async call ordering still require proof; an operation object alone is not durable evidence. First check of this increment pending.

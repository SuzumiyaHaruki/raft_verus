# Progress

Round 6 retains the checked voting, storage and caller-context results and adds the actual sender required by request provenance. The real timer calls checked Runtime::begin_election, which derives the next term with overflow rejection and an exact self-vote request. The existing persist/finish_rpc path completes that self-vote before checked campaign enters candidate state. The async caller then submits that exact request to each other peer. The journal distinguishes internally authorized SelfVote from received Request and records candidate entry. Quorum collection, timeout reset and leader heartbeats remain unfinished; incoming election replies currently have no state effect.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = A real election timeout initiates a checked self-vote, persists it through the existing caller, records candidate entry, and sends the exact authorized RequestVote payload to peers.
goal = execution_link
kind = step
functions = madraft/src/raft/state.rs::election_request, madraft/src/raft/execution.rs::begin_election, madraft/src/raft/execution.rs::campaign, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/implementation.rs
parent = madraft/proof/execution_connection.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/durable_vote_reply.rs
premises = Checked self-vote/candidate histories are per-incarnation. Async timer/send composition, exact filesystem effects, network envelopes and prior sends, deployment identity and cross-crash history remain open. Reply collection and heartbeat behavior remain unfinished; global step/provenance/theorem exports remain missing.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/implementation.rs | Actual journal initialization and initial-registry projection checked; deployment mapping and full cluster constructor remain open. |
| transitions | madraft/src/raft/execution.rs::begin_election, madraft/src/raft/execution.rs::campaign, madraft/src/raft/execution.rs::begin_rpc, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Internal self-vote authorization and candidate entry checked; genuine delivery and async caller adequacy remain open. |
| outputs | madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/execution.rs::release_vote, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Actual election sends follow completed self-vote in code and regression; host composition, exact send history and filesystem result correspondence remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Exact image/follower/empty pending state establishes new journal; prior-incarnation linkage remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 5 independent replay retained all selected local and parent results; the fixed execution caller still lacks global step/provenance functions. Both round-6 obligation attempts passed build, initialization and durable-vote dependencies, implementation entry and execution-history parent (runs/checks/1793347723281686692/results.json and runs/checks/1793347888249710521/results.json). The latter passed all five vote_tests, including election_sends_persisted_self_vote, which observes the actual outgoing request and disk image at peer delivery. initial_election_2a still fails with no leader because quorum collection and heartbeats remain unfinished. The parent now includes actual_candidate_entry: an accepted campaign has no pending operation, has voted for itself, preserves its persistent image and enters Candidate at the matching term. These results are partial execution correspondence, not any global safety theorem.

Failed approaches retained: truncate/write recovery stops on incomplete images and still lacks recoverable generations; cross-module reveal was repaired with an owner lemma; executable enum equality was repaired with pattern matching; concurrent notes mutation required a stable check rerun; ghost pattern ownership required borrowing; recovery required exported follower-role/pending-state facts. Round 5 required an explicit state/view correspondence at the real release caller. No rollback, assumptions, skipped consensus bodies, fabricated results or earlier candidates were used.

Scope/next dependency: connect actual outgoing request envelopes and reply continuations to the cluster history, retaining source peer, request context, old genuine replies and outstanding calls. Implement checked distinct-voter collection and term-sensitive leader entry, then heartbeat/reset behavior. The real sender now exists; received replies currently have no state effect. Async storage composition and cross-crash history remain open. The global adapter still lacks step/receive/emit and runtime proof functions; no permissive or empty relation was substituted. All five fixed goals, complete labs, snapshots, conditional liveness and application correctness remain OPEN. No rollback was requested.

Combined progress proof passed after the round-6 source and parent changes (runs/checks/1793347984103075673/results.json). This includes the actual shared-source candidate initiation and the retained local connections; it does not export the missing aggregate cluster theorem.

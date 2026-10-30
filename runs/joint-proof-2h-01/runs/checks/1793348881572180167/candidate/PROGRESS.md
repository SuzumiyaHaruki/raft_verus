# Progress

Round 6 retains the checked voting, storage and caller-context results and adds the actual sender required by request provenance. The real timer calls checked Runtime::begin_election, which derives the next term with overflow rejection and an exact self-vote request. The existing persist/finish_rpc path completes that self-vote before checked campaign enters candidate state. The async caller then submits that exact request to each other peer. The journal distinguishes internally authorized SelfVote from received Request and records candidate entry. Quorum collection, timeout reset and leader heartbeats remain unfinished; incoming election replies currently have no state effect.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The real RPC continuation feeds its checked distinct-grant tally into a term/member/candidate-checked leader transition recorded in the actual runtime history.
goal = execution_link
kind = step
functions = madraft/src/raft/election.rs::complete_call, madraft/src/raft/protocol.rs::elect, madraft/src/raft/execution.rs::elect, madraft/src/raft/raft.rs::send_vote_request
entry = madraft/proof/implementation.rs
parent = madraft/proof/execution_connection.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/durable_vote_reply.rs
premises = Checked histories cover per-incarnation voting and per-election collectors. Async sender/receiver composition, network send provenance, exact filesystem effects, deployment identity and cross-crash history remain open. Leader authorization, higher-term reply persistence and heartbeats remain unfinished. The last Cargo check timed out and needs a longer build/test check.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/implementation.rs | Actual journal initialization and initial-registry projection checked; deployment mapping and full cluster constructor remain open. |
| transitions | madraft/src/raft/execution.rs::begin_election, madraft/src/raft/execution.rs::campaign, madraft/src/raft/execution.rs::begin_rpc, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Internal self-vote authorization and candidate entry checked; genuine delivery and async caller adequacy remain open. |
| outputs | madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/execution.rs::release_vote, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/election.rs::outbound, madraft/src/raft/election.rs::complete_call, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Exact request/peer context and distinct-grant collector checked; async send provenance, cluster history and filesystem correspondence remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Exact image/follower/empty pending state establishes new journal; prior-incarnation linkage remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 5 independent replay retained all selected local and parent results; the fixed execution caller still lacks global step/provenance functions. Both round-6 obligation attempts passed build, initialization and durable-vote dependencies, implementation entry and execution-history parent (runs/checks/1793347723281686692/results.json and runs/checks/1793347888249710521/results.json). The latter passed all five vote_tests, including election_sends_persisted_self_vote, which observes the actual outgoing request and disk image at peer delivery. initial_election_2a still fails with no leader because quorum collection and heartbeats remain unfinished. The parent now includes actual_candidate_entry: an accepted campaign has no pending operation, has voted for itself, preserves its persistent image and enters Candidate at the matching term. These results are partial execution correspondence, not any global safety theorem.

Failed approaches retained: truncate/write recovery stops on incomplete images and still lacks recoverable generations; cross-module reveal was repaired with an owner lemma; executable enum equality was repaired with pattern matching; concurrent notes mutation required a stable check rerun; ghost pattern ownership required borrowing; recovery required exported follower-role/pending-state facts. Round 5 required an explicit state/view correspondence at the real release caller. No rollback, assumptions, skipped consensus bodies, fabricated results or earlier candidates were used.

Scope/next dependency: connect actual outgoing request envelopes and reply continuations to the cluster history, retaining source peer, request context, old genuine replies and outstanding calls. Implement checked distinct-voter collection and term-sensitive leader entry, then heartbeat/reset behavior. The real sender now exists; received replies currently have no state effect. Async storage composition and cross-crash history remain open. The global adapter still lacks step/receive/emit and runtime proof functions; no permissive or empty relation was substituted. All five fixed goals, complete labs, snapshots, conditional liveness and application correctness remain OPEN. No rollback was requested.

Combined progress proof passed after the round-6 source and parent changes (runs/checks/1793347984103075673/results.json). This includes the actual shared-source candidate initiation and the retained local connections; it does not export the missing aggregate cluster theorem.

Final round-6 obligation replay on the saved nomination passed build, all five vote regressions, both dependencies, implementation entry and parent (runs/checks/1793348003162867199/results.json); initial_election_2a remains failing. The stopped candidate retains these partial results and the next reply-collection/provenance dependency.

Round 7: the actual outgoing RPC future now carries a private VoteCall containing exact request and peer identity; completion feeds its real Result (including errors) into a per-election checked Tally. The tally records all matching-context completions and counts each in-range granting peer once at the requested term. Contexts from another request cannot mutate it. The parent invokes its evidence theorem to establish distinct-voter membership and matching reply origins. These are per-election histories; actual network provenance and cluster stitching remain open. The previous round’s core results are retained.

Round-7 diagnostics: renamed TallyView to avoid shadowing the Verus View trait, repaired a public contract exposing private fields, supplied loop termination, and separated read-only duplicate lookup from mutation. Reply-origin preservation initially failed; explicit witnesses over the exact final TallyView repaired it without weakening the invariant (runs/checks/1793348561623747187/results.json PASS). One 45-second obligation run timed out in Cargo; later checking must re-establish the build. New regression covers duplicate, old-term, rejected, error, invalid-peer and wrong-context replies. Final obligation replay passed both prerequisites, implementation entry and parent; Cargo timed out, so the new regression remains unrun (runs/checks/1793348609530078477/results.json). Next: connect the checked tally to current-term candidate/leader authorization and implement higher-term reply persistence and heartbeat/reset behavior. No leader transition or global safety proof is claimed.

Stopped-candidate check: runs/checks/1793348609530078477/results.json passed both dependencies and the actual implementation/parent proofs. Build hit the short per-stage time allowance; inventory and tests were consequently not run. First next action is a longer build and vote_tests check, then leader authorization and higher-term reply handling. Earlier regression passes apply to earlier checked code, not the new unrun test.

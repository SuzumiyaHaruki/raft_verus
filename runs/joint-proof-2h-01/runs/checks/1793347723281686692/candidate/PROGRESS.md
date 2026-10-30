# Progress

Round 5 connects the real async request caller to the checked history. Runtime::begin_rpc calls the journaled begin_vote and returns an owned write operation plus a private ReplyContext containing the exact request, operation number, image and ghost pre-state. The real RPC future carries that context across persist and calls Runtime::finish_rpc. The checked finish method compares both the receipt and every request field against the pending request before releasing a reply; mismatch leaves the pending vote and journal unchanged. This closes a specific caller ambiguity: two different requests can produce the same image at the same operation number. Existing constructor/history/storage/recovery results are retained.

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
premises = Checked caller context and histories are per-incarnation. Actual filesystem effects, network envelopes and prior sends, deployment group identity, and cross-crash history remain open. Global adapter types, projection and initial-registry relation exist; global steps and provenance/theorem exports remain missing.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/implementation.rs | Actual journal initialization and initial-registry projection checked; deployment mapping and full cluster constructor remain open. |
| transitions | madraft/src/raft/execution.rs::begin_rpc, madraft/src/raft/execution.rs::begin_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Exact caller request/submission is recorded; genuine delivery and async caller adequacy remain open. |
| outputs | madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/execution.rs::release_vote, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Receipt and full pending request must match caller context; actual filesystem result and RPC send provenance remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Exact image/follower/empty pending state establishes new journal; prior-incarnation linkage remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 4 independent replay preserved connection-unit and regression passes, while all global callers reported missing adapter types. Round 5 repaired State/Input/Output/RuntimeState/Message data shapes and checked the fixed execution_link caller; its diagnostics advanced to missing global initial/step/receive/emit and runtime relation functions (runs/checks/1793347147399334297/execution_link.log). Actual initial-registry definitions and projection were subsequently constructed and verified. The first caller-context proof failed because release_vote did not expose core-state/view equality; invoking the existing checked view lemma discharged that concrete postcondition. The final obligation check passed build, four vote regressions (including identical-image/different-request context rejection), both dependencies, implementation entry and parent (runs/checks/1793347366179672458/results.json). initial_election_2a still fails with no leader. Combined progress proof passed (runs/checks/1793347431237882695/results.json). The current fixed execution_link caller still fails on missing step/receive/emit/runtime_step and remaining runtime proof functions, after type/initialization/projection errors were repaired (runs/checks/1793347431234620305/execution_link.log).

Failed approaches retained: truncate/write recovery stops on incomplete images and still lacks recoverable generations; cross-module reveal was repaired with an owner lemma; executable enum equality was repaired with pattern matching; concurrent notes mutation required a stable check rerun; ghost pattern ownership required borrowing; recovery required exported follower-role/pending-state facts. Round 5 required an explicit state/view correspondence at the real release caller. No rollback, assumptions, skipped consensus bodies, fabricated results or earlier candidates were used.

Scope/next dependency: map types and initial projection are interface preparation, not a complete cluster semantics. Local executable journals are real, but global steps still need actual message envelopes, submitted storage callbacks and cross-incarnation stitching. Request delivery provenance also needs the missing real candidate/send_vote_request path; checked election initiation and its actual sends are a necessary next dependency, as reflected by the retained no-leader diagnostic. All five global targets, full labs, snapshots, liveness and application correctness remain OPEN. No absent global step was replaced with a permissive or empty relation.

Round 6: establishing the actual sender needed by request-delivery provenance. The timer invokes checked begin_election, persists its checked self-vote through the existing receipt/context path, enters candidate state through checked campaign, then emits the exact returned request to real peers. The local journal distinguishes internal SelfVote from received Request and records candidate entry. Replies are received but quorum collection remains unimplemented; no leader is claimed. First obligation check pending.

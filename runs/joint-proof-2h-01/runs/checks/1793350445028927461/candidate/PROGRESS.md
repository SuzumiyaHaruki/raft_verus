# Progress

Round 9 retains checked voting, collection, leader-entry and term-storage behavior and adds exact shared-source RPC descriptors. The real sender obtains payload/destination from RequestSend::from_call; the real reply closure returns ResponseSend::body. These functions preserve every supplied value, including malformed requests and erroneous replies. The partial cluster adapter now distinguishes local transitions, submission, host emission and delivery. Pending submitted envelopes and consumed emission slots are retained; submission itself emits nothing. The global runtime and crash adapter remains absent.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The actual RPC receiver retains the transport source, validates its candidate identity in checked code, and records the same delivered request in the running vote handler and response context.
goal = execution_link
kind = step
functions = madraft/src/raft/wire.rs::new, madraft/src/raft/execution.rs::begin_received, madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/raft.rs::start_rpc_server
entry = madraft/proof/adapter.rs
parent = madraft/proof/execution_connection.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/durable_vote_reply.rs
premises = Exact executable descriptor extraction and partial adapter segments are checked. Actual host emission linearization, send and delivery linkage to local handlers, deployment group mapping, storage callbacks and cross-crash history remain open. runtime_step/runtime_invariant and projection theorem exports remain missing; no full cluster coverage is claimed.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/adapter.rs | Local journal and initial registry projection checked; full deployment mapping and cluster constructor remain open. |
| transitions | madraft/src/raft/protocol.rs::elect, madraft/src/raft/execution.rs::elect, madraft/src/raft/execution.rs::prepare_term, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Actual local histories checked; delivered-envelope linkage, async caller and crash histories remain open. |
| outputs | madraft/src/raft/wire.rs::from_call, madraft/src/raft/wire.rs::args, madraft/src/raft/wire.rs::new, madraft/src/raft/wire.rs::body, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::start_rpc_server | madraft/proof/execution_connection.rs | Exact request/response descriptors and submission extraction checked; actual host emission linearization and global source provenance remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/adapter.rs | Validated image establishes follower/empty pending state; prior incarnations and interrupted-write availability remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | Creation, installation, compaction and application decoder correspondence remain open. |
<!-- /connections -->

Checks: round-8 independent replay retained build, vote regressions and all three seed-1 2A passes; other Raft tests still fail at unfinished start agreement. Round-9 initial descriptor attempt passed selected checks (runs/checks/1793349609469171555/results.json). Review identified a semantic gap: submission had been counted as emission. The revised adapter separates these events and ties each emission to one unconsumed submitted slot; the revised check passed (runs/checks/1793349729237249403/results.json). The fixed execution caller now accepts step/receive/emit signatures and stops on missing runtime_step/runtime_invariant/projection functions (runs/checks/1793349735738900075/execution_link.log).

Final round-9 obligation passed build, nine vote-path regressions, initial_election_2a, both dependencies, adapter entry and parent (runs/checks/1793349799582373771/results.json). Combined progress proof passed (runs/checks/1793349806384436351/results.json). The new regression preserves malformed request metadata and erroneous reply fields unchanged; the parent explicitly invokes the descriptor submission lemma. Round-8 _2a tests passed seeds 1–5, but full labs and all-family/all-seed final acceptance remain unrun and incomplete. No global theorem pass is claimed.

Failed approaches retained: truncate/write recovery stops on incomplete images and lacks recoverable generations; closed-spec visibility required an owner lemma; executable enum equality required pattern matching; a concurrent notes edit required a stable rerun; ghost ownership required borrowing; recovery and RPC callers required explicit postconditions. Collector proofs required renaming TallyView, public abstract accessors, loop termination, separate read-only duplicate lookup and exact final-view witnesses. Short Cargo allowances caused local timeouts resolved by independent replay. Round 8 replaced unsupported compound assignment and moved step-down before the term write to stop leadership during pending I/O. No rollback, assumptions, skipped bodies or imported candidate was used. Round 9 corrected submission/emission conflation and switched the proof entry to adapter.rs so standalone and fixed callers share the exact support types. No checkpoint restoration was used.

Next dependency: establish the actual async host-send event from the owned descriptor and pending call, then bind delivered envelopes to the real RequestVote and reply-continuation inputs. The current local-step branch does not yet enforce that binding, and the partial adapter is not claimed to cover all executions. Add runtime_step and its initialized/preserved relation with actual storage callbacks and crash/restart histories before exporting aggregate projection proofs. Heartbeats/stable timeouts, logs, services, snapshots and all five fixed global safety goals remain OPEN. Conditional liveness and application correctness remain OPEN.

Round 10: pinned rpc.rs confirms server responses use the actual transport source, not the claimed candidate. The previous descriptor inferred recipient from candidate and could misdescribe malformed input. The real Raft receiver now retains from/tag using the pinned raw envelope, constructs a checked InboundVote, invokes begin_received, and returns the checked response to the same source/tag. Source-address mapping and host send/delivery semantics remain explicit outer obligations.

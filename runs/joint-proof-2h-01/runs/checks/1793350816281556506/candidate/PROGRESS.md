# Progress

Round 10 retains the checked voting and election behavior and connects actual transport-source validation to the running RequestVote handler. The partial runtime_step now extends concrete local histories and retains transport submissions, emissions and deliveries. Its Received branch requires an earlier matching delivered envelope; aggregate network provenance, crash coverage and preservation remain open.

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
premises = Checked inbound contexts and concrete history extensions are connected conditionally to recorded deliveries. Actual host send/delivery linearization, deployment source mapping, reply continuation provenance, storage callbacks and cross-crash histories remain open. runtime_invariant and initialized/preserved projection theorem exports remain missing; runtime_step covers only the supplied partial segments.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/adapter.rs | Local journal and initial registry projection checked; full deployment mapping and cluster constructor remain open. |
| transitions | madraft/src/raft/execution.rs::begin_received, madraft/src/raft/execution.rs::elect, madraft/src/raft/execution.rs::prepare_term, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::start_rpc_server | madraft/proof/execution_connection.rs | Actual local histories and Received request extension checked; host delivery linkage, async composition and crash histories remain open. |
| outputs | madraft/src/raft/wire.rs::from_call, madraft/src/raft/wire.rs::args, madraft/src/raft/wire.rs::new, madraft/src/raft/wire.rs::body, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::start_rpc_server | madraft/proof/execution_connection.rs | Exact request/response descriptors and submission extraction checked; actual host emission linearization and global source provenance remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/adapter.rs | Validated image establishes follower/empty pending state; prior incarnations and interrupted-write availability remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | Creation, installation, compaction and application decoder correspondence remain open. |
<!-- /connections -->

Checks: round-8 independent replay retained build, vote regressions and all three seed-1 2A passes; other Raft tests still fail at unfinished start agreement. Round-9 initial descriptor attempt passed selected checks (runs/checks/1793349609469171555/results.json). Review identified a semantic gap: submission had been counted as emission. The revised adapter separates these events and ties each emission to one unconsumed submitted slot; the revised check passed (runs/checks/1793349729237249403/results.json). The fixed execution caller now accepts step/receive/emit signatures and stops on missing runtime_step/runtime_invariant/projection functions (runs/checks/1793349735738900075/execution_link.log).

Final round-9 obligation passed build, nine vote-path regressions, initial_election_2a, both dependencies, adapter entry and parent (runs/checks/1793349799582373771/results.json). Combined progress proof passed (runs/checks/1793349806384436351/results.json). The new regression preserves malformed request metadata and erroneous reply fields unchanged; the parent explicitly invokes the descriptor submission lemma. Round-8 _2a tests passed seeds 1–5, but full labs and all-family/all-seed final acceptance remain unrun and incomplete. No global theorem pass is claimed.

Failed approaches retained: truncate/write recovery stops on incomplete images and lacks recoverable generations; closed-spec visibility required an owner lemma; executable enum equality required pattern matching; a concurrent notes edit required a stable rerun; ghost ownership required borrowing; recovery and RPC callers required explicit postconditions. Collector proofs required renaming TallyView, public abstract accessors, loop termination, separate read-only duplicate lookup and exact final-view witnesses. Short Cargo allowances caused local timeouts resolved by independent replay. Round 8 replaced unsupported compound assignment and moved step-down before the term write to stop leadership during pending I/O. No rollback, assumptions, skipped bodies or imported candidate was used. Round 9 corrected submission/emission conflation and switched the proof entry to adapter.rs so standalone and fixed callers share the exact support types. No checkpoint restoration was used.

Next dependency: establish host send/delivery events from the actual pinned endpoint operations and deployment address mapping, and connect reply continuations and storage callbacks to their prior submissions. The Received local branch now requires a matching recorded envelope, but populating that transport history from the async host remains open. Establish the runtime invariant and projection preservation over these histories, then extend through crashes/restarts. Heartbeats/stable timeouts, logs, services, snapshots, all five fixed global safety goals, conditional liveness and application correctness remain OPEN.

Round 10: pinned rpc.rs confirms server responses use the actual transport source, not the claimed candidate. The previous descriptor inferred recipient from candidate and could misdescribe malformed input. The real Raft receiver now retains from/tag using the pinned raw envelope, constructs a checked InboundVote, invokes begin_received, and returns the checked response to the same source/tag. Source-address mapping and host send/delivery semantics remain explicit outer obligations.


Round-10 checks: the first obligation attempt (runs/checks/1793350205604991178/results.json) exposed a struct-literal parsing error in the new contract; parentheses repaired it. The next obligation passed build, nine vote regressions, initial_election_2a, both dependencies, adapter entry and execution parent (runs/checks/1793350302668237779/results.json). The combined progress proof passed (runs/checks/1793350445960210835/results.json). The fixed execution_link caller advanced past runtime_step and now stops on missing runtime_invariant/prove_runtime_initial/prove_runtime_step (runs/checks/1793350445028927461/results.json). These are partial results, not final acceptance. No dedicated spoofed-source regression was added this slice; existing real RPC tests exercise the new receiver.

Handoff: received_journal_enters_registry checks the conditional parent use of an actual Received history extension, while begin_received records the exact inbound context and operation. Source validation is checked code; translating the transport address to deployment member identity and associating host delivery with prior actual sends remain outer obligations. The former response descriptor inferred recipient from claimed candidate; it now receives the actual transport recipient explicitly. No rollback or imported candidate was used.

# Progress

Round 8 connects the actual outgoing RPC continuations to checked leader entry and higher-term handling. Machine::elect consumes the real distinct-grant tally and requires matching current candidate, term, identity, membership, self-vote, empty pending operation and majority. Runtime records the role transition; the parent derives its majority and identity facts. A genuine higher reply immediately steps down, clears the vote and creates an owned term-storage operation/context. The real caller holds its operation gate through write completion and validates the receipt before continuing. Existing initialization, vote, storage, recovery and collector results are retained.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The actual RPC caller uses checked exact-value request/reply descriptors; the partial cluster adapter extracts matching envelopes without filtering erroneous payloads.
goal = execution_link
kind = step
functions = madraft/src/raft/wire.rs::from_call, madraft/src/raft/wire.rs::args, madraft/src/raft/wire.rs::new, madraft/src/raft/wire.rs::body, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::start_rpc_server
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
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/implementation.rs | Local journal and registry projection checked; deployment mapping and cluster initialization remain open. |
| transitions | madraft/src/raft/protocol.rs::elect, madraft/src/raft/execution.rs::elect, madraft/src/raft/execution.rs::prepare_term, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Actual leader and higher-term histories checked; global grant/send provenance and async composition remain open. |
| outputs | madraft/src/raft/election.rs::complete_call, madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/execution.rs::finish_term, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_vote_request | madraft/proof/execution_connection.rs | Exact request/peer, distinct-grant and receipt contracts checked; actual filesystem effects and cluster send history remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Validated image establishes follower/empty pending state; prior-incarnation and interrupted-write availability remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot creation, installation, compaction and application decoding remain open. |
<!-- /connections -->

Checks: independent round-7 replay passed build and the six existing vote regressions, resolving the local short-build timeout. Round-8 first obligation passed leader-entry implementation, dependencies, parent, build, vote tests and initial_election_2a (runs/checks/1793348881572180167/results.json). The higher-term attempt first failed Verus compilation on compound assignment; explicit addition repaired it. The next attempt passed all selected checks (runs/checks/1793349026460009958/results.json). Final obligation passed build, eight vote regressions, initial election, dependencies and both entries (runs/checks/1793349113968738991/results.json). New regressions exercise an actual higher-term RPC reply and durable image, and reject an obsolete quorum both before and after term-write completion. Combined progress proof passed (runs/checks/1793349161961858243/results.json).

All three retained _2a tests passed on seeds 1–5: runs/checks/1793349161937923332/results.json, runs/checks/1793349171123803070/results.json, runs/checks/1793349171116450716/results.json, runs/checks/1793349171128938387/results.json and runs/checks/1793349171134706985/results.json. This does not complete the 2A behavior: heartbeats and stable timeout/reset handling are absent; timers currently start repeated elections. Other lab families remain unfinished, and full final acceptance has not been run.

Failed approaches retained: truncate/write recovery stops on incomplete images and lacks recoverable generations; closed-spec visibility required an owner lemma; executable enum equality required pattern matching; a concurrent notes edit required a stable rerun; ghost ownership required borrowing; recovery and RPC callers required explicit postconditions. Collector proofs required renaming TallyView, public abstract accessors, loop termination, separate read-only duplicate lookup and exact final-view witnesses. Short Cargo allowances caused local timeouts resolved by independent replay. Round 8 replaced unsupported compound assignment and moved step-down before the term write to stop leadership during pending I/O. No rollback, assumptions, skipped bodies or imported candidate was used.

Next dependency: connect the real sender/receiver envelopes and collector histories to the cluster execution relation; prove term-completion contexts have their actual earlier submission and preserve this relation across crashes. The local term-completion relation checks context/image/current-state equality but does not alone establish network or submitted-operation provenance. Continue heartbeat/reset behavior and then logs while extending the same checked execution relation. Global adapter step/receive/emit and theorem exports remain missing. All five fixed safety goals, full labs, snapshot fidelity, conditional liveness and application correctness remain OPEN.

Round 9 first attempt constructs exact request/response descriptors in shared executable code and uses them at the actual RPC boundaries. The partial adapter now defines explicit local, submission and delivery segments and exact receive/emit extraction. Submission descriptors are not yet host-send completion evidence; runtime/host composition and crash adapters remain OPEN.

Adapter refinement: submission alone cannot count as a send. Pending exact envelopes are now retained separately; only an explicit HostEmission for an unconsumed submitted operation contributes to emit. Delivered envelopes are retained without imposing Raft correctness. The host must still establish that emission event at the actual transport linearization point; no aggregate runtime correspondence is claimed. The entry wrapper supplies the same support module type used by the fixed caller.

Round-9 revised obligation passed build, eight vote regressions, initial election, both dependencies, adapter entry and parent (runs/checks/1793349729237249403/results.json). The fixed execution caller now passes the missing step/receive/emit interfaces and reports missing runtime_step/runtime_invariant/projection functions (runs/checks/1793349735738900075/execution_link.log). The descriptor parent now explicitly invokes its submission connection; a new regression preserves malformed requests and erroneous replies unchanged.

# Receiver and campaign progress

Latest review: independent check-63-1793506445069748073 raw logs show build and all eight formal entries PASS, covering receiver behavior, campaigns, completion authorization, unified Choice history and four recovery entries. All five seeds complete with exit 101 and 15/16 tests; only protected rpc_tests.rs:213 fails. There are no compilation failures, proof failures or tool timeouts. All 30 current Rust source/proof and Cargo files match the checked snapshot. Check-62-1793506381804714594 and all prior failures remain retained. Acceptance is INCOMPLETE and the first dependency is BLOCKED_FIXED_TEST: corrected protected input is necessary. Rereading Core and audit_step confirms that receive preserves an existing same-term vote, start_election changes votes only while increasing the term, and complete changes only pending/halt and Choice observations; recovery retains the recorded vote under the disclosed bounded-window premises. The protected test aborts and joins its sole sender at line 203, making the receive at line 213 immediately return a closed-channel error rather than time out. No permitted candidate-side repair was identified, and no protocol, host, test, snapshot or checkpoint changes were made. Reported cumulative time remaining at entry was 918.7 seconds; the controller retains exact cumulative spending without reset.

Implemented checked construction/recovery, receiver decisions, campaign self-choice, shared operation allocation and matching-completion authorization in one authoritative Core. The inductive type invariant connects actual state, pending action, actual choice history and recovery base. The fixed caller observes actual calls; no tally, promotion or new trusted protocol body was added.

Current disposition: BLOCKED_FIXED_TEST. Independent `check-5-1793501463891997249` raw logs confirm that compilation, inventory and all eight proof configurations complete successfully. All five seed runs complete with exit 101 solely at `tasks/election_start/rpc_tests.rs:213`; this is a functional assertion failure, not a proof failure or tool timeout. Latest final recheck `runs/checks/1793501388965748671/results.json` reproduces that result: eight formal PASS results and 15/16 tests per seed 1–5. Implementation/proof source and protected wiring match the final snapshot.

Supplemental actual execution: added madraft/repro/receiver_pending_rpc.rs and its offline runner, directly importing unchanged madraft/src/raft/raft.rs (and thereby the same Core and fixed host). Seeds 1–5 pass actual pending-write restart, cancellation/no-message observation, stale completion, rival denial, chosen-candidate retry and byte-oracle file IDs. This exercises the RPC suffix that the protected assertion prevents from running. The independent protected test is untouched and still required for acceptance; this diagnostic does not replace it. Raw output is madraft/repro/receiver_pending_rpc.log. Final recheck completed with all eight proofs PASS and the same sole protected assertion failure on all five seeds; no compile/proof failure or timeout occurred.

Actual proof work: pending-ID equality and recovery-origin bounds are maintained by Core; audit_step verifies every Start/Receive/Complete branch preserves an existing vote at unchanged term, monotone term/ID, log/config, halt and unique history. stale_after_recovery proves all notices at or before the retained operation remain inert after later calls. No additional vote-changing path was found in the source audit. The protocol implementation remains unchanged because the independent logs identify no candidate defect requiring repair; this continuation rechecks the unchanged candidate and records the precise upstream observer correction.

Failed acceptance / next obstacle: restart_written_pending aborts and joins the sole observation sender's task at line 203, then requires observed.recv() to time out at line 213. The empty closed channel instead completes immediately with Err, and the pinned MadSim timeout wraps that in Ok. The actual MadSim reproduction `python3 madraft/repro/run_closed_observer.py` confirms this on seeds 1–5. No Core function receives or owns this channel. The fixed test must distinguish no message/channel closure from an actual reply; task instructions prohibit editing it and require reporting contradictions. Acceptance cannot be closed by a permitted Core change. The concrete upstream correction is to require `matches!(timeout(Duration::from_millis(30), observed.recv()).await, Ok(Err(_)))` after abort/join: closure with no value passes, while every delivered value fails. This proposal is unapplied because the test is protected. All host premises remain disclosed in RESULT.md.

Retained evidence under runs/checks/: first obligation 1793500312489856031, first final 1793500403902337952, strengthened obligation 1793500679934930078, strengthened final 1793500745069038486 final 1793500955608472153 final 1793501186601522995 and latest final 1793501388965748671. Independent controller check-1 through check-5 confirm the same blocker. Diagnostic sources/logs remain in madraft/repro/. No rollback, checkpoint restoration, skipped test or proof weakening occurred; cumulative spending and failures remain retained.

<!-- obligation -->
```ini
[obligation]
id = election-start-rules
statement = All fixed core proof callers pass; close local acceptance after resolving the protected closed-channel assertion contradiction.
goal = election-start-rules
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete
entry = madraft/src/raft/vote_core.rs
parent = madraft/proof/vote_lemmas.rs
dependencies =
premises = Host serialization, truthful completion, faithful readback and unchanged forwarding; protected closed-channel test contradiction blocks acceptance.
tests = raft::raft::vote_tests::
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/src/raft/vote_core.rs | PASS: fixed constructors and nonempty receiver/campaign callers; deployed membership remains trusted. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election | madraft/proof/vote_lemmas.rs | PASS: fixed mixed_calls/from_constructor and audit_step cover actual mutations; host serialization remains trusted. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/src/raft/vote_core.rs | PASS: exact return and matching successful Pending; actual I/O truth and unchanged forwarding remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | PASS: fixed restart_window, four recovery callers and stale_after_recovery; faithful readback is a premise, interrupted writes and repeated crashes remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot | — | OPEN, unchanged scaffold and outside this bounded task; no snapshot proof nominated. |
<!-- /connections -->

Connection contracts: Core::new/recover return Some exactly for valid inputs and establish fresh/recovered views; Core::receive/start_election/complete implement the fixed result and after-state specifications and exact choice updates; inspect returns the actual view. Core::facts derives the maintained type invariant without caller preconditions. vote_lemmas::uniqueness and restart_composition are invoked by the fixed executable mixed-history/recovery callers. Source audit found Core::new/recover as the only construction sites and receive/start_election as the only live term/vote mutations; complete changes only Pending/halt and ghost observations, and inspect returns a copy. The unused old State and unfinished lab methods add no active vote mutation. Runtime host paths are RaftHandle::new_receiver → vote_host::load → Core::new/recover; RaftHandle::request_vote → Raft::request_vote → Core::receive → vote_host::save → Core::complete; RaftHandle::start_election → Core::start_election → vote_host::save → Core::complete → RaftHandle::send_vote_request. Host code is unchanged and unverified; exact boundary details are in RESULT.md.

All five global Raft properties, full labs, snapshots, applications and conditional liveness remain OPEN.

Budget record: the handoff began with a reported cumulative 6123.3 seconds remaining; the fresh final check and this continuation spend from that same budget. No reset or restored time is claimed; the controller remains authoritative for exact cumulative accounting.

Continuation disposition: CLOSED_BLOCKED for this bounded work slice; local acceptance remains INCOMPLETE. Inspected all raw logs from independent check-5-1793501463891997249 and compared its checked Core, lemma and host bytes with the actual workspace: exact matches. This fresh independent run rechecks all eight entries and every configured seed. No additional identical local run or protocol edit is justified by these unchanged diagnostics. The next necessary action is an upstream correction of protected rpc_tests.rs:213; a candidate-only continuation cannot repair that channel assertion. The reported remaining cumulative budget at entry was 5907.4 seconds; earlier spending and failures remain retained without reset.

# Receiver and campaign progress

Implemented checked construction/recovery, receiver decisions, campaign self-choice, shared operation allocation and matching-completion authorization in one authoritative Core. The inductive type invariant connects actual state, pending action, actual choice history and recovery base. The fixed caller observes actual calls; no tally, promotion or new trusted protocol body was added.

Current disposition: BLOCKED_FIXED_TEST. Independent `check-2-1793500893132226386` raw logs confirm that compilation, inventory and all eight proof configurations complete successfully. All five seed runs complete with exit 101 solely at `tasks/election_start/rpc_tests.rs:213`; this is a functional assertion failure, not a proof failure or tool timeout. Latest final recheck `runs/checks/1793500955608472153/results.json` reproduces that result: eight formal PASS results and 15/16 tests per seed 1–5. Implementation/proof source and protected wiring match the final snapshot.

Actual proof work: pending-ID equality and recovery-origin bounds are maintained by Core; audit_step verifies every Start/Receive/Complete branch preserves an existing vote at unchanged term, monotone term/ID, log/config, halt and unique history. stale_after_recovery proves all notices at or before the retained operation remain inert after later calls. No additional vote-changing path was found in the source audit. This continuation made no implementation changes because the independent logs identify no candidate defect requiring repair.

Failed acceptance / next obstacle: restart_written_pending aborts and joins the sole observation sender's task at line 203, then requires observed.recv() to time out at line 213. The empty closed channel instead completes immediately with Err, and the pinned MadSim timeout wraps that in Ok. The actual MadSim reproduction `python3 madraft/repro/run_closed_observer.py` confirms this on seeds 1–5. No Core function receives or owns this channel. The fixed test must distinguish no message/channel closure from an actual reply; task instructions prohibit editing it and require reporting contradictions. Acceptance cannot be closed by a permitted Core change. All host premises remain disclosed in RESULT.md.

Retained evidence under runs/checks/: first obligation 1793500312489856031, first final 1793500403902337952, strengthened obligation 1793500679934930078, strengthened final 1793500745069038486 and latest final 1793500955608472153. Independent controller check-1 and check-2 confirm the same blocker. Diagnostic sources/logs remain in madraft/repro/. No rollback, checkpoint restoration, skipped test or proof weakening occurred; cumulative spending and failures remain retained.

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

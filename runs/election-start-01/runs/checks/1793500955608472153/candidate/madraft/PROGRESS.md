# Receiver and campaign progress

Implemented checked construction/recovery, receiver decisions, campaign self-choice, shared operation allocation and matching-completion authorization in one authoritative Core. The inductive type invariant connects actual state, pending action, actual choice history and recovery base. The fixed caller observes actual calls; no tally, promotion or new trusted protocol body was added.

Checks retained from the first slice: obligation `1793500312489856031` and final `1793500403902337952` under `runs/checks/`; both passed all eight formal goals, while every tested seed failed only the protected observer assertion. Independent controller raw logs in `check-1-1793500593779139295` confirm build and eight formal successes, and 15/16 tests per seed 1–5. Processes completed normally: this is a functional assertion failure, not a compilation/proof failure or timeout. No rollback or failed proof attempt occurred.

Continuation: strengthened pending-ID equality and recovery-origin ID bounds in the actual Core type invariant. Added verified executable callers `audit_step` (all Start/Receive/Complete branches preserve an existing vote at unchanged term, term/ID monotonicity, log/config, halt and unique history) and `stale_after_recovery` (any completion at or before the retained operation has no effect, including after later calls). New obligation report `runs/checks/1793500679934930078/results.json` passes all eight formal goals and 15/16 seed-1 tests; final recheck `runs/checks/1793500745069038486/results.json` passes build, inventory, source boundary and all eight formal goals, with exactly the same 15/16 tests on each seed 1–5; status remains INCOMPLETE. Checked implementation/proof source and protected wiring match the stopped final snapshot.

Failed acceptance attempt / next obstacle: protected `restart_written_pending` aborts and joins the only task owning the observation sender, then requires the receiver to time out. Cancellation drops that sender, so the empty channel is closed and receive completes immediately with an error. This is not a synthesized reply. The final test fails at fixed `tasks/election_start/rpc_tests.rs:213` on all five seeds. `python3 madraft/repro/run_closed_observer.py` independently reproduces Ready(Err) and also runs actual MadSim spawn/abort/join/timeout on seeds 1–5, confirming Ok(Err(RecvError)) with an empty closed channel; logs are retained under madraft/repro/. Fixed tests and host wiring were preserved; no permissible Core change can keep a sender that the Core never receives alive. Local acceptance remains OPEN pending an upstream correction of that assertion.

<!-- obligation -->
```ini
[obligation]
id = election-start-rules
statement = Actual construction and receive/start/complete preserve vote history and save authorization; fixed proof callers now pass.
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

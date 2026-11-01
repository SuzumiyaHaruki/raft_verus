# Receiver and campaign progress

Implemented checked construction/recovery, receiver decisions, campaign self-choice, shared operation allocation and matching-completion authorization in one authoritative Core. The inductive type invariant connects actual state, pending action, actual choice history and recovery base. The fixed caller observes actual calls; no tally, promotion or new trusted protocol body was added.

Checks: `python3 scripts/check.py obligation --task election-start --timeout 180` passed all eight formal goals and 15/16 tests on seed 1. `python3 scripts/check.py final --task election-start --timeout 600` passed build, inventory, source boundary and all eight formal goals; each seed 1–5 passed 15/16 tests. Final report: `runs/checks/1793500403902337952/results.json`, status INCOMPLETE. The original scaffold's unimplemented bodies and failed proof assertions are replaced; no generated proof attempt failed and no rollback occurred.

Failed acceptance attempt / next obstacle: protected `restart_written_pending` aborts and joins the only task owning the observation sender, then requires the receiver to time out. Cancellation drops that sender, so the empty channel is closed and receive completes immediately with an error. This is not a synthesized reply. The final test fails at fixed `tasks/election_start/rpc_tests.rs:213` on all five seeds. `python3 madraft/repro/run_closed_observer.py` independently reproduces Ready(Err) using the pinned async_channel dependency. Fixed tests and host wiring were preserved; no permissible Core change can keep a sender that the Core never receives alive. Local acceptance remains OPEN pending an upstream correction of that assertion.

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
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election | madraft/proof/vote_lemmas.rs | PASS: fixed mixed_calls/from_constructor observe actual choices; host serialization remains trusted. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/src/raft/vote_core.rs | PASS: exact return and matching successful Pending; actual I/O truth and unchanged forwarding remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | PASS: fixed restart_window and four recovery callers; faithful readback is a premise, interrupted writes and repeated crashes remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot | — | OPEN, unchanged scaffold and outside this bounded task; no snapshot proof nominated. |
<!-- /connections -->

Connection contracts: Core::new/recover return Some exactly for valid inputs and establish fresh/recovered views; Core::receive/start_election/complete implement the fixed result and after-state specifications and exact choice updates; inspect returns the actual view. Core::facts derives the maintained type invariant without caller preconditions. vote_lemmas::uniqueness and restart_composition are invoked by the fixed executable mixed-history/recovery callers. Runtime host paths are RaftHandle::new_receiver → vote_host::load → Core::new/recover; RaftHandle::request_vote → Raft::request_vote → Core::receive → vote_host::save → Core::complete; RaftHandle::start_election → Core::start_election → vote_host::save → Core::complete → RaftHandle::send_vote_request. Host code is unchanged and unverified; exact boundary details are in RESULT.md.

All five global Raft properties, full labs, snapshots, applications and conditional liveness remain OPEN.

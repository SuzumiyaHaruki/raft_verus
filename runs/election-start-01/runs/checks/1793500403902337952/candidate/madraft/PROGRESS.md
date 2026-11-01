# Receiver and campaign progress

Implemented one authoritative Core with checked construction/recovery, receiver decisions, campaign self-choice, shared operation allocation and matching-completion authorization. The ghost history records actual accepted self-choices and affirmative replies. The inductive type invariant relates histories to the current persistent vote, pending actions and the recovered base record. No tally or promotion was added.

Current attempt: first independent obligation check running. Starting failures were the supplied unimplemented bodies and false proof assertions; no earlier candidate was imported. Next obstacle is discharging the type invariant at actual state mutations and composition of the two recovery windows.

<!-- obligation -->
```ini
[obligation]
id = election-start-rules
statement = Actual construction and receive/start/complete preserve vote history and save authorization.
goal = election-start-rules
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete
entry = madraft/src/raft/vote_core.rs
parent = madraft/proof/vote_lemmas.rs
dependencies =
premises = Serialized host calls, truthful completion, faithful readback and unchanged forwarding.
tests = raft::raft::vote_tests::
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/src/raft/vote_core.rs | Checked validation and type invariant; verification pending. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election | madraft/proof/vote_lemmas.rs | Fixed caller observes actual calls; host serialization remains trusted. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/src/raft/vote_core.rs | Matching success authorizes actual return; truthful I/O and unchanged forwarding remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | Faithful readback in the two bounded windows; arbitrary crashes remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot | madraft/proof/implementation.rs | OPEN and outside this bounded task. |
<!-- /connections -->

All five global Raft properties, full labs, snapshots, applications and conditional liveness remain OPEN.

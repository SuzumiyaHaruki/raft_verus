Round 2 repaired the first independent proof failures: the actual Core invariant now constrains output Choice node/term/candidate, equal-term current vote, recovered base and pending granted Reply. vote_lemmas::uniqueness and restart_composition are proved from that invariant; executable decisions and public contracts are unchanged. Core check 1793624216318333848 PASS (59 verified, zero errors); local_history 1793624251002251375 PASS. Independent check-1 build/inventory/43 seed-1 tests PASS; this round's 43 seed-2 tests PASS (1793624344713672616), and the obligation run's seed-1 tests also PASS.

The new source_check::finite_sources executes arbitrary structurally enabled fixed recovery-network prefixes. recovery_source proves initialization and single-step preservation of valid pending/retained records, actual successful-write origins, and all restarted instances' exact record/base/empty-volatile correspondence. Check 1793624447135242101 PASS (90 verified, zero errors); an extension asserting all four retention predicates together timed out at 25 seconds (1793624512256568125). Removed the three redundant prefix assertions, retaining exact frame-prefix correspondence and all historical write-source obligations; the fixed executor continues to retain Choices, sends and replies. No safety predicate enables a restart and no pre-restart evidence is deleted.

Failed approach/repair: the first source induction omitted explicit instantiations of boot's epoch/liveness quantifier, sequence prefix facts and the actual executor's frame-index bound; those caused proof failures, not execution failures. The checked source theorem includes the machine bound discharged by its actual loop. Next obstacle: construct the old network invariant and the cross-instance archived Choice/current vote/retained image/majority invariant, then compose source induction with full recovery preservation and historical Election Safety. The original general network and full recovery lemma bodies remain OPEN.

Checks are distinct: safety 1793624252156454503 passed rules, campaign, authorization, history, reply_rules, self_vote, rounds, higher_reply and promotion; provenance/nonempty failed mathematically at network_initial, election_basis timed out, and later groups were NOT RUN after budget exhaustion. Obligation 1793624424773628093 passed build/inventory/tests and eight proof groups, then provenance timed out. The root PROGRESS.md discovery mismatch is documented in OBJECTION.md and is separate from those failures. All other safety targets, full labs, arbitrary/interrupted-write recovery, snapshots and liveness remain OPEN. Truthful I/O/readback, identity, encoding, RPC forwarding, callback incarnation/cancellation and scheduling remain UNVERIFIED host premises detailed in RESULT.md.

<!-- obligation -->
```ini
[obligation]
id = successful-write-origin
statement = All actual enabled finite executor prefixes retain successful Write origins and recover from the same node's retained record while retaining prior Choice/message/frame evidence.
goal = election_safety
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::recover
entry = madraft/proof/source_check.rs
parent =
dependencies = madraft/proof/local_history.rs
premises = Truthful I/O and fixed host mapping; archived Choice uniqueness and current/historical majority provenance remain open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/source_check.rs | Actual fixed executor initialization checked; full network invariant open. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/local_history.rs | Local invariant checked; historical majority induction open. |
| outputs | madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::start_election | madraft/proof/source_check.rs | Truthful I/O and RPC mapping remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/source_check.rs | All-prefix write origins checked; archived vote/majority composition open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Unimplemented and unproved. |
<!-- /connections -->

Stopped-candidate final 1793624584766961427 completed INCOMPLETE: build/inventory and all 43 election tests at every seed 1–5 PASS; the same nine scoped groups listed above PASS. provenance, nonempty, election_basis, election_history and election_nonempty FAIL at unfinished network_initial. receive_confirmed TIMEOUT under the remaining final budget; the other three local windows and all three general restart groups were NOT RUN due to budget exhaustion. Final source_check recheck 1793624583630103300 PASS (90 verified, zero errors), including exact frame-prefix correspondence. No complete Election Safety status is claimed.

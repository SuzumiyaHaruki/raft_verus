Round 3 repaired network_initial, network_step and network_conclusion. network_invariant tracks the actual Core history and pending write/request association, real Send/Reply emissions, every counted-vote source, and historical promotion majorities. Initialization, local step preservation, retained-evidence extension and majority-intersection safety PASS (network_check 1793625498195146313: 83 verified, zero errors). Its executable finite_actual_network wrapper invokes real fixed new_network/is_enabled/execute on every enabled finite prefix and checks sources and historical Election Safety: PASS 1793625561472613953 (85 verified, zero errors). Only visibility of three already-verified pure Core lemmas changed; executable decisions and all earlier contracts remain unchanged.

Earlier checked evidence remains: Core/local Choice uniqueness and restart_composition PASS; all 43 election tests passed seeds 1–5 in round-2 final 1793624584766961427. Independent check-2 additionally passed all four local recovery windows and the concrete restart witness. The all-prefix successful-write source proof source_check 1793624583630103300 PASS (90 verified, zero errors) preserves actual retained-image origins and recovered instances across repeated restarts. These source/local results do not establish general restarted-cluster safety.

Failed approach/repair: one monolithic network-preservation assertion omitted source witnesses and did not instantiate pending/sent evidence. Split it into actor_local, extension, lift_choice and lift_evidence, supplying actual send/delivery indices; an explicit delivered[j] trigger repaired compilation. All parts now verify. Next concrete obstacle: recovery_lemmas::initialize/preserve/conclude must combine the checked successful-write source invariant with a cross-instance ledger lock and retained current/historical majority evidence. A voter restart clears its local Choice suffix, so the no-restart node-history invariant cannot simply be reused; old sends, replies, ledger Choices and election frames must remain available.

Truthful I/O/readback, fixed identity/peer mapping, encoding, callback incarnation/cancellation, RPC forwarding and scheduling remain UNVERIFIED host premises. The root PROGRESS.md discovery mismatch in OBJECTION.md is a tool issue, separate from proof failures/timeouts. Full Raft/labs, four other safety properties, arbitrary/interrupted-write recovery, snapshots and liveness remain OPEN. Final checks are running on stopped code; exact outcomes are appended after completion.

<!-- obligation -->
```ini
[obligation]
id = network-history
statement = Every enabled actual no-restart network call preserves message and Choice provenance and historical strict-majority Election Safety.
goal = election_safety
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/network_check.rs
parent =
dependencies = madraft/proof/local_history.rs
premises = Truthful I/O, stable peer mapping and async scheduling remain unverified. Cross-instance ledger/retained-image/majority induction remains open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/network_check.rs | Actual no-restart initialization checked; restarted-cluster invariant open. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/network_check.rs | Actual no-restart preservation checked; cross-instance ledger lock open. |
| outputs | madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::start_election | madraft/proof/network_check.rs | Actual no-restart sources checked; host I/O and RPC mapping unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/source_check.rs | All-prefix write origins checked; archived vote/majority composition open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Unimplemented and unproved. |
<!-- /connections -->

Stopped-candidate round-3 final 1793625649869373950: INCOMPLETE. PASS: build, inventory, tests_seed_1_raft_raft_vote_tests, tests_seed_2_raft_raft_vote_tests, tests_seed_3_raft_raft_vote_tests, tests_seed_4_raft_raft_vote_tests, tests_seed_5_raft_raft_vote_tests, source_boundary, recovery_rules, recovery_campaign, recovery_authorization, recovery_history, recovery_reply_rules, recovery_self_vote, recovery_rounds, recovery_higher_reply, recovery_provenance. OPEN: recovery_nonempty: verification_timeout; recovery_promotion: check_budget_exhausted; recovery_election_basis: check_budget_exhausted; recovery_election_history: check_budget_exhausted; recovery_election_nonempty: check_budget_exhausted; recovery_receive_confirmed: check_budget_exhausted; recovery_receive_pending: check_budget_exhausted; recovery_self_confirmed: check_budget_exhausted; recovery_self_pending: check_budget_exhausted; recovery_restart_source: check_budget_exhausted; recovery_restart_history: check_budget_exhausted; recovery_restart_nonempty: check_budget_exhausted.

Safety 1793625562729749325 passed provenance/nonempty in addition to earlier local groups; election_basis timed out under its 75-second total budget and later groups were not run. Current actual-call network wrapper PASS remains 85 verified, zero errors. Full completed-write restart safety remains OPEN at recovery_lemmas::initialize and its cross-instance ledger/majority preservation; host premises remain UNVERIFIED.

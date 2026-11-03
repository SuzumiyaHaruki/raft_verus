Round 5 redesigns only the failed recovery-step decomposition, motivated by independent check-4 resource failures in disk_step, nodes_step and provenance_step. Core bodies, fixed inputs, actual restart windows and retained Choice/message/election history are unchanged. Disk lifecycle and pending-RPC association now have nine event-specific lemmas using small per-node premises; node linkage composes local state, ledger locks, disk facts, pending association and suffix retention through a pure composition lemma. Provenance separates individual votes and historical election frames. Solver limits remain below the fixed default; tools.ini and acceptance are unchanged.

Checked redesign progress: disk-case redesign 1793627587489315253 verified 122 functions with only nodes/provenance resource failures. After further splits, 1793627815248236216 verified 138 functions; all disk and node-linkage obligations passed. Remaining diagnostics were a generic vote-case resource limit and one frame node-index assertion. The last revision splits vote evidence by event type and adds net.nodes.len<=usize::MAX to the invariant/initialization contract. This is the real Vec-based cluster bound, to be discharged by the unchanged constructor/caller, not a new voting or restart premise. The full actual finite caller recheck passed 149 verified, 0 errors (1793627932208800533), including affected initialization and every imported step body.

Failures remain retained: 1793627632811186123, 1793627712325715002 and 1793627739078110850 timed out, with resource diagnostics in the earlier broad pending/nodes composition. These are distinct from the frame-index assertion in 1793627815248236216. Earlier independent check-4 passes all 18 preserved groups, the concrete restart witness, build/inventory and seed-1 election tests; prior final runs passed all 43 election tests at seeds 1–5. Actual source induction and no-restart executable safety remain checked dependencies. All imported step bodies now pass the direct executable recovery proof; fixed scoped acceptance remains OPEN pending completion of the scoped final checks.

Current obstacle: rerun the fixed scoped restart-source/history callers and remaining groups with enough check budget. The actual executable arbitrary finite completed-write restart proof now PASSES: runs/checks/1793627932208800533/proof.log reports 149 verified, 0 errors, completed exit 0, with all imported bodies checked. Final stopped-candidate check 1793627979954497291 is INCOMPLETE: promotion timed out after 1.82 seconds remaining in the total 85-second budget; ten subsequent groups including restart source/history were NOT RUN (check_budget_exhausted). These are not proof counterexamples. No new fixed-input objection; prior root-progress discovery objection remains separate. Host I/O, storage retention, serialization, identity/RPC forwarding, callback incarnation/cancellation and scheduling remain UNVERIFIED; full labs, other four safety properties, snapshots, arbitrary/interrupted-write recovery and liveness remain OPEN.

<!-- obligation -->
```ini
[obligation]
id = restart-ledger-preservation
statement = Actual recovery initialization establishes ledger/disk/source invariants, and every enabled recovery step must preserve archived Choices and current/historical election evidence.
goal = election_safety
kind = step
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::recover, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader
entry = madraft/proof/recovery_check.rs
parent =
dependencies = madraft/proof/recovery_initial_check.rs, madraft/proof/source_check.rs, madraft/proof/network_check.rs
premises = Actual finite-loop initialization, ledger/disk/provenance preservation and safety conclusion checked (149 verified, 0 errors); fixed scoped reruns outstanding. Host I/O, encoding, identity, callback association and scheduling are unverified.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/recovery_check.rs | Actual new_network and Core construction establish the recovery invariant; host initialization unverified. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/recovery_check.rs | Actual enabled execute steps preserve retained global ledger and historical elections. |
| outputs | madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::start_election | madraft/proof/recovery_check.rs | Actual Send/Reply emissions and counted source correspondence checked; host I/O and RPC mapping unverified. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/recovery_check.rs | Actual successful writes, recovered instances and archived vote/majority composition checked; physical durability unverified. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Unimplemented and unproved. |
<!-- /connections -->

Stopped final results: build: PASS (checked); inventory: PASS (checked); tests_seed_1_raft_raft_vote_tests: PASS (checked); tests_seed_2_raft_raft_vote_tests: PASS (checked); tests_seed_3_raft_raft_vote_tests: PASS (checked); tests_seed_4_raft_raft_vote_tests: PASS (checked); tests_seed_5_raft_raft_vote_tests: PASS (checked); source_boundary: PASS (same_candidate_imports); recovery_rules: PASS (checked); recovery_campaign: PASS (checked); recovery_authorization: PASS (checked); recovery_history: PASS (checked); recovery_reply_rules: PASS (checked); recovery_self_vote: PASS (checked); recovery_rounds: PASS (checked); recovery_higher_reply: PASS (checked); recovery_provenance: PASS (checked); recovery_nonempty: PASS (checked); recovery_promotion: OPEN (verification_timeout); recovery_election_basis: OPEN (check_budget_exhausted); recovery_election_history: OPEN (check_budget_exhausted); recovery_election_nonempty: OPEN (check_budget_exhausted); recovery_receive_confirmed: OPEN (check_budget_exhausted); recovery_receive_pending: OPEN (check_budget_exhausted); recovery_self_confirmed: OPEN (check_budget_exhausted); recovery_self_pending: OPEN (check_budget_exhausted); recovery_restart_source: OPEN (check_budget_exhausted); recovery_restart_history: OPEN (check_budget_exhausted); recovery_restart_nonempty: OPEN (check_budget_exhausted)

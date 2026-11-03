Implemented the shared executable Core constructors, receiving, campaign preparation, completion authorization, collection of distinct votes, majority promotion and recovery. All 43 election tests passed at seed 1 in runs/checks/1793623374295078319; the first cold test invocation timed out during compilation. No earlier candidate was imported.

Current proof repair: the first direct field invariant approach failed because intermediate field mutations did not preserve the invariant. A private Data transition now works on a copy and restores the Core invariant at its public boundary. Core verification is being rerun; local uniqueness and general network induction remain OPEN.

<!-- obligation -->
```ini
[obligation]
id = successful-write-origin
statement = An actual campaign Write followed by successful Write, optional Notify, Stop and Restart retains the observed Choice ledger and recovers exactly that node's retained record.
goal = election_safety
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::start_election, madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::recover
entry = madraft/proof/recovery_connection.rs
parent =
dependencies = madraft/proof/core_check.rs
premises = Truthful completed-write/readback and fixed host mapping; arbitrary repeated-history induction and archived vote provenance remain open.
tests = raft::raft::vote_tests
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/proof/core_check.rs | Establish private invariant and fixed host inputs. |
| transitions | madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::collect, madraft/src/raft/vote_core.rs::try_become_leader | madraft/proof/core_check.rs | General network preservation remains open. |
| outputs | madraft/src/raft/vote_core.rs::complete, madraft/src/raft/vote_core.rs::start_election | madraft/proof/recovery_connection.rs | Truthful I/O completion, RPC mapping and scheduling remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/recovery_connection.rs | Repeated restart ledger/provenance induction remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Unimplemented and unproved. |
<!-- /connections -->

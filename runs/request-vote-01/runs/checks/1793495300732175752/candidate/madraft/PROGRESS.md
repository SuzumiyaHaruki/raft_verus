# RequestVote progress

Implemented the actual synchronous Core used by the existing Raft RPC: checked fresh/recovered construction, term/log/vote decision, ordered rejection, operation exhaustion, pending save, matching completion, failed-save halt and exact ghost output accounting. Private type invariants connect every emitted grant to the current vote and to the recovered origin record; constructors establish them and receive/complete preserve them. The fixed finite caller observes all actual replies.

Checks: direct Verus with the fixed pending caller and --no-cheating passed (32 verified, 0 errors). Official obligation attempt 1 failed compilation: reserved invariant identifier, then direct compilation identified spec-mode borrowing and ghost assignment; repaired with core_ok, tracked proof references and ghost field updates. Official obligation attempt 2 is running. Next: all configured RPC seeds and all four fixed proof entries on one stopped candidate. No rollback or imported candidate was used.

<!-- obligation -->
```ini
[obligation]
id = request-vote-recovery-composition
statement = Actual constructors and transitions preserve output uniqueness and the recovered-record constraint for arbitrary finite calls.
goal = execution_link
kind = composition
functions = madraft/src/raft/vote_core.rs::new, madraft/src/raft/vote_core.rs::recover, madraft/src/raft/vote_core.rs::receive, madraft/src/raft/vote_core.rs::complete
entry = madraft/proof/vote_lemmas.rs
parent =
dependencies = madraft/src/raft/vote_core.rs
premises = Faithful I/O readback, genuine completion, serialized host calls and forwarding to the original RPC requester remain unverified host premises.
tests = raft::raft::vote_tests::
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/vote_core.rs::new | madraft/src/raft/vote_core.rs | Fixed constructors/nonempty caller checks exact state and a real grant; host configuration and log fixture are inputs. |
| transitions | madraft/src/raft/vote_core.rs::receive | madraft/src/raft/vote_core.rs | Fixed rules and finite_calls check every actual decision; async host serialization remains unverified. |
| outputs | madraft/src/raft/vote_core.rs::complete | madraft/proof/vote_lemmas.rs | Fixed caller appends every returned reply; authentic save completion and RPC forwarding remain host premises. |
| recovery | madraft/src/raft/vote_core.rs::recover | madraft/proof/vote_lemmas.rs | Fixed restart_window checks confirmed and written-pending windows under readback equality; arbitrary crashes remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | | OPEN; unrelated student TODOs preserved. |
<!-- /connections -->

The fixed call path remains RaftHandle::new/new_receiver → vote_host::load → Core::new/recover, and start_rpc_server → RaftHandle::request_vote → Raft::request_vote → Core::receive → vote_host::save → Core::complete. The code does not add a service or move voting rules into the host. Five global safety properties, full MadRaft, full log correspondence and conditional liveness remain OPEN.

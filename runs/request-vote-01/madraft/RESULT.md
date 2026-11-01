# RequestVote result

REQUEST_VOTE_CHECKS_PASSED for the bounded receiver task. Official final evidence: [results](../runs/checks/1793495300732175752/results.json) and [summary](../runs/checks/1793495300732175752/results.md). This is local machine-check acceptance; semantic review remains required and full implementation acceptance is not established.

| Local proof | Outcome | Actual connection |
| --- | --- | --- |
| rules | PASS; 26 verified, 0 errors | Core::new/recover validate external state; receive obeys fixed term, vote, log, rejection and counter rules; complete authorizes only matching successful saves; inspect exposes exact state. Fixed nonempty constructs and obtains a real positive reply. |
| history | PASS; 30 verified, 0 errors | Private type invariant is established by constructors and preserved by both transitions. Fixed finite_calls records every actual reply for any finite event vector; uniqueness follows from the invariant. |
| confirmed | PASS; 32 verified, 0 errors | Actual receive and successful complete precede recover of the same record; arbitrary finite suffix outputs compose with the prior output history. |
| pending | PASS; 32 verified, 0 errors | Actual receive precedes recover of the written record without inventing a completion or old reply; arbitrary suffix composes with the existing history. |

The invariant connects granted output terms to the current persistent term, fixes every equal-term grant to the current voted_for, and preserves uniqueness of prior outputs. Pending granted replies satisfy the same constraint. A recovered origin additionally bounds future grants from below and fixes equal-origin-term grants to the retained vote. The restart proof combines the prefix upper bound with the suffix lower bound; equal-term cross-boundary grants must name the same candidate. Ghost outputs are erased at runtime, and the fixed caller checks their exact correspondence to every actual complete return.

Build, inventory and same-candidate source boundary passed. All eight fixed tests passed for each configured seed 1, 2, 3, 4, 5 (40 test executions): rules_and_nonempty; invalid_busy_failure_completion_and_bounds; rpc_rules_and_file; rpc_nonempty_log_and_denial_persist; rpc_delayed_and_failed_save; restart_confirmed; restart_written_pending; malformed_storage_is_error. These exercise the original registered RPC, independent file-byte assertions, a real write failure, pre-write delay, same-node kill/restart with fresh endpoint/Core, malformed files, nonempty log fixtures, maximum terms/indexes and operation exhaustion. The other 60 discovered tests were filtered out by this task's fixed runner.

The original path is RaftHandle::new/new_receiver → vote_host::load → Core::new/recover; start_rpc_server registers RaftHandle::request_vote, which calls Raft::request_vote → Core::receive, awaits vote_host::save, then calls Core::complete and forwards its authorized reply. Protocol decisions remain in the verified synchronous core. Only vote_core.rs and vote_lemmas.rs were changed as code; other student TODOs, fixed interfaces, tests and checker were preserved.

Host boundary remains UNVERIFIED: RaftHandle::new_receiver, start_rpc_server and request_vote; Raft::request_vote argument adaptation; vote_host::{load,encode,decode,save,checkpoint}; mutexes, async scheduling, RPC identity/forwarding, actual I/O results and faithful retained-record readback. Verus/vstd/Rust/SMT are trusted. The two recovery theorems require readback equal to the actual emitted record and no intervening accepted operation. A success boolean does not prove external I/O. MadSim completed writes survive node restart, sync_all is a no-op, and create truncates; this result does not establish real-disk durability or interrupted-write crash consistency.

Recovery coverage is restricted to confirmed-save and written-but-uncompleted windows. Arbitrary crashes, recovery after failed/interrupted writes, other term/vote-changing paths, full replicated-log-to-summary correspondence and snapshots remain OPEN. No snapshot creation, installation, decoding or application correctness is claimed.

| Global target | Outcome |
| --- | --- |
| Election Safety | OPEN |
| Leader Append-Only | OPEN |
| Log Matching | OPEN |
| Leader Completeness | OPEN |
| State Machine Safety | OPEN |

| Student lab | Outcome |
| --- | --- |
| Raft 2A | INCOMPLETE; receiver task locally checked, candidate tally/majority/election not completed |
| Raft 2B | OPEN |
| Raft 2C | INCOMPLETE; only the two bounded receiver recovery windows checked |
| Raft 2D | OPEN |
| KV 3A/3B | OPEN |
| Shard controller 4A | OPEN |
| Sharded KV 4B | OPEN |

Full MadRaft is INCOMPLETE. Conditional liveness and application correctness beyond the Raft properties remain OPEN. No assume, admit, skipped consensus body or imported earlier solution was used.

Reproduce from the workspace root using the existing tools.ini and offline dependencies:

```bash
python3 scripts/check.py obligation --task request-vote --timeout 120
python3 scripts/check.py safety --task request-vote --timeout 240
python3 scripts/check.py final --task request-vote --timeout 300
```

The successful obligation evidence is runs/checks/1793495270835672399; the earlier compilation failure is retained in runs/checks/1793495224351266169. Final code was unchanged after the successful final snapshot; only these reports were completed afterward.

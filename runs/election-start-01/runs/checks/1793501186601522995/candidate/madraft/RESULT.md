# Receiver-plus-campaign result

The actual shared Core and its invariant proofs are implemented. Full local acceptance is INCOMPLETE because one protected test has a reproducible closed-channel contradiction; ELECTION_START_CHECKS_PASSED was not achieved. No fixed specification, caller, test, host wiring or global tool was changed.

Latest handoff disposition: BLOCKED_FIXED_TEST. Final recheck `runs/checks/1793500955608472153/results.json` passes all eight formal entries, build, inventory and source boundary, with 15/16 tests on each of seeds 1–5. Raw independent `check-2-1793500893132226386` logs and the latest local logs show completed processes: build/proofs exit 0, tests exit 101 at the protected assertion, and no tool timeouts. No candidate source change was justified in this continuation. All previous failures and cumulative spending are retained.

| Local obligation | Outcome |
| --- | --- |
| Receiver rules and real constructors | PASS |
| Campaign guards, self-choice, checked increment and immutable payload | PASS |
| Matching-success reply/Send authorization | PASS |
| Arbitrary finite mixed actual-call histories | PASS |
| Receiver recovery, confirmed and written-pending windows | PASS / PASS |
| Self-vote recovery, confirmed and written-pending windows | PASS / PASS |
| Cargo build, inventory and same-candidate imports | PASS |
| RPC/file/core tests on seeds 1, 2, 3, 4, 5 | 15 passed, 1 failed per seed |

Evidence: obligation report `runs/checks/1793500312489856031/results.json`; final report `runs/checks/1793500403902337952/results.json`. Every formal configuration uses the fixed caller, actual Cargo source and all imported bodies under --no-cheating. These passes establish the bounded local contracts; they do not establish global implementation acceptance or replace semantic review.

Continuation evidence: inspected independent raw logs in `/home/nitro/Desktop/raft_verus/runs/.controller/election-start-01/check-1-1793500593779139295`; all build/proof processes completed successfully and all five test runs completed with the same single assertion failure. There were no tool timeouts. The continuation final report `runs/checks/1793500745069038486/results.json` is INCOMPLETE: build, inventory, source boundary and all eight proof configurations PASS; all five seeds again pass 15/16 tests and fail only rpc_tests.rs:213. Checked implementation/proof sources and protected wiring match its stopped snapshot. A new obligation run at `runs/checks/1793500679934930078/results.json` verifies the strengthened invariant and audit callers; all eight local goals pass, with 15/16 tests on seed 1. Earlier failures/checkpoints remain retained, and no rollback was performed.

The continuation strengthens the invariant to require Pending.op + 1 = next, next greater than the recovery record's operation, and every new Pending operation greater than that recovery operation. The verified actual-call audit `vote_lemmas::audit_step` covers every Event variant and establishes preservation of an existing vote when the term stays equal, monotone terms/IDs, unchanged log/config, sticky halt and unique Choice history. `vote_lemmas::stale_after_recovery` establishes that every completion ID at or below the retained operation is inert in any invariant-satisfying recovered Core, even after later operations. Both callers are verified as imports of every fixed proof configuration. They are audit callers, not a replacement async host or an expanded crash model.

The invariant requires valid persistent state, pending actions consistent with that state, uniqueness of the recorded choices, and bounds relating every choice to the current term/vote. Recovery also establishes a lower bound at the retained term and preserves its chosen candidate at that term. Real operations preserve these facts. The fixed observer records accepted campaign self-choices before completion and affirmative replies from actual completion results. The composition proof combines arbitrary actual prefixes and suffixes across both specified successful-write windows without inventing an old completion. A self-choice is neither durable I/O evidence nor a counted vote.

Passing tests cover receiver rejection priority and log freshness, repeated same-candidate votes, higher-term denials, shared campaign/receiver state, exhaustion, wrong/duplicate completion, halting on failed save, real candidate-to-two-receiver RPCs, delayed save, file byte contents, malformed storage, confirmed receiver recovery, both self-vote recovery windows and no single-member promotion. The written-pending receiver test reaches and passes retained-file/recovered-state assertions before failing on its observer check; its later post-recovery RPC assertions are not reached. Formal receiver pending recovery nevertheless passes.

Protected-test contradiction: `tasks/election_start/rpc_tests.rs:188` creates the observation channel; the task receives its sole sender. At line 203 the test aborts and awaits that task, dropping the sender without sending a reply. At line 213 it asserts that `timeout(..., observed.recv()).await.is_err()`. Empty closed-channel receive returns Err immediately, so timeout returns Ok(Err), making this assertion false even with no old reply. The same failure occurs on all five seeds. The independent diagnostic `madraft/repro/closed_observer.rs` polls the actual pinned channel after dropping a future owning its only sender and observes Ready(Err). Its output is retained in `madraft/repro/closed_observer.log`. The additional `madraft/repro/cancelled_observer.rs` uses the actual pinned MadSim runtime to spawn, start, abort and join the sole-sender task, then invokes the actual 30 ms timeout. On seeds 1–5 it confirms that the channel is empty and closed and the result is Ok(Err(RecvError)); output is retained in `madraft/repro/cancelled_observer.log`. The pinned MadSim TimeHandle::timeout implementation in madsim/src/sim/time/mod.rs uses a biased select with the input future first and returns Ok(res) for a ready result. Thus even a queued value would produce outer Ok; the protected assertion cannot be satisfied after the sole sender has been dropped. A correct no-message observer must distinguish channel closure from a delivered value; the protected test has been left intact. The Core neither owns nor receives this channel, so repairing its lifetime would require changing protected test/host behavior or an unrelated workaround.

Unverified host premises: serialized receive/save/complete and campaign/save/complete calls; actual write results reported truthfully; retained bytes read back equal to the emitted Record in the bounded recovery windows; encoding/decoding preserve values; configured membership/address mapping and requester identity are correct; authorized replies and immutable requests are forwarded unchanged; cancellation releases the old Core before recovery. Exact functions are vote_host::{encode,decode,load,checkpoint,save}, RaftHandle::{new_receiver,start_rpc_server,request_vote,start_election,send_vote_request}, and the mutex/task/network/filesystem operations they call. Core validation discharges domain checks without assuming valid external data; protocol decisions and authorization remain in the verified Core. Source inspection confirms new_receiver constructs the sole Core, receive/start_election perform all active term/vote mutations, complete only changes Pending/halt plus ghost observations, and term/is_leader/start inspect that same Core. The old State is unused; no active alternate vote authority was found. The async host itself is not formally verified. MadSim completed-write retention and sync_all do not prove real-disk power-loss durability. The Rust/Verus/vstd/SMT toolchain and ghost erasure remain trusted.

| Global target / lab | Outcome |
| --- | --- |
| Election Safety | OPEN |
| Leader Append-Only | OPEN |
| Log Matching | OPEN |
| Leader Completeness | OPEN |
| State Machine Safety | OPEN |
| Raft 2A | INCOMPLETE: bounded receiver/campaign subset only |
| Raft 2B, 2C, 2D | OPEN: no full replication, crash protocol or snapshots |
| KV 3A/3B | OPEN |
| Shard controller 4A | OPEN |
| Sharded KV 4B | OPEN |
| Application correctness and conditional liveness | OPEN |

Additional open coverage includes arbitrary interrupted writes, recovery after failed writes, induction over repeated crashes, actual replicated-log-to-summary correspondence, reply provenance/higher reply terms, tallying, majority reasoning, leader promotion, heartbeats, snapshot creation/installation/decoding and application observations. No such functionality was added or claimed.

Reproduce from the workspace root:

```bash
python3 scripts/check.py obligation --task election-start --timeout 180
python3 scripts/check.py final --task election-start --timeout 600
python3 madraft/repro/run_closed_observer.py
```

The first two commands preserve the failing test and reproduce INCOMPLETE with all eight formal goals PASS. The final command prints READY_CLOSED and the five MadSim cancellation/timeout results without sending any value.

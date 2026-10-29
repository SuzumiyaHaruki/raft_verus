# Result

Incomplete overall: the student executable work is implemented and all active retained lab tests pass, but none of the five required global safety theorems has been completed. The independent aggregate safety check fails because `madraft/proof/implementation.rs` and its execution/refinement proof are missing.

| Lab | Current seed 1 result |
| --- | --- |
| Raft 2A | 3 passed |
| Raft 2B | 7 passed |
| Raft 2C | 8 passed |
| Raft 2D | 5 passed |
| KV 3A | 12 passed |
| KV 3B | 7 passed |
| Shard controller 4A | 2 passed |
| Sharded KV 4B | 12 passed; original empty `unreliable3_4b` placeholder remains ignored |
| Additional checks | 7 new regressions and 3 supplied host regressions passed |

The full current suite reports 66 passed, 0 failed and 1 ignored. Earlier revisions also passed full suites at seeds 2–5. These are finite simulator results, not universal safety, application correctness or liveness proofs. Original tests and independent check scripts were not changed.

| Global target | Outcome |
| --- | --- |
| Election Safety | OPEN |
| Leader Append-Only | OPEN |
| Log Matching across nodes and time | OPEN |
| Leader Completeness | OPEN |
| State Machine Safety | OPEN |
| Full observation/refinement and snapshot fidelity | OPEN |

The local Verus entrypoint verifies 89 items from the actual executable `state.rs`, `log.rs`, `quorum.rs`, `cursor.rs`, `core.rs` and `storage.rs`. It proves initialization/preservation of the synchronous core representation and functional contracts for voting, proposal extension, outgoing log slices, follower repair, current-term majority selection, ordered application, snapshot suffix/cursor transformations and persistent-data dirty tracking. A directly linked constructor proof and a conditional zero-event initial-history base case use the supplied target vocabulary; they do not establish any nonempty execution theorem. The proof does not establish durable origins of vote/message evidence, matching-prefix certificates for replication cursors, the initialized cluster/history invariant, async host correspondence or decoder refinement. Conditional lemmas remain conditional. No `assume`, `admit`, external consensus body or fabricated aggregate theorem closes these gaps.

Snapshot/recovery coverage includes all retained 2D/3B/4B tests, the snapshot size and shard-deletion checks, real-file tests at all seven normal image-write phases, incoherent-image rejection, a regression that first failed and then passed after fixing a second crash during recovery, and a regression that first failed and then passed after making suffix application wait for acknowledgment of the current installed snapshot. `state` and `snapshot` retain their required names; `raft.backup` stores one additional complete current image, including snapshot bytes. Recovery preserves that authority while repairing both primary projections before replacing the backup. The extra copy is bounded by the current image, rather than an accumulated log history. Formal crash/storage refinement and proof that the actual service decoder equals execution of the represented committed prefix remain incomplete.

Deployment inputs identify a group by its fixed ordered peer addresses and each node by its position; these are persisted and checked across restart. Shard configuration numbers and GIDs remain separate from consensus terms and membership. Host dependencies are `Endpoint::{bind,add_rpc_handler,call_timeout}`, `fs::read`, `File::{create,write_all_at,sync_all}`, mutex/task/channel mechanics, MadSim time/randomness, `bincode::{serialize,deserialize,serialized_size}`, serde and Rust/vstd/Verus/SMT. Main-task clients receive a reserved simulator IP through `NetSim::set_ip`; server-side controller clients reuse their concrete endpoint to avoid wildcard-port collisions. Host-call preconditions and publication/storage order are not fully formally refined. The pinned filesystem has immediate completed writes, no implemented power failure and a no-op `sync_all`; no real-disk power-loss or torn-write guarantee is claimed. Allocation failure may halt, and term/index/storage-generation exhaustion rejects new work. Conditional liveness remains OPEN.

Exact required reproduction commands, from the workspace root:

```bash
python3 scripts/check.py build
python3 scripts/check.py inventory
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py test
python3 scripts/check.py definitions
python3 scripts/check.py proof madraft/proof/progress.rs
python3 scripts/check.py safety
python3 scripts/check.py final --timeout 3600
```

Current command statuses are retained in `runs/delivered-status.json`; readable output is in `runs/delivered-*.console`, with exact underlying commands and diagnostics in `runs/checks/`. The recorded outcomes are build/inventory/election/tests/definitions/local proof passing, and safety/final failing on the missing aggregate proof. `runs/double-crash-before.console` preserves the real seed 1 recovery failure; `runs/double-crash-repaired.console` records its repair. `runs/snapshot-ack-before.console` and `runs/snapshot-ack-repaired.console` preserve the snapshot acknowledgment race and the passing repair. Remaining work is the complete cluster/message/history/storage invariant and executable refinement needed to discharge the unchanged independent callers.

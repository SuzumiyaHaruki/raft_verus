# Candidate result — round 6, incomplete

All lab runtime paths are implemented and the last observed shard-storage failure is repaired. Full implementation correctness and all five fixed history-wide Raft safety goals remain OPEN. The candidate has checked real log mutation/call composition contracts, but no complete execution/observation/refinement interface yet.

| Lab | Actual evidence and limits |
| --- | --- |
| Raft 2A/2B/2C | Full Raft suite passes seeds 1–5 after actual log-core refactor; global protocol invariants and durable-history refinement OPEN. |
| Raft 2D | Actual snapshot create/install/compact/recover paths implemented and tested; application fold/fidelity and complete storage bound OPEN. |
| KV 3A/3B | Runtime/retries/dedup/decoder implemented; bounded final run passes full family on seeds 1–3; all-seed stopped-candidate acceptance OPEN. |
| Shard controller 4A | Runtime historical query/join/leave/move implemented; bounded final run passes seeds 1–3; full acceptance OPEN. |
| Sharded KV 4B | Routing/freeze/import/acknowledged deletion implemented; all 12 retained non-ignored tests pass seeds 1–5; original ignored test unchanged; semantic proofs OPEN. |

| Fixed theorem / connection | Outcome |
| --- | --- |
| Election Safety | OPEN; local term/vote/tally contracts checked, global genuine-quorum/durability/history proof absent. |
| Leader Append-Only | OPEN; actual proposal call preserves exact resident prefix, but full leadership/compaction history relation absent. |
| Log Matching | OPEN; actual byte comparison/conflict mutation checked locally, cross-node/cross-time invariant absent. |
| Leader Completeness | OPEN; local real-match-slot counting/current-term commitment checked, protocol support and election preservation absent. |
| State Machine Safety | OPEN; ordered application exists, cross-history equality and snapshot application proof absent. |
| Execution link / nonempty witness | OPEN; `proof/implementation.rs` absent; all real async callers have not been composed into the fixed history. |
| Snapshot observation/fidelity | OPEN; service decoder implemented, but exact committed-prefix/application-fold semantics unproved. |
| Initialization/recovery/output connections | OPEN; partial real constructors/log callers verified, outer async/storage/recovery interpretation remains unproved. |

Round-5 changes: removed redundant prior-configuration server maps only after all imports finish; pending acknowledgments retain required addresses. Moved actual `LogEntry`, sentinel construction, exact byte equality, leader-state-based proposal, conflicting suffix truncation and received-entry caller composition into `src/raft/log.rs`, shared by Cargo and Verus. Verified failure leaves the vector unchanged, accepted proposals preserve the old prefix, and receive-entry changes preserve every already-committed resident entry. The actual async role/predecessor/provenance/storage/application contexts remain explicit gaps. Recovery uses real durable images, never an invented ghost history.

Actual checks: local Verus PASS, 41 verified / 0 errors (`runs/checks/1793336964963621990/proof.log`); full Raft and shard suites pass separately on seeds 1–5; deletion-size case passes unchanged. A bounded `final --timeout 120` run (`runs/checks/1793336938406854471`) passed build/inventory and every family on seeds 1–2 plus Raft/KV/controller on seed 3, then exhausted the reserved functional budget. It retained an intermediate callee-contract failure that was subsequently repaired and reverified. That final run is INCOMPLETE and does not aggregate acceptance across candidates. Exact final safety recheck and additional logs are in PROGRESS.md; no all-family/all-seed final pass is claimed.

Trust boundary remains MadSim endpoint/storage/scheduler/clock/randomness, mutex/channel primitives, serde/bincode value encoding, Rust memory safety and Verus/vstd/Rust/SMT. Persistent coherent `image.N` records are authoritative; current `state`/`snapshot` mirrors are accounting views, written after completed image persistence. Older completed image bytes are reclaimed, including on recovery; zero-length historical inode metadata and linear scans still grow. Simulator writes survive restart, sync is a no-op, power failure is unimplemented, and opened-inode `set_len` behavior is simulator-specific. No atomic multi-file update, real-disk guarantee, assumed protocol invariant, RPC majority premise or trusted consensus body closes a goal. Random client-ID collision handling and complete finite resource/caller bounds remain OPEN.

Reproduce with `python3 scripts/check.py proof madraft/proof/progress.rs`, `python3 scripts/check.py test raft::tests:: --seed S`, `python3 scripts/check.py test shardkv::tests:: --seed S` for S=1,2,3,4,5, `python3 scripts/check.py safety`, and a sufficiently budgeted `python3 scripts/check.py final --timeout 3600` on one stopped candidate. Next proof work is actual snapshot/durable-core constructor and recovery composition; the required connection table appears once in PROGRESS.md.

Round 6: actual snapshot creation calls checked `compact_committed`, preserving the boundary term and exact suffix contents. Actual recovery calls checked `recover_decoded`, composing vote-state reconstruction with nonempty/finite/ordered-log validation on the real decoded fields. Neither function has unchecked input preconditions. Trigger/loop-invariant proof failures were repaired; current local proof log is `runs/checks/1793337310471326313/proof.log`. Development functional results passed but intermediate checks recorded source drift during annotation edits; final reruns are recorded in PROGRESS.md. Installation authorization, decoder-to-application fold, durable-history origin and all fixed global goals remain OPEN. Prior all-seed functional results remain historical after this refactor.

Final round-6 evidence: local proof 45 verified / 0 errors; full retained Raft suite PASS on seed 1 (`runs/checks/1793337358043689426`), four additional recovery tests PASS (`runs/checks/1793337317533680414`); current safety run `runs/checks/1793337318687210336` leaves all aggregate goals OPEN while foundation/definitions/local proof PASS.

Final regression checks also PASS: full shard suite seed 1 (`runs/checks/1793337377249547730`) and KV snapshot tests seed 1 (`runs/checks/1793337377220103947`). No full all-family/all-seed acceptance is claimed.

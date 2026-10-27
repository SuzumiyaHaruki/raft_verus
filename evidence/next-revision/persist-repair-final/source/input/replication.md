# Log replication

Status: DRAFT. Module: INCOMPLETE. Only log-suffix reconciliation and its local proof/tests/replay are implemented; follower and leader protocol paths are absent.

## Responsibility and connections

Implement the full replication exchange over successive requests: leader proposal append and persistence, per-peer AppendEntries generation, heartbeat/retry service, follower term/predecessor validation, log reconciliation, durable replies and leader response processing/backtracking. Maintain current-term peer progress and supply validated durable match positions to [commitment](commitment.md).

[Elections](elections.md) establishes roles; [state](state.md) stores logs/progress; [runtime](runtime.md) orders writes and messages. Use existing MessageBody variants and the model's `propose`, `send_append`, `merged`, `receive_append`, `receive_append_response`, `higher_term`, `storage_ack` and `flush_message`. Read [model.md](model.md), [io.md](io.md) and the executable library first. Multi-entry AppendEntries is required; snapshots and performance pipelining remain excluded.

## Module completion

Both sides of replication run through the node dispatcher and handle stale terms, failed predecessor checks, duplicates, reordered responses and retries. Tests cover empty heartbeats, matching tails, conflicts, missing suffixes, committed-prefix protection, durable response timing and leader changes. Cluster fault scenarios use these actual handlers. Prove local correspondence and supporting log/term invariants feeding S-APPEND, S-LOG and S-COMPLETE; global composition remains required. `LOCAL-RECONCILE` proves only the helper's relation to `merged`. Passing the ready steps below leaves this module incomplete while protocol paths are missing.

## Step: Reconcile log contents

Stage: READY for local recheck or repair of the existing operation.

Use `reconcile_log(log: &mut Vec<Entry>, previous: usize, incoming: Vec<Entry>)` in `crates/raft-verified/src/reconcile.rs`. Reject invalid predecessor/checked length overflow without changing the log. Compare overlapping terms; at the first conflict or missing position replace the remaining suffix. Empty, fully matching or shorter matching requests preserve the existing tail. Same-term/different-payload entries retain the old entry under `merged`; changing comparison semantics needs review. Protocol positions use zero as the prefix sentinel; stored entries use zero-based offsets. The future caller checks u64-to-usize conversion, message and predecessor terms, committed-prefix compatibility and durability ordering.

Edit only the shared executable source. Preserve the protected view, proof caller and regression oracles. Rust tests, replay, benchmarks and Verus must use this same operation. Vec allocation may abort; no recoverable-OOM or resource-liveness claim is made. Read the slice tests and `crates/raft-verified/check.py` before editing.

Run `python3 -m orchestrator verify --profile interface-check`, then profiles `reconcile-tests`, `reconcile-replay`, `reconcile-proof` and `reconcile-bench`. Finish only when all required checks succeed for their respective purposes on unchanged code. Missing tools or checks block completion; failed semantics or protected-target changes require review, not weakened expectations.

## Step: Prove reconciliation correspondence

Stage: READY after the reconciliation step.

Read the shared executable source, `merged`, the protected `crates/raft-spec/src/reconcile_view.rs`, `crates/raft-proofs/reconcile.rs` caller, [proofs.md](proofs.md) and the documented build path. Add proved corollaries only in `crates/raft-proofs/reconcile_lemmas.rs`. Preserve field contents, index bounds, termination and the full result/prefix/error contract. Empty and shorter matching requests must preserve the whole old log. No stronger caller preconditions, assumptions, trusted bodies or partial-verification flags may replace the proof.

Run `python3 -m orchestrator verify --profile spec` and `python3 -m orchestrator verify --profile reconcile-proof`. The first is only a typecheck; the second must actually verify the protected local target and shared executable code. Finish with current real LOCAL-RECONCILE evidence. Record failed goals and resource limits honestly; an unproved goal is not automatically a code counterexample. A proof-only edit needs no benchmark when executable behavior and build path are unchanged. Global log matching and liveness remain open.

## Step: Replay reconciliation regressions

Stage: READY after the reconciliation step.

Add a bounded regression in `crates/raft-verified/tests/replay_cases.rs` against the actual library operation. Include matching-tail preservation for empty/shorter inputs and a conflicting suffix followed by duplicate delivery. Preserve explicit inputs, seed and before/after states; report the earliest mismatch. Existing permanent oracles and the replay harness are protected during this task.

Run `python3 -m orchestrator verify --profile reconcile-tests` and `python3 -m orchestrator verify --profile reconcile-replay`; direct reproduction is `python3 crates/raft-verified/check.py replay`. Completion requires actual nonempty execution and reproducible observations. The permanent truncation regression must reject the controlled defect and pass after repair. Scripted candidate edits and real Rust/Verus checking have separate provenance. Save failures and inspect core, adapter and expectation before attributing blame. This local replay provides no partition/crash/election coverage; those scenarios belong to recovery/integration.

## Step: Integrate follower append handling

Stage: PLANNED. Requires term handling, node state and persistence coordination.

Implement the reusable AppendEntries handler. Reject stale terms; handle higher terms and step-down under the durability contract. Validate predecessor index/term including the zero sentinel. Call the checked reconciliation helper, preserve a matching tail and the committed prefix, and limit follower commit advancement to the request-covered position and valid bounds without decreasing it. Identify any conflict with reachable-state invariants for review. Bind responses to peer, term and RPC; emit success only after the required durable acknowledgment.

Deliver handler code, term/predecessor/heartbeat/conflict tests, a crash/delayed-write trace and local `receive_append` correspondence. Configure actual checks including helper regressions before enabling this step. No follower handler profiles exist yet. Do not assume arbitrary received logs satisfy the required invariant.

## Step: Send proposals and replication requests

Stage: PLANNED. Requires leader transition and persistence coordination.

Implement leader proposal admission and durable append, per-peer next/match state, AppendEntries construction, heartbeat timers and retry scheduling. Respect request identity and Client/Noop distinction; ProposalAccepted is local receipt, not commitment. Record request/term/RPC coverage so responses can be validated. Send multi-entry suffixes from valid predecessors without overflowing indices or tokens.

Deliver shared-runtime source, proposal/heartbeat/retry tests and local `propose`/`send_append` correspondence. Supply independent profiles for these paths and existing helper regressions before activation. Leader state/scheduling and those profiles are missing. Do not introduce pipelining or new retry assumptions without reviewing the existing scope and progress contract.

## Step: Process replication responses

Stage: PLANNED. Requires follower and leader replication paths.

Handle accepted/rejected responses against the corresponding peer, term and RPC coverage. Ignore stale/duplicate/out-of-order responses that cannot justify new progress; process higher terms through step-down. Update confirmed durable match positions and bounded retry/backtracking state without inventing acknowledgments. Feed valid progress to commitment and continue sending until the peer catches up.

Deliver response handling, delayed/reordered-response and divergent-log catch-up traces, and local `receive_append_response` proofs. Run both-side replication regressions and supporting log invariants. Profiles and peer-progress code are missing today. Completing this step still requires dispatcher integration and the module-level tests/proofs above before replication can be considered complete.

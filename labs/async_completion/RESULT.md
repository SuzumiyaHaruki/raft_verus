# Async completion calibration — DRAFT

**Status A: the second minimal loop is complete under the disclosed host premises.** Real checks completed on 2026-10-31. This maintenance-authored non-Raft example is separate from `execution_boundary`; it is not autonomous experiment-agent success or complete async verification.

Run from the repository root:

```bash
python3 labs/async_completion/check.py
```

The recorded command added `--output labs/async_completion/evidence/acceptance` and exited **0**. [results.json](evidence/acceptance/results.json) records all 15 actual command outcomes, versions, arguments, working/build directories, seeds, exit codes, timings and full stdout/stderr paths; no steps were skipped. Command counts are not property coverage. The [input snapshot](evidence/acceptance/inputs/) records the exact `async-completion-0.1.0` source, tests, proof, checker and lockfile. Each run creates fresh evidence; each mutant has isolated temporary source/build directories. Missing tools, timeouts, unexpected failures or skipped steps cause nonzero exit.

| Machine-checked behavior | Actual entry and guarantee |
| --- | --- |
| Accept without Ack | [src/core.rs](src/core.rs), `Core::new`/`begin`: initial completed value 0; idle with allocatable ID must return Write and retain completed value. Begin's result type cannot contain Ack. Busy leaves Pending intact. |
| Authorize exactly matching success | `Core::complete`: arbitrary IDs and success/failure notices are legal inputs. Exactly a matching Success produces Ack using the saved request's value/source, updates completed value and clears Pending. Matching failure clears Pending without Ack/value change; absent/mismatched completion preserves state. |
| No reuse or repeat authorization | Accepted IDs are consecutive, with checked increment; IDs 0 through `u64::MAX - 1` are allocatable and MAX is the exhaustion sentinel. Ghost history proves authorized IDs strictly increase, excluding duplicate and stale authorization. No client deduplication is supplied. |
| General finite history | [proof.rs](proof.rs), `finite_history`/`any_finite_calls`: actual constructor and arbitrary sequences of begin/complete calls preserve `history_ok`. Each Ack has an earlier accepted request and a matching successful notice; completed value is the latest authorized value, or 0 if none. |

The real entries append actual accepted actions and completion input/output pairs to ghost history, with invocation indices. Rejected requests advance the invocation count without accepting an operation; all completion notices, including rejected ones, are recorded. A private type invariant connects this history to actual Pending/counter/value fields. Public entries require no caller-supplied invariant or well-matched notice. Cargo and Verus use the same core; no algorithm-selecting conditional compilation, `assume`, `admit` or skipped core body is used. The theorem concerns received notices, not independently established I/O facts.

| Real evidence | Result |
| --- | --- |
| Correct proof with `--no-cheating` | [16 verified, 0 errors](evidence/acceptance/positive-proof.stdout.log), exit 0. |
| Correct runtime checks, seeds 1 and 7 | **5 tests pass per seed**: one arithmetic boundary unit test plus four direct-core/MadSim tests. [Seed 1](evidence/acceptance/positive-tests-seed-1.stdout.log), [seed 7](evidence/acceptance/positive-tests-seed-7.stdout.log). |
| Core treats failure as success | Rust compiles (exit 0); Verus **15 verified, 1 error**, exit 1, at the state/history invariant. [Diagnostic](evidence/acceptance/core-failed-result-proof.stderr.log). |
| Core ignores completion identity | Rust compiles (exit 0); Verus **15 verified, 1 error**, exit 1. [Diagnostic](evidence/acceptance/core-wrong-id-proof.stderr.log). |
| Adapter skips writing but reports Ok | Unchanged core remains **16/0**; the actual-file assertion fails, exit **101**. [Diagnostic](evidence/acceptance/adapter-fake-success-test.stderr.log). |
| Correct version recheck | Original bytes unchanged; proof **16/0**, all **5 tests pass at both seeds**. [Proof](evidence/acceptance/restored-proof.stdout.log), [seed 1](evidence/acceptance/restored-tests-seed-1.stdout.log), [seed 7](evidence/acceptance/restored-tests-seed-7.stdout.log). |

[tests/service.rs](tests/service.rs) independently reads the actual server-node file after each client's Ack, before the next overwrite, checking the 16-byte operation ID/value record. Each simulation starts fresh. The delay test observes entry into a channel-controlled wait, holds the gate closed while checking no Ack/file, then releases it to perform real I/O. The failure test injects Err explicitly before create, checks no Ack and completed value still 7, and does not claim all failed writes leave files unchanged. Direct calls cover Busy, wrong IDs, stale/duplicate notices and repeated payloads. The module-local ID-limit test is an arithmetic fixture after ghost erasure, not a claimed execution of MAX-1 requests; Verus proves reachable-history behavior separately. Three mutation patches accompany the logs; contracts, proof entry and tests remain unchanged in those copies.

The explicitly **UNVERIFIED** boundary is [src/adapter.rs](src/adapter.rs): `serve` owns one core, decodes actual receive/source data and truthfully passes this write's result; `write_record` encodes ID/value, awaits MadSim `fs::File::create` then `write_all_at`; `send_ack` forwards the returned action unchanged. MadSim files/network/task scheduling, codecs and ordinary Verus/vstd/solver/Rust compilation assumptions remain trusted. Success means the selected create/write sequence returned Ok; a gate release or operation ID is not proof of I/O. The host mutant deliberately violates truthful reporting and is detected by file contents, not an adapter flag. Network transport must reflect actual source/bytes/destination without fabrication, including wrong outputs from wrong code.

Given faithful I/O execution and notification, core authorization follows the corresponding successful write. This is conditional reasoning across a trusted adapter, not a proof of the async adapter or filesystem. Failure may have effects; cancellation may stop forever; send error does not establish non-emission. No crash recovery, exactly-once delivery, parallel writes, liveness or Raft claim is made.

Tools: Rust **1.92.0**, Cargo **1.92.0**, Verus **0.2026.01.23.1650a05**, MadSim **0.2.34** at `519950efb4711464f300ed7edf2967ed62d5f502`; existing `tools.ini`, offline dependencies and check helpers are reused. No dependency version or global tool/configuration changed. MadSim's file calls are real simulator APIs, not physical-device durability proofs; no fsync is used.

[Preservation evidence](evidence/preservation.log) records byte-identical first-step source/proof/tests/evidence and protected root files. This lab is excluded by the unchanged experiment input allowlist. Root Git metadata remains unavailable (exit 128); `e85739c418ba2af29374ec8fa03652657b3711f7` is the supplied reference, not an asserted checkout HEAD. No old candidate, scaffold, main requirement/controller/model/budget was modified. No agent, new experiment, commit or push was invoked. No in-scope core/history obligation remains open; the disclosed host boundary remains unverified. Stop for human review.

# Execution boundary calibration — DRAFT

**Verdict A: the minimal code/proof/runtime loop is established under the explicit host premises below.** Real checks completed on 2026-10-31. This is a maintenance-authored non-Raft calibration, not an autonomous experiment success or a proof of the entire async service.

Reproduce from the repository root; the command actually runs all checks, writes a fresh evidence directory, and exits nonzero on an unexpected result, missing tool, timeout or unexecuted step:

```bash
python3 labs/execution_boundary/check.py
```

The recorded invocation used `--output labs/execution_boundary/evidence/acceptance-fixed`: exit **0**, **17/17** expected outcomes, nothing skipped. [results.json](evidence/acceptance-fixed/results.json) records exact commands, directories, relevant environment, seeds, exits, timings and full stdout/stderr logs. The runner reuses `scripts/tools.py`/`process.py`; it never invokes an agent. Mutations use temporary copies with isolated Cargo outputs; delivered source stays unchanged.

| Machine-checked claim | Actual code and proof |
| --- | --- |
| Initialization and state update | [src/core.rs](src/core.rs): `Core::new` sets actual memory to 0; `Core::handle` sets it to the received u64. `Core::value` reads that memory. |
| Ack payload and destination | `Core::handle` constructs `SendAck` in exec code with this request's value and source. These are checked postconditions, not trusted adapter guarantees. |
| Every finite processing history | [proof.rs](proof.rs): `finite_processing_history` exposes the proved history invariant; `any_finite_calls` starts with the real constructor and loops over an arbitrary input vector using the actual handler. Every returned action matches its indexed request; final memory is the last input value, or 0 for an empty history. |

The actual handler appends its input/action to ghost history. The checked constructor and handler establish/preserve a private type invariant relating history to memory. No caller-supplied protocol invariant, correct reply/history or correct-step premise is required. Cargo and Verus import the same `src/core.rs`; ghost/proof erasure leaves the executable algorithm intact. There is no `assume`, `admit`, external/skipped core body or algorithm-selecting `cfg`.

| Real check | Observed result |
| --- | --- |
| Correct proof, `--no-cheating` | **11 verified, 0 errors**, exit 0. [Proof log](evidence/acceptance-fixed/positive-proof.stdout.log). |
| Correct Cargo/MadSim tests, seeds 1 and 7 | **3 passed per seed**: direct memory/action checks (including 0, maximum u64 and duplicate inputs), two real clients, delayed observation of Ack(7) after the server has processed Put(9). [Seed 1](evidence/acceptance-fixed/positive-tests-seed-1.stdout.log), [seed 7](evidence/acceptance-fixed/positive-tests-seed-7.stdout.log). |
| Core reply changed to 0 | Rust compiles, exit 0; Verus **10 verified, 1 error**, exit 1 at the history-extension precondition. [Diagnostic](evidence/acceptance-fixed/core-reply-proof.stderr.log). |
| Core memory changed to 0 | Rust compiles, exit 0; Verus **10 verified, 1 error**, exit 1 at the constructed state's type invariant. [Diagnostic](evidence/acceptance-fixed/core-state-proof.stderr.log). |
| Adapter changes payload to value + 1 | Unchanged core proof still **11/0**; actual MadSim test fails, exit 101: client A receives 8 instead of 7. [Diagnostic](evidence/acceptance-fixed/adapter-value-test.stderr.log). |
| Adapter sends all replies to B | Unchanged core proof still **11/0**; actual MadSim test fails, exit 101: B receives A's 7 instead of 9. [Diagnostic](evidence/acceptance-fixed/adapter-recipient-test.stderr.log). |
| Correct source recheck after mutations | Byte-identical input files; proof **11/0**, exit 0; all **3 tests pass at each seed**, exit 0. [Proof](evidence/acceptance-fixed/restored-proof.stdout.log), [seed 1](evidence/acceptance-fixed/restored-tests-seed-1.stdout.log), [seed 7](evidence/acceptance-fixed/restored-tests-seed-7.stdout.log). |

Four recorded patches change only executable statements; specs, proof callers and tests stay fixed. Checks are real, not simulated. Mutation testing is not complete. Adapter defects escape the core proof because [src/adapter.rs](src/adapter.rs) is outside its verified unit.

| Public host premise — not machine-proved | Concrete boundary |
| --- | --- |
| Receive source and bytes reflect an actual delivered message; send uses the supplied destination, tag and bytes | Pinned MadSim `Endpoint::bind`, `recv_from`, `send_to`. Transport may delay, lose, reorder or duplicate real messages, but does not fabricate/modify them. Wrong program outputs remain possible; the premise never says replies are correct. |
| Correct IPv4 and u64 conversion | `adapter::source_peer`, `address`, and little-endian encoding/decoding in `serve`/`send_action`. Put uses tag 1, Ack tag 2; non-eight-byte requests and IPv6 sources are rejected. |
| Exclusive, faithful driving and forwarding | `adapter::serve` owns one `Core`, initializes it once, calls `handle` once per decoded input using its received source, then `send_action` forwards the returned destination and value unchanged. These async control/data paths are reviewed and tested, not verified. |
| Ordinary tool/runtime trust | Verus/vstd/solver, Rust compilation and ghost erasure, safe-Rust memory semantics, MadSim endpoint/task runtime and standard-library conversions. No async executor, network implementation or codec proof is supplied. |

Under these premises, emitted/delivered server Acks inherit the proved action's value/recipient. This is conditional reasoning across the trusted adapter, not a machine-checked theorem about `serve`. Send failure stops processing without undoing Put and does not imply non-emission. Loss/cancellation/suspension gives no eventual-reply guarantee. The delayed test observes Ack(7) while memory is 9. Repeated requests may be processed again. Durability, recovery, shared-state concurrency, exactly-once behavior and liveness are outside scope.

Tools: Rust **1.92.0** (`ded5c06cf`), Cargo **1.92.0** (`344c4567c`), Verus **0.2026.01.23.1650a05**, MadSim **0.2.34** at `519950efb4711464f300ed7edf2967ed62d5f502`. Existing configuration, offline dependencies and vstd are reused; no global changes or dependency-version upgrades. The lab-local [Cargo.lock](Cargo.lock) only reduces unused `futures-util` dependency edges.

[Development diagnostics](evidence/development/) and the [first full run](evidence/acceptance/results.json) remain available. That run exited 1: shared Cargo outputs reused a mutant binary, and the runner did not recognize a type-invariant diagnostic. Build isolation and diagnostic handling were fixed, then all checks rerun; old failures were not relabeled.

Changes are confined to this lab. [Byte comparisons](evidence/preservation.log) found the scaffold, requirements, scripts, docs and root configuration/documents unchanged. Old experiments were untouched; the input allowlist excludes this lab. Git metadata is unavailable (exit 128), so HEAD cannot be confirmed against reference `0d84789e234206f9ca1b276f7d665eb6995a09f5`. No experiment, reset, commit or push was performed. The remaining unproved connection is the disclosed async/host boundary; the core/history proof is closed. Broader Raft obligations remain unaffected. Stop for review.

# MadSim hosting the Verus executable — DRAFT

This bounded experiment uses modern MadSim to run the actual `raft-verified` library. The current verdict is **PARTIAL — RUNTIME ONLY**, because the RequestVote and completion paths use dedicated methods instead of the shared `Node::step` interface. The local executable proofs do pass; the verdict does not mean that only Cargo was checked. Read the [review report](../../review/MADRAFT_FEASIBILITY_REPORT.md) for actual results and limitations.

## Versions and compatibility

| Source/tool | Checked selection | Role |
|---|---|---|
| Repository comparison | `f7cf8eb727b0b37a625dbc669ac764c5c2bcace2` | All upstream-tracked non-evidence authored files matched before editing; local HEAD is unavailable because the supplied `.git` is empty and read-only. |
| [MadRaft](https://github.com/madsim-rs/MadRaft/tree/8ec4565ecd3844f54f52865f2d7b3da4044f657f) | `8ec4565ecd3844f54f52865f2d7b3da4044f657f`, Apache-2.0 | Reference test host; its `madsim = "0.1.1"` resolves to 0.1.3 in the disposable build. |
| [MadSim](https://github.com/madsim-rs/madsim/tree/519950efb4711464f300ed7edf2967ed62d5f502) | `519950efb4711464f300ed7edf2967ed62d5f502`, package 0.2.34, Apache-2.0 | Actual runtime dependency; git revision and Cargo.lock are pinned. |
| [raft-rs RawNode/Ready](https://github.com/tikv/raft-rs/blob/ad13f3d90780f53aea2488c6a4b76c0d334bf136/src/raw_node.rs) | `ad13f3d90780f53aea2488c6a4b76c0d334bf136` | API comparison only: separate synchronous protocol progress from storage and sending. No implementation imported. |
| [etcd Raft integration contract](https://github.com/etcd-io/raft/blob/c51f010c851edd0dada5af41bafbf68e54617767/doc.go) | `c51f010c851edd0dada5af41bafbf68e54617767` | API comparison only: Step/Tick and durable Ready handling before message release. No implementation imported. |
| Rust/Cargo | 1.92.0, `x86_64-unknown-linux-gnu` | Installed isolated tools selected through workflow.toml; system default 1.88 is unchanged. |
| Verus/vstd | `0.2026.01.23.1650a05`, bundled vstd | Same Rust 1.92 toolchain, whole proof entrypoint with `--no-cheating`. |
| Z3 | 4.12.5 | Bundled verifier solver. |

The original MadRaft build failed before protocol tests: MadSim 0.1.3 transmutes Rust's 128-bit TypeId into u64, producing E0512 under Rust 1.92. Its README already requires rustc below 1.72 for that dependency line. We did not downgrade the core or install an old compiler. The modern probe imports the actual `raft-verified` crate, sends through MadSim Endpoint, exercises timeouts, packet loss and node kill, and passes. The selected arrangement is work-package A option B: modern MadSim with a narrowly adapted host design. No complete MadRaft lab or working external Raft implementation was copied.

The old tester uses a shared runtime handle's network controls and the old RPC surface. The new adapter uses `Runtime::with_seed_and_config`, `Handle::create_node`, node-scoped tasks, `Endpoint::{bind,send_to,recv_from}`, `NetSim::update_config`, `time::sleep` and `Handle::kill`. It uses a small explicit binary message codec rather than porting the old RPC macro interface. Both endpoints run inside actual MadSim node contexts. Compilation requires `--cfg madsim`; the host refuses a normal non-simulation build.

## One executable and explicit interfaces

`experiments/madraft-poc` is a separate Cargo workspace with a path dependency on `crates/raft-verified` and its opt-in `madraft-poc` feature. `check.py` reuses `cargo_environment` from the existing core checker to select installed Rust and erase proof syntax using the same bundled vstd. Verus's independent entrypoint imports the exact `crates/raft-verified/src/madraft.rs`, Node, observation and I/O source files by path. There is no generated Rust translation or separate protocol engine. Existing controller source bindings record the inputs; no new hashing system was added.

| Method/type | Actual behavior |
|---|---|
| `Node::feasibility_follower` | Construct a valid fresh follower; reject empty, duplicate or missing-self membership. Its proof uses the original independent initialization relation. |
| `Node::handle_request_vote` | Validate identity, term, freshness and vote eligibility; return a reject reply or a persistence action. Never release an immediate grant. |
| `VoteAction::Persist { operation, retry }` | The host obtains the current intended hard state through `observe()`. `retry=true` explicitly requests redelivery after higher-term persistence, without a host-side vote decision. |
| `Node::finish_vote` | Check the exact operation token and completion kind; release a pending response only on matching Success; use the original protected completion relation. Failed or uncertain writes require recovery and release no grant. |

The intended sequence is network receive → dedicated vote method → persist intended image → matching dedicated completion → network send. Higher-term requests first persist the higher term and cleared vote with no response; the host redelivers the retained request only after `Ok(None)` with `retry=true`. A qualifying grant then needs its own durable write and completion. Repeated same-candidate requests may persist again, as permitted by the existing model. Errors are traced and end that request; Busy requests are not buffered, so an external retry is needed. The core locks are released before every await.

The original `Node::new`, `submit_persist`, `complete_persist` and `Node::step` seeds and all their protected targets remain unchanged. The timer test delivers a real simulated timeout to `Node::step` and observes `UnsupportedEvent`; it does not implement election timers. The dedicated completion method satisfies the old completion specification but does not repair the unfinished original completion implementation. Unifying these paths is the next interface task, not a hidden behavioral migration.

## Proof scope and host premises

`crates/raft-spec/src/madraft_vote.rs` is an additive DRAFT behavioral target, separate from the executable. Protected callers in `crates/raft-proofs/madraft_vote.rs` check the constructor, exact vote handler and exact completion method. Local membership loops and representation facts are ordinary proof helpers. The caller requires the established representation predicate plus valid membership, a member-valued vote and well-correlated pending state; it does not assume fresh candidates, successful storage, matching callbacks or a fault-free network. The constructor establishes these conditions and both handlers preserve them. `from_observation` remains only a boundary adapter; arbitrary injected observations do not establish this stronger protocol invariant or reachable-state correctness.

The targets prove input-error/no-mutation behavior, lexicographic log freshness, one eligible candidate per unchanged term, exact term/vote/pending updates, no immediate positive reply and release only after a matching successful persistence completion. `check_freshness_projection` connects the freshness comparison to `model::term_at` through the existing log view. Completion directly uses the existing `persistence_spec::complete_result` rather than a substitute relation.

This is not a proof of `model::receive_vote_request` or `higher_term` as complete cluster transitions. The ordering and durable term/vote projections agree with those transitions, but the new slice does not reset the election deadline, model the cluster network/outbox, clear leader flight/vote collections, or prove a finite-step cluster simulation. Those features are absent from this small executable representation. All corresponding refinement and liveness obligations remain OPEN; the existing full model and assumptions are unchanged.

The host is trusted/tested for transport identity, codec correctness, truthful completion, atomic whole-image storage, source wiring and no sends except returned core replies. The simulated store is a map outside the killed node; this is an explicit fault model, not a verified filesystem. At the pinned MadSim revision, [FsSim power_fail is TODO and sync_all has no durability model](https://github.com/madsim-rs/madsim/blob/519950efb4711464f300ed7edf2967ed62d5f502/madsim/src/sim/fs.rs). We therefore do not claim that MadSim's filesystem establishes a real disk's crash guarantees. Unknown storage completion may follow an actual simulated write; NotPerformed leaves the disk unchanged. Both return no grant. Killing the node after the write cancels the callback; no restart is attempted.

The test network is non-Byzantine and identities 1 and 3 are message injectors, not verified candidates. A term-zero injected vote is a handler-boundary input, not an autonomous election trace. The nonempty-log scenario is explicitly tagged `BoundaryFixture`; it is not evidence of legal recovery or log replication. Invalid wire input or transport errors fail the test task; this host is not a robust production network adapter. Fixed-seed repeatability applies to this pinned configuration, not all systems or scheduler versions.

## Commands and evidence

From the repository root, run these existing-controller commands. They perform no agent generation or interactive approvals:

```sh
python3 -m orchestrator validate
python3 -m orchestrator verify --profile madraft-poc-build
python3 -m orchestrator verify --profile madraft-poc-runtime
python3 -m orchestrator verify --profile madraft-poc-fault
python3 -m orchestrator verify --profile madraft-poc-verus
python3 -m orchestrator verify --profile madraft-poc-negative
```

The runtime profile runs eight tests: seven actual MadSim scenarios/test groups and one direct-core boundary test. The fault profile selects three of those tests. Traces between `TRACE_BEGIN` and `TRACE_END` record decisions, writes, matching completions, sends, delivered replies and faults. Fixed-seed 771 runs twice and compares entire typed protocol traces, omitting logging timestamps. The negative profile edits only a disposable copy, labels the mutation SIMULATED, requires both actual Rust barrier rejection and actual Verus rejection of the handler, restores the source, and requires both checks to pass. Historical and final raw outputs remain separate.

Profiles use locked, offline Cargo by default. The pinned dependencies were downloaded into the repository's ignored `target/madraft-cargo`, never the user's global toolchain. On a fresh machine with the configured compatible tools already installed, explicitly populate that cache with `python3 experiments/madraft-poc/check.py fetch`; then the five checks can run offline. Missing tools are BLOCKED, not successful proof evidence. A missing offline dependency causes a real Cargo failure and must be fetched before rechecking; it is not silently replaced.

The standalone compatibility probe is `python3 experiments/madraft-poc/check.py compat`. To reproduce the original incompatibility, clone the pinned MadRaft revision into a disposable directory and run its `cargo test --no-run` using the selected Rust 1.92, as follows. `MADRAFT_CHECKOUT` must refer to that disposable checkout; this does not change system defaults.

```sh
git clone https://github.com/madsim-rs/MadRaft /tmp/madraft-original-repro
git -C /tmp/madraft-original-repro checkout --detach 8ec4565ecd3844f54f52865f2d7b3da4044f657f
export MADRAFT_CHECKOUT=/tmp/madraft-original-repro
python3 - <<'PY'
import os, subprocess
from pathlib import Path
from orchestrator.config import load_config
root = Path.cwd()
selected = load_config(root)['toolchain']['resolved']
env = os.environ.copy()
env.update(selected['environment'])
env.update(RUSTC=selected['paths']['rustc'], RUSTDOC=selected['paths']['rustdoc'], CARGO_HOME=str(root/'target/madraft-cargo'), CARGO_TARGET_DIR=str(root/'target/madraft-original'))
raise SystemExit(subprocess.run([selected['paths']['cargo'], 'test', '--no-run'], cwd=os.environ['MADRAFT_CHECKOUT'], env=env).returncode)
PY
```

The original attempt's resolved lockfile and command/environment record are in `evidence/madraft-feasibility/original.Cargo.lock` and `original.json`; copy that lockfile into the disposable checkout and add `--locked --offline` to reproduce its exact dependency selection after fetching it. The unfinished original lab tests are not expected to pass even on a compatible old compiler and were not executed here.

## Experiment provenance and next step

The optional vote slice is MAINTENANCE_AUTHORED, explicitly requested for this feasibility task. Its default feature is off, so the STATE-INIT and other original seeds still fail as intended. The new feasibility files are visible in ordinary candidate copies: disabling a Cargo feature is not information isolation. A future run starting here must declare these supplied implementations or explicitly prepare an isolated experiment input without them before claiming from-zero generation. No such real generation was run in this revision.

The smallest next technical task is a reviewable shared event/effect extension: route Receive and Complete through the same Node entrypoint, preserve the old completion/error semantics, and rerun this host against that interface. Then address the missing deadline update and a narrow finite-step refinement to the existing model. Do not infer full elections, restart correctness, global safety or conditional liveness from these local checks.

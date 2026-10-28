# MadRaft/MadSim × Verus feasibility — DRAFT

Delivered 2026-10-28 for human review. **Verdict: PARTIAL — RUNTIME ONLY.** Real MadSim hosts the actual `raft-verified` RequestVote executable, deterministic fault tests pass, the same-source local Verus proof passes, and a controlled defect is rejected by both. The shared public `Node::step` still does not process RequestVote or ordered effects: this bounded experiment uses explicit dedicated methods, so the requirements' GO criterion is not met. This is neither full Raft nor evidence of successful LLM generation.

## Source, compatibility and tools

The supplied `.git` directory is empty and read-only, so actual local HEAD could not be recovered. Before edits, every remote-tracked, non-evidence authored file matched `SuzumiyaHaruki/raft_verus` at `f7cf8eb727b0b37a625dbc669ac764c5c2bcace2`; this is a content comparison, not an assertion that the local checkout has that HEAD. [Inspection](../evidence/madraft-feasibility/source-inspection.json), [backup location](../evidence/madraft-feasibility/backup-path.txt), [change audit](../evidence/madraft-feasibility/audit.json) and [source diff](../evidence/madraft-feasibility/changes.patch) retain provenance. Existing user work, default seeds, old targets and historical evidence were preserved.

| Source/tool | Actual selection and result |
|---|---|
| MadRaft | Checked commit `8ec4565ecd3844f54f52865f2d7b3da4044f657f`, Apache-2.0. Its `madsim = "0.1.1"` resolved to 0.1.3. The disposable original `cargo test --no-run` failed with E0512: TypeId is 128 bits but the old RPC code transmutes it to u64. This is toolchain incompatibility, before unfinished lab tests. [Command](../evidence/madraft-feasibility/original.json), [raw failure](../evidence/madraft-feasibility/original.stderr.log), [resolved lockfile](../evidence/madraft-feasibility/original.Cargo.lock). |
| Modern MadSim | Checked/pinned commit `519950efb4711464f300ed7edf2967ed62d5f502`, package 0.2.34, Apache-2.0. Actual network, clock, loss and node-kill probe linked and invoked the existing core before protocol development. [Original probe output](../evidence/madraft-feasibility/modern.stdout.log), [command](../evidence/madraft-feasibility/modern.json), [final probe](../evidence/madraft-feasibility/final/compat.stdout.log). |
| API comparisons | raft-rs `ad13f3d90780f53aea2488c6a4b76c0d334bf136`, `src/raw_node.rs`; etcd Raft `c51f010c851edd0dada5af41bafbf68e54617767`, `doc.go`. Used only to compare synchronous protocol/I/O separation and persistence-before-send ordering. No working Raft implementation imported. |
| Host platform | Linux 6.8.0-138-generic, x86_64, glibc 2.35. [Actual environment](../evidence/madraft-feasibility/host-environment.json). |
| Rust/Cargo | Installed isolated Rust 1.92.0 (`ded5c06cf21d2b93bffd5d884aa6e96934ee4234`), Cargo 1.92.0; target `x86_64-unknown-linux-gnu`. System Rust/Cargo 1.88 remain unchanged. |
| Verus/vstd/Z3 | Verus `0.2026.01.23.1650a05`, matching bundled vstd and Rust 1.92; Z3 4.12.5. [Executed version commands](../evidence/madraft-feasibility/tools.json). No required check tool was missing. |

Selected arrangement: work-package A option B, modern MadSim with a narrowly adapted MadRaft host design. The original lab was not copied wholesale. The new adapter replaces the old RPC surface with node-scoped Endpoint transport and explicit encoding, while borrowing its three-node fault-testing structure. [Architecture, pinned source links, reproduction commands and TCB](../docs/experiments/madraft-poc.md); [retained attribution/license](../experiments/madraft-poc/NOTICE.md). All named upstream references were accessible. The isolated Rust installation lacks rustfmt, so only formatting used the already installed system rustfmt 1.8.0; compilation and proof used 1.92 throughout.

## Delivered files and actual behavior

| Files | Role |
|---|---|
| [madraft.rs](../crates/raft-verified/src/madraft.rs), crate feature/export | The sole executable vote implementation, using the existing Node/Observation/I/O vocabulary. Adds `feasibility_follower`, `handle_request_vote`, `finish_vote` and `VoteAction`; enabled only by the opt-in `madraft-poc` feature. |
| [independent target](../crates/raft-spec/src/madraft_vote.rs), [proof caller](../crates/raft-proofs/madraft_vote.rs) | Additive DRAFT vote/order/invariant targets and real executable callers, plus freshness projection to the model. Original initialization and persistence relations are reused unchanged. |
| [host](../experiments/madraft-poc/src/lib.rs), [compatibility probe](../experiments/madraft-poc/src/bin/compat.rs), [tests](../experiments/madraft-poc/tests/scenarios.rs) | Real MadSim transport/time/crash, simulated durable store, typed traces, independent expected results and fixed-seed comparison. No host-side vote eligibility calculation or fabricated grant. |
| [Cargo manifest](../experiments/madraft-poc/Cargo.toml), Cargo.lock | Separate host workspace, exact MadSim git revision and dependency lock, path dependency on the actual Verus-backed core. |
| [host checker](../experiments/madraft-poc/check.py), [negative control](../experiments/madraft-poc/negative.py), [core checker](../crates/raft-verified/check.py), [workflow.toml](../workflow.toml) | Reuse the existing Python controller and installed-tool environment. Five implemented check profiles; disposable source mutation and repair; no new orchestration framework. |
| README, input/starting-point.md, Agent.md, docs and REVIEW_PACKET.md | Disclose the bounded maintenance-code exception, unchanged default seeds, future generation provenance and current review entrypoint. |

Three simulated identities are created. Identities 1 and 3 inject test-controlled messages; identity 2 executes the actual core. Positive replies are withheld until the core sees a matching successful persistence completion. Higher terms first persist the term/reset-vote state and then explicitly request redelivery; the grant requires a second persistence cycle. Duplicate same-candidate requests, competing candidates, old terms, freshness, unknown members, wrong destination, exhausted tokens, mismatched/duplicate callbacks and failed/unknown writes have observable tested behavior. Busy/error results are recorded and end that request; the adapter does not fabricate success or automatically repair storage failure.

The old constructor, persistence, reconciliation and general dispatcher seeds were not replaced. `Node::step` receives a simulated timer and returns `UnsupportedEvent`, which the test asserts. No complete election, shared effect dispatcher or verified restart was added. The synthetic log test is explicitly marked as a boundary fixture, not a reachable cluster history.

## Actual commands and results

All commands below ran from the repository root. Check profiles are REAL_LOCAL. The only SIMULATED generation is the deliberate scripted defect/repair; the delivered slice itself is MAINTENANCE_AUTHORED. No Codex task, subagent, recursion, merge, push or publication was performed. [Final command/exit-code record](../evidence/madraft-feasibility/final/commands.json).

| Exact command | Actual result | Retained evidence |
|---|---|---|
| `python3 -m orchestrator validate` | PASS, structural only | [JSON](../evidence/madraft-feasibility/final/validate.stdout.log) |
| `python3 -m orchestrator verify --profile madraft-poc-build` | PASS, real dependency build | [Result](../evidence/madraft-feasibility/final/madraft-poc-build.stdout.log), [Cargo output](../evidence/madraft-feasibility/final/madraft-poc-build/madraft-poc-build.stderr.log) |
| `python3 -m orchestrator verify --profile madraft-poc-runtime` | PASS, 8 tests: 7 MadSim test groups and 1 direct-core boundary test | [Result](../evidence/madraft-feasibility/final/madraft-poc-runtime.stdout.log), [tests and traces](../evidence/madraft-feasibility/final/madraft-poc-runtime/madraft-poc-runtime.stdout.log) |
| `python3 -m orchestrator verify --profile madraft-poc-fault` | PASS, 3 selected tests; seed 771 run twice yields identical typed traces | [Result](../evidence/madraft-feasibility/final/madraft-poc-fault.stdout.log), [fault traces](../evidence/madraft-feasibility/final/madraft-poc-fault/madraft-poc-fault.stdout.log) |
| `python3 -m orchestrator verify --profile madraft-poc-verus` | PASS, 33 verified / 0 errors, whole entrypoint and required functions | [Per-function result](../evidence/madraft-feasibility/final/madraft-poc-verus.stdout.log), [raw Verus output](../evidence/madraft-feasibility/final/madraft-poc-verus/madraft-poc-verus.stdout.log) |
| `python3 -m orchestrator verify --profile madraft-poc-negative` | PASS: wrong early grant fails the barrier test and proof; restored source passes both | [Result](../evidence/madraft-feasibility/final/madraft-poc-negative.stdout.log), [four real results and mutation provenance](../evidence/madraft-feasibility/final/madraft-poc-negative/negative-control/summary.json) |
| `python3 experiments/madraft-poc/check.py compat` | PASS, actual core invocation/network/clock/loss/kill | [Output](../evidence/madraft-feasibility/final/compat.stdout.log) |
| `python3 -m unittest discover -s orchestrator/tests -v` | PASS, 119 tests, no skips | [Controller regression](../evidence/madraft-feasibility/controller-regression.log) |
| `python3 -m orchestrator verify --profile interface-check` | PASS after changes | [Result](../evidence/madraft-feasibility/regression-final/interface-check.json) |
| `python3 -m orchestrator verify --profile node-tests` | FAIL as expected for preserved seed: 4 passed / 4 failed | [Actual tests](../evidence/madraft-feasibility/regression-final/node-tests/node-tests.stdout.log) |
| `python3 -m orchestrator verify --profile node-proof` | FAIL as before: 23 verified / 1 error | [Result](../evidence/madraft-feasibility/regression-final/node-proof.json) |
| `python3 -m orchestrator verify --profile persist-tests` | FAIL as expected for preserved seed: 0 passed / 6 failed | [Actual tests](../evidence/madraft-feasibility/regression-final/persist-tests/persist-tests.stdout.log) |
| `python3 -m orchestrator verify --profile persist-proof` | FAIL as before: 29 verified / 3 errors | [Result](../evidence/madraft-feasibility/regression-final/persist-proof.json) |

The regression suite also checks the original completed initialization/persistence fixture paths with real Rust/Verus; those fixtures remain SIMULATED. The default seed failures above are retained rather than hidden behind the new feature. [Initial checks](../evidence/madraft-feasibility/initial-checks.log) and the byte-preservation audit distinguish them from regressions. The unchanged global suite and full original MadRaft lab were not rerun or claimed complete.

The negative copy returns a positive VoteAction immediately after creating a pending write. Actual Rust returns exit 101 with the `persistence_barrier` test failed; actual Verus reports **32 verified / 1 error**, specifically `Node::handle_request_vote`. After restoring the original source in that same disposable copy, the barrier test passes and Verus returns **33 / 0**. [Defect test output](../evidence/madraft-feasibility/final/madraft-poc-negative/negative-control/defect.stdout.log), [proof failure](../evidence/madraft-feasibility/final/madraft-poc-negative/negative-control/defect/madraft-poc-verus.stderr.log), [restored test](../evidence/madraft-feasibility/final/madraft-poc-negative/negative-control/repair.stdout.log). The main source stayed unchanged. Earlier proof attempts are retained under `vote-first`, `vote-second` and `vote-third`; the constructor initially needed explicit membership/log-view facts, not a weakened target.

## What the proof establishes

The exact proof command is `verus --crate-type lib --no-cheating --output-json --time-expanded --triggers-mode silent crates/raft-proofs/madraft_vote.rs`, using the resolved executable path in the check result. The controller checks the entire entrypoint and individual required targets, not just a verified count. Counts include dependencies and derives; they are not 33 Raft invariants.

- `madraft_vote::check_follower` calls the actual constructor and establishes the original valid fresh initialization target plus the slice invariant.
- `madraft_vote::check_vote` calls `Node::handle_request_vote` and proves the independent input-error, term, eligibility, freshness, vote/pending update and no-early-grant relation while preserving representation and slice invariants.
- `madraft_vote::check_durable_reply` calls `Node::finish_vote`, proves the unchanged `persistence_spec::complete_result`, and requires a matching token, Persist kind and Success for every returned reply.
- `madraft_vote::check_freshness_projection` connects the executable observation's freshness predicate to the existing model's log projection and `term_at`.

The executable methods themselves are verified in the same entrypoint and reported as `madraft_vote::node_state::Node::{feasibility_follower,handle_request_vote,finish_vote}`. Host Cargo and Verus use the identical `crates/raft-verified/src/madraft.rs`, whose existing-controller digest is `210d44f189da3857e0e06d6244f26909047b127b8f574fee8e2ba55f8082f04a`. [Shared source manifest and recomputed final check bindings](../evidence/madraft-feasibility/same-source.json) confirm source/profile correspondence. Different profile digests are expected because the existing binding includes the profile. No new hash, frozen contract or approval mechanism was introduced.

## Limits and pending decisions

All 26 global obligations in [input/proofs.md](../input/proofs.md) remain OPEN. Global Election Safety, other Raft safety properties, conditional liveness, complete recovery, cluster refinement, infinite executions and production durability are not established. No benchmark or performance target was added to this compatibility task. All new specifications, methods and targets remain DRAFT for review.

| Pending item | Current bounded choice; work still required |
|---|---|
| Shared interface | Dedicated vote/completion methods are smaller and preserve the old protected dispatcher semantics. Review a shared Receive/Complete/effect extension; do not silently change `dispatch_result`. This alone prevents a GO verdict. |
| Full model correspondence | Local vote/term rules match selected projections; freshness is formally connected and completion uses the existing relation. Election deadline reset, cluster packet consumption/outbox and leader bookkeeping are absent. A finite-step simulation of `receive_vote_request`/`higher_term` is OPEN. |
| Storage and crash boundary | Host storage is a tested/trusted atomic whole-image map outside the killed node. At pinned MadSim, filesystem power-failure modeling is TODO. Tests cover write-before-callback crash and no fabricated grant; real fsync, torn writes and restart reconstruction remain OPEN. |
| Identity and retries | Transport source is checked against the message sender under a non-Byzantine host premise. RPC identifiers are correlated, not authenticated peer-incarnation histories. Busy requests are dropped with a recorded error; fairness/retry progress is OPEN. |
| Error/recovery semantics | Failed or Unknown completion is fail-stop, reusing the old independent target. No recovery algorithm or approval of the existing pending storage/recovery decisions is implied. |
| Generation provenance | The optional implementation remains visible in ordinary candidate copies. Future studies must declare it as supplied code or explicitly isolate it before claiming from-zero generation. Feature disabling alone does not prevent answer exposure. No real agent run occurred. |
| Trusted boundary | Rust/compiler, Verus/vstd/Z3, checker tooling and the host's codec, identity, atomic store and truthful completions remain trusted. Same-source local proofs do not verify async scheduling, filesystem or network adapters. |

## Next runnable work

Rerun any of the five registered profiles above independently; they use locked offline dependencies already cached under `target/madraft-cargo`. A fresh environment first needs the configured compatible tools and explicit `python3 experiments/madraft-poc/check.py fetch`. Missing tools/dependencies must be reported, never substituted with fake checks. [Full commands and original incompatibility reproduction](../docs/experiments/madraft-poc.md#commands-and-evidence).

The smallest next development assignment is to unify this tested Receive/Persist/Complete/Send path with the shared `Node::step` interface while preserving existing completion/error behavior, then rerun the same simulation and independent proofs. The deadline/refinement gap needs its own explicit target and proof. This report requests review of those interface/semantic choices; it adds no approval files or checkpoints to workflow execution. Work stops here for human review.

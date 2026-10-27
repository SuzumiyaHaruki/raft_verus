# Sources and toolchain observations — DRAFT

Source review date: 2026-10-26. References support design choices, not project correctness. No upstream Raft/Anvil implementation or proof code was copied. Following explicit user authorization, official Verus and isolated Rust were downloaded and actually checked with the model. verus-tla was not integrated; an upstream commit observation is not an approved version pin. Upstream observations below are historical. The missing temporary tool installation was restored on 2026-10-27 to `/home/nitro/.local/share/raft-verus/`; see [tool setup](tools.md) for current local records.

| ID | Source and inspected material | Use and status |
|---|---|---|
| S1 | [OpenAI non-interactive mode](https://developers.openai.com/codex/non-interactive-mode): JSONL, structured outputs, stdin | Official page read; CLI adapter design. Installed help determines actual support |
| S2 | [CLI reference](https://developers.openai.com/codex/cli/reference) | Ran codex --version, --help and exec --help to calibrate arguments; no generation task |
| S3 | [Configuration reference](https://developers.openai.com/codex/config-reference) | Not independently read in full; sandbox/approval are explicit, user model/authentication not assumed |
| S4 | [AGENTS.md](https://developers.openai.com/codex/agent-configuration/agents-md) | Not independently read in full; instruction-file reference, not an isolation mechanism |
| S5/S6 | [etcd/raft doc.go](https://github.com/etcd-io/raft/blob/main/doc.go): Usage, Node.Ready/Advance, Propose | Read for core/I/O separation, persist/send ordering and retries. No import or implementation pin; upstream Apache-2.0 |
| S7 | [Verus modes](https://verus-lang.github.io/verus/guide/modes.html) | Read for spec/proof/exec separation; modes need not be separate crates |
| S8 | [Verus TCB](https://verus-lang.github.io/verus/guide/tcb.html) | Read for trust-entry scanning; scanning is not a semantic safety proof |
| S9 | [verus-tla commit](https://github.com/anvil-verifier/verus-tla/tree/84487a0ef8c01e79fa6aaa035476d0970775bac1): README, Cargo.toml, src/defs.rs, CI | Read and remote commit inspected; not installed or compatibility-checked; BLOCKED |
| S10 | [Ongaro raft.tla](https://github.com/ongardie/raft.tla/blob/master/raft.tla): Quorum, elections, allLogs, Init | Read for history and strict-majority design. Not translated/imported; CC-BY-4.0; not this project's proof |
| S11 | [Anvil](https://github.com/anvil-verifier/anvil) | Build/proofs not independently checked; retained as a method reference, not evidence of verified Raft |
| S12 | [LLMs for Verus](https://verus-lang.github.io/verus/guide/llmforverusproof.html) | Not independently read in full; no generated-code verification claim based on it |
| S13 | [Ponytail AGENTS.md](https://github.com/DietrichGebert/ponytail/blob/main/AGENTS.md) | Read for minimal complete changes, reuse/standard library, boundary validation and actual checks; no code import |
| S14 | [Inductive Deductive Synthesis: Enabling AI to Generate Formally Verified Systems](https://arxiv.org/html/2605.23109v1), Agarwal et al., 2026; sections 2–4, 6 and Appendix F consulted on 2026-10-27 | Joint incremental implementation/proof synthesis, diagnostic-guided redesign and performance feedback. Main distributed-system experiments use supplied Rocq KV specifications; this is not a demonstrated Rust/Verus Raft or liveness result. No code imported. |

IDS leaves specification adequacy as an input problem. Its partial Rocq states may
contain deferred holes, with a separate final audit; our Verus workflow keeps the
existing no-assume/no-admit rule and reports unfinished obligations as OPEN.
The prompt/roadmap adopt the joint-development method. Automated strategic search,
the paper's agent architecture and Raft foundations are not implemented by that
documentation change; no new agent platform or runtime review gate is introduced.

## verus-tla compatibility

Observed remote commit: `84487a0ef8c01e79fa6aaa035476d0970775bac1`. Cargo package: `verus_temporal_logic`, version `0.1.0`, MIT. `defs.rs` provides `Execution<T>` with `nat_to_state: spec_fn(nat) -> T`, `TempPred`, `lift_state`, `lift_action`, `always`, `eventually`, `weak_fairness`.

Cargo.toml references a Verus Git branch without a compatible commit pin. README uses `cargo verus verify --lib`; observed CI uses `cargo verus focus --lib -- --time` and builds Verus main. This difference needs local calibration, not an arbitrary command followed by a verification claim. The current model uses explicit quantified trace definitions, not a substitute axiom library. Integration of temporal operators/lemmas remains ADR-08 and open proof work.

## Local observations

Actual versions and availability are recorded under `evidence/bootstrap/` and `evidence/model-*`. Initial observations: Python 3.10.12, rustc/cargo 1.88.0, codex-cli 0.162.0-alpha.17.2; Verus/Z3 initially absent. Downloads followed the user's explicit instruction. Read-only permissions caused a Codex PATH-alias warning, while version/help returned zero. Authentication content was not read and system defaults were not changed.

Python uses installed jsonschema 3.2.0 beyond the standard library; TOML compatibility is described in README. These are observations, not approval. No new proof axioms, third-party Raft Rust crates or runtime Raft backend were introduced.

The authorized [official Verus release](https://github.com/verus-lang/verus/releases/tag/release%2F0.2026.01.23.1650a05) has commit `1650a05d64219fdd1d150f32d0d7d5b19b8defcd`, with bundled vstd and Z3 4.12.5. It was installed under `/tmp/raft-verus-tools/compatible/verus-x86-linux/`. Rust 1.92.0 came from `static.rust-lang.org` into isolated RUSTUP_HOME/CARGO_HOME. The first newer download was incompatible with Rust/system-library requirements; failures were preserved and system libc was not upgraded.

Actual results: positive 2 verified/0 errors; negative 0 verified/1 error with exit 1; model 0 verified/0 errors, TYPECHECKED only. Collected license/source records are in [tools.md](tools.md) and `evidence/model-*`; a full transitive-license audit was NOT_RUN. These checks do not establish temporal-library compatibility. verus-tla remains uninstalled/unverified and ADR-08 remains DRAFT. The current persistent installation uses the same release; tool changes still require recalibration. Original raw evidence remains unchanged; non-English raw evidence must not be forwarded to LLMs as context.

## Incremental revision observations (2026-10-27)

The [official Codex CLI reference](https://developers.openai.com/codex/cli/reference/)
was consulted for non-interactive invocation, then installed `codex exec --help`
confirmed `--add-dir` and `--skip-git-repo-check`. These support a controller-owned
external build directory and candidate copies without Git metadata. Only help was
run; no live generation session was launched. The revised controller validates
installed capabilities rather than assuming every CLI version supports them.

Installed `verus --help` and actual JSON output confirmed entire-crate results
and successful function attribution under `--output-json --time-expanded`.
The protected local proof caller is checked against the same executable body used
by runtime tests. Current results and limitations are in the review packet;
these tool observations do not approve specifications or prove global Raft.

## IDS-inspired workflow revision

The [published synthesis implementation guide](https://github.com/skydiscover-ai/skydiscover/blob/main/skydiscover/synthesize/README.md)
describes separate implementation, evaluation and critique roles and incremental
formal code/proof synthesis. We reuse those ideas in one bounded serial controller:
independent oracles, joint source/proof edits and previous-attempt feedback. No
SkyDiscover dependency or multi-agent framework is installed. Our draft targets
remain outside ordinary repair scope; target changes return to user review.
The STATE-INIT demonstration exercises real Rust/Verus rejection and repair, not
IDS reproduction, real model success, full refinement or Raft liveness.

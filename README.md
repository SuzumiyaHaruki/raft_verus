# One MadRaft code-and-proof experiment — DRAFT

The experiment asks Codex to complete the actual MadRaft student project and jointly construct Verus proofs of the executable Raft implementation. This checkout supplies the problem, original test coverage and tools. It does not supply a Raft solution. Preparation and a completed experiment are different outcomes; see [RESULT.md](RESULT.md) for the actual current result.

| Path | Purpose |
| --- | --- |
| `madraft/src/raft/` | Upstream Raft interfaces, student placeholders and election, replication, persistence and snapshot tests. |
| `madraft/src/kvraft/` | Replicated KV service and clients, including retry and snapshot work. |
| `madraft/src/shard_ctrler/` | Shard-assignment service built on Raft; its `Config` interface is retained. |
| `madraft/src/shardkv/` | Sharded KV clients, servers and migration/recovery tests. |
| `madraft/proof/` | The generator's local proof entrypoints and final executable correspondence. Initially only an existing observer is checked; protocol proofs are absent. |
| `requirements/` | English scope, safety definitions, environment/TCB, original test inventory and independent Verus callers. |
| `docs/` | Source provenance and preparation records for the operator; not copied into experiments. |
| `scripts/` | Thin launch/resume helper, real checker and offline helper tests. |
| `runs/` | Fresh experiment workspaces and raw outputs, all git-ignored. |
| `target/` | Existing local dependency cache only, git-ignored. No previous protocol source is copied into experiments. |

This README is for the experiment operator and is not copied into experiment workspaces. [AGENTS.md](AGENTS.md) supplies the main instruction and local check commands. [scope.md](requirements/scope.md) describes the whole assignment. [provenance.md](docs/provenance.md) records upstream versions, compatibility edits and preparation history for operator review only. The START/CONTINUE prompts and the supplied MadRaft documentation and source comments also guide the experiment. Every authored specification is DRAFT. Human review happens after delivery; no approval file is used during execution.

## Run an experiment

Python 3.10+ and the paths in [tools.ini](tools.ini) are required. The local configuration selects the already installed Verus/Rust pair and a repository-local Cargo cache. It does not install tools or change global configuration. `MADRAFT_VERUS`, `MADRAFT_RUST_BIN`, `MADRAFT_RUSTUP_HOME` and `MADRAFT_CARGO_HOME` override paths. The experiment explicitly selects `gpt-6-astra` with `high` reasoning, preserving the previously effective selection; `--model` and `--effort` override it. An empty model setting uses the CLI default, not the operator's personal configuration.

Each experiment uses `runs/<name>/runs/codex-home` as its own `CODEX_HOME`, also retained on resume. Only the operator's existing `auth.json` and execution `rules/` are linked, preserving authentication and command safeguards without copying credentials. Personal `config.toml`, `AGENTS.md`, `AGENTS.override.md`, earlier sessions, memories and agent definitions are not linked or copied. This excludes automatic loading from the personal Codex home; it is not filesystem isolation. Automatic credential reuse supports the current file-based login; keyring-only authentication and custom providers need experiment-specific setup. The prompts no longer prohibit subagents; no fixed delegation scheme is imposed.

```bash
# Preview: zero agent calls, zero workspace creation.
python3 scripts/run.py new runs/experiment

# Explicitly start the real agent in a fresh directory.
python3 scripts/run.py new runs/experiment --execute

# Resume its conversation and evolving files; the wall-clock budget is cumulative.
python3 scripts/run.py resume runs/experiment --execute

# Explicitly extend the cumulative budget when needed.
python3 scripts/run.py resume runs/experiment --total-seconds 7200 --execute
```

Each new experiment requires a fresh direct child of this checkout's `runs/`, such as `runs/experiment/`. The launcher copies `madraft/`, `requirements/`, `AGENTS.md`, `tools.ini`, `.gitignore` and the three local checking helpers into it, then invokes `codex exec` once with that new directory as its working directory. All generated implementation and proof edits belong to `runs/experiment/madraft/`; the supplied root `madraft/` remains the starting point for later experiments. Files are copied, not moved or linked, and generated changes are never copied back automatically. The operator README, `docs/`, launcher, maintenance tests, prior results and previous run output are not supplied. `madraft/README.md` remains with the upstream project as assignment context and attribution.

A continuation uses the recorded thread ID with `codex exec resume` in the same workspace; it does not recreate source. The agent chooses increments, runs real tools, repairs its code/proofs and writes `PROGRESS.md`. There is no task DAG or automatic agent loop. Raw prompt, JSONL and stderr are retained in that workspace's `runs/`. The configured budget bounds wall-clock runtime, not token cost. Output size is also bounded. A timeout preserves the workspace. A controller crash is recorded conservatively as spent budget; inspect the recorded process and logs before recovering it. A successful Codex exit is not proof acceptance. The input-copy policy and working-directory selection are not an additional OS isolation mechanism; the configured Codex sandbox still applies.

## Check current or delivered files

```bash
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check.py build
python3 scripts/check.py inventory
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py test kvraft::tests:: --seed 1
python3 scripts/check.py test shard_ctrler::tests:: --seed 1
python3 scripts/check.py test shardkv::tests:: --seed 1
python3 scripts/check.py definitions

# Current local proof entrypoint; the agent extends it while developing:
python3 scripts/check.py proof madraft/proof/progress.rs --workspace runs/experiment
python3 scripts/check.py safety --workspace runs/experiment
python3 scripts/check.py final --workspace runs/experiment --timeout 3600
```

`test` without a filter runs every lab and added regression; `inventory` lists them. `definitions` verifies definition regressions, not Raft correctness. `proof` checks a complete local proof entrypoint (initially the existing role observer, not protocol safety). `safety` checks all five final callers and snapshot/executable correspondence together with `--no-cheating`; a missing generated file produces a real compiler failure. `final` runs tests and that aggregate check independently using the invoking checkout's tools and target definitions. It does not trust an agent's success message. Review the generated execution relation, host boundary, source coverage and any test-plumbing changes before accepting semantic adequacy. No script can infer that an arbitrary model describes every real operation merely from the existence of a proof.

All Cargo checks use the locked dependencies, offline mode, `--cfg madsim` and the Rust compiler matching Verus. The same `vstd` macro erases proof syntax in executable builds. If the dependency cache is absent, fetch the locked dependencies explicitly with that Cargo and `CARGO_HOME`; checking never downloads tools or silently switches a compiler. `MADSIM_TEST_NUM` and `MADSIM_TEST_CHECK_DETERMINISTIC` remain available for the original simulator's repeated-seed and replay modes. See [test-inventory.md](requirements/test-inventory.md).

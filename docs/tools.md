# Verus installation and checks

Status: DRAFT. Tool installation is separate from approval of specifications or completion of Raft proofs.

## Current installation

The persistent directory is `/home/nitro/.local/share/raft-verus/`:

```text
raft-verus/
├── verus-x86-linux/     Official Verus release, bundled vstd and Z3
├── rustup/              Isolated Rust 1.92.0 toolchain
└── verus-0.2026.01.23.1650a05-x86-linux.zip
```

Verus is `0.2026.01.23.1650a05`, commit `1650a05d64219fdd1d150f32d0d7d5b19b8defcd`; bundled Z3 is 4.12.5. The required Rust is 1.92.0. CARGO_HOME is configured as a separate `cargo/` path under this directory; that directory is created only if a tool needs it. Actual version output and installation commands are in [installation records](../evidence/layout-and-tools/installation.json). The user's default Rust remains 1.88.0. No shell profile, global PATH, system libc or default toolchain was changed.

`workflow.toml` uses the persistent executable paths and sets RUSTUP_HOME/CARGO_HOME for verifier subprocesses. From the repository root:

```sh
python3 -m orchestrator verify --profile smoke-positive
python3 -m orchestrator verify --profile smoke-negative
python3 -m orchestrator verify --profile spec
python3 -m orchestrator verify --profile full
```

Positive smoke checks the tool on a small proof. Negative smoke must reject an incorrect postcondition. `spec` only typechecks the candidate model and closes no Raft obligation. `full` is blocked by unfinished implementation, proofs and formal isolation. The [review packet](../review/REVIEW_PACKET.md) separates actual results from open work.

## Why the previous installation was missing

The bootstrap installed Verus and isolated Rust under `/tmp/raft-verus-tools/`. That directory was absent when checked on 2026-10-27, and the configured version probe returned file-not-found. The available records do not establish who removed it or when. A temporary directory was unsuitable for a toolchain intended for repeated use; the persistent installation above corrects that choice. Original installation and successful proof-smoke/typecheck logs remain in `evidence/model-*` and `evidence/bootstrap/`; earlier missing-tool results are retained in `evidence/simplification/`.

## Reinstallation commands

These commands install the same release in the persistent location. Per-command environment variables keep the default Rust unchanged:

```sh
mkdir -p /home/nitro/.local/share/raft-verus
curl --proto '=https' --tlsv1.2 --fail --location \
  https://github.com/verus-lang/verus/releases/download/release/0.2026.01.23.1650a05/verus-0.2026.01.23.1650a05-x86-linux.zip \
  -o /home/nitro/.local/share/raft-verus/verus-0.2026.01.23.1650a05-x86-linux.zip
unzip -n /home/nitro/.local/share/raft-verus/verus-0.2026.01.23.1650a05-x86-linux.zip \
  -d /home/nitro/.local/share/raft-verus
RUSTUP_HOME=/home/nitro/.local/share/raft-verus/rustup \
CARGO_HOME=/home/nitro/.local/share/raft-verus/cargo \
  rustup toolchain install 1.92.0 --profile minimal --no-self-update
```

## Provenance and limits

The [official release](https://github.com/verus-lang/verus/releases/tag/release/0.2026.01.23.1650a05) supplies the matching Verus/vstd pair. Its finite Map/Set API is the one used by the candidate model: `Map::new` takes a membership predicate and quorums use `members.filter(...)`. Earlier API/trigger failures remain in the historical evidence; no replacement axioms were introduced. The first newer release failed local system-library compatibility, so the already exercised compatible release was restored.

Verus/vstd and Z3 use MIT licenses; isolated Rust generally uses MIT/Apache-2.0 with component-specific licenses. Original license files are retained in `evidence/model-verus-LICENSE.txt` and `evidence/model-z3-LICENSE.txt`. A full transitive-license audit is NOT_RUN. verus-tla compatibility and temporal proofs remain BLOCKED/OPEN. No tool source was copied into the Raft implementation.

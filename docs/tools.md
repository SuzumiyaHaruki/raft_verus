# Local toolchain

Status: DRAFT. Tool availability does not approve specifications or complete Raft proofs.

`workflow.toml` selects the installed Verus 0.2026.01.23.1650a05, its matching vstd, isolated Rust/Cargo 1.92.0 and bundled Z3 4.12.5. The local installation root is `/home/nitro/.local/share/raft-verus`; change toolchain.install_root or set RAFT_TOOLCHAIN_ROOT on another machine. The wrappers configure child processes only and never replace the system default toolchain.

```text
raft-verus/
├── verus-x86-linux/   Verus, matching vstd, Z3 and installed license files
└── rustup/            Isolated Rust 1.92.0 toolchain
```

`RAFT_WORKFLOW_CONFIG` selects an alternate TOML for direct wrappers. `python3 -m orchestrator doctor` records resolved paths and actual installed versions without model generation. Cargo receives the installed Verus macro dependencies; macro erasure and verification therefore use the same executable source. Plain Cargo without those flags is not the supported checking command.

```sh
python3 -m orchestrator verify --profile smoke-positive
python3 -m orchestrator verify --profile smoke-negative
python3 -m orchestrator verify --profile spec
python3 crates/raft-verified/check.py build
python3 crates/raft-verified/check.py core-proof
```

Current positive/negative calibration logs are under evidence/calibration. A negative smoke result must be the expected false-postcondition rejection, not a missing-tool or compiler failure. Typechecking closes no proof obligation. Full proofs remain incomplete; verus-tla integration has not been compatibility-checked. The official matching release is [release/0.2026.01.23.1650a05](https://github.com/verus-lang/verus/releases/tag/release/0.2026.01.23.1650a05). This cleanup does not download or reinstall tools.

## MadSim dependency cache

`toolchain.madsim_cargo_home` identifies `/home/nitro/Desktop/raft_verus/target/madraft-cargo` on this machine. Change this path when relocating the checkout. It contains pinned dependencies required for offline checks, not generated Raft solutions. Each check stages dependencies under its writable CARGO_TARGET_DIR/cargo-home; candidate source does not contain target directories. Normal checks use --locked --offline. Missing tools or dependencies are blockers, never simulated success.

Explicitly populate a missing dependency cache with `python3 experiments/madraft-poc/check.py fetch` when downloads are authorized. Build outputs can be deleted without changing the installed toolchain. The source checker records its actual argv, environment, result and raw diagnostics; obsolete installation archives are no longer retained in this repository.

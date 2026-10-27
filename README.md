# Rust Raft with Verus

Status: DRAFT. The goal is an AI-assisted Rust Raft implementation with Verus safety and conditional-liveness proofs. This bootstrap contains the candidate model, LLM materials and runnable Python workflow. All 26 Raft obligations remain OPEN. Start with [the review packet](review/REVIEW_PACKET.md).

## Materials to maintain

All LLM material is English and grouped in `input/`. Each file has one topic:

| File | Topic |
|---|---|
| [requirements.md](input/requirements.md) | Scope, architecture and exclusions |
| [properties.md](input/properties.md) | Six safety and three liveness conclusions |
| [assumptions.md](input/assumptions.md) | Conditions under which those conclusions are required |
| [io.md](input/io.md) | Library/host events, effects, durability and application responsibilities |
| [trust.md](input/trust.md) | Trusted components and their limits |
| [model.md](input/model.md) | Existing Verus model semantics and gaps |
| [proofs.md](input/proofs.md) | Proof checklist, dependencies, coverage and incremental proof work |
| [instructions.md](input/instructions.md) | Common LLM instructions |
| `input/T01-MODEL.md` through `input/T08-INTEGRATION.md` | One task per file |
| [workflow.toml](workflow.toml) | Task order, editable paths, checks, tools and budgets |

Edit Markdown for meaning; edit TOML for execution settings. There are no separate author-maintained JSON contracts or task registries. Small JSON schemas inside the Python package remain for CLI reports and external approval. Execution evidence is generated automatically.

## Actual directory layout

```text
raft_verus/
├── input/                     English requirements, proofs and task Markdown
├── workflow.toml            Single workflow configuration
├── crates/
│   ├── raft-spec/src/       Candidate Verus state, transitions and properties
│   ├── raft-verified/src/   Rust interface and token helpers; Raft core still absent
│   ├── proof-smoke/         Positive and deliberately failing Verus examples
│   └── raft-proofs/         Placeholder for future Raft proofs
├── orchestrator/            Python package; CLI and controller modules live here
│   ├── wire/               Internal report/approval schemas and invalid approval example
│   └── tests/
│       └── fixtures/       Executable fake subprocesses for offline tests
├── docs/                   Workflow explanation, tool setup and sources
├── review/                 Review packet, pending decisions and traceability
├── evidence/               Actual logs; offline fake results marked SIMULATED
├── target/                 Cargo-generated build artifacts
├── .git/                   Original empty read-only directory; not a Git repository
├── .agents/                Original read-only directory
├── .codex/                 Original read-only directory
└── .aws/                   Original read-only directory
```

Root files also include `AGENTS.md`, Cargo manifests, `.gitignore` and this README. Generated Python caches/build metadata may appear after commands; they are not material to maintain.

## Use without launching Codex

```sh
cd /home/nitro/Desktop/raft_verus
python3 -m orchestrator validate
python3 -m orchestrator plan
python3 -m orchestrator prompt --task T01-MODEL
python3 -m orchestrator run-task --task T01-MODEL --dry-run
python3 -m unittest discover -s orchestrator/tests -v
cargo test --workspace --offline
```

`prompt` prints the actual assembled English prompt. `validate` checks structure, not proofs. Offline tests exercise fake subprocesses, not real Codex. Rust tests cover interface helpers, not Raft correctness.

Linux/POSIX and Python 3.10+ are supported. The existing dependencies suffice: Python 3.11 uses `tomllib`; 3.10 uses `tomli` or the installed pip's vendored fallback. `jsonschema` validates wire reports. Optional offline installation:

```sh
python3 -m pip install --no-index --no-deps --no-build-isolation \
  --target /tmp/raft-workflow-bootstrap ./orchestrator
PYTHONPATH=/tmp/raft-workflow-bootstrap python3 -m orchestrator validate
```

`doctor` probes versions/help only. `verify --profile smoke-positive`, `smoke-negative`, `spec`, `interface-check` and `full` run the independently configured checks. `report --run-id ACTUAL_RUN_ID` reads saved evidence.

Verus and its isolated Rust toolchain are installed persistently under `/home/nitro/.local/share/raft-verus/`; `workflow.toml` points there. The previous `/tmp` installation was missing, so it was restored to this user-owned location without changing the default Rust. See [tool setup](docs/tools.md) and the [review packet](review/REVIEW_PACKET.md) for actual check results. `full` remains BLOCKED by unfinished Raft proofs.

Real generation remains disabled in `workflow.toml`. Future execution needs human review, external approval, appropriate task checks and isolation; see [workflow behavior and limits](docs/workflow.md). Stop at the review packet. Do not recursively call Codex, automatically merge candidates or start full Raft development.

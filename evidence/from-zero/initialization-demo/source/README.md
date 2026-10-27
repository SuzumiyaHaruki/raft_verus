# Rust Raft generation with Verus

Status: DRAFT. The experiment starts without working Raft behavior. AI must generate one Rust Raft implementation and its Verus proofs incrementally, using functional/fault tests and measurements as independent feedback. Global safety, conditional liveness and implementation/cluster refinement remain open.

The supplied starting material is explicit: English requirements, shared data types, API signatures, candidate formal specifications, protected proof consumers, independent tests, replay/measurement harnesses and the Python controller. This is synthesis from specified interfaces, not synthesis from blank files. Read [the starting-point boundary](input/starting-point.md) and [the actual review results](review/REVIEW_PACKET.md).

`node.rs`, `reconcile.rs` and `persistence.rs` contain deliberately incorrect error-only seeds. `progress_lemmas.rs` has target signatures with empty bodies. These are unfinished assignments and must fail their semantic checks. Former working implementations and the induction proof are preserved only as offline fixtures under `orchestrator/tests`; that entire directory, historical evidence and maintenance-only `Agent.md` are omitted from every fresh candidate copy. No solution overlay runs on launch or resume.

## Incremental development

[The roadmap](input/roadmap.md) describes state, runtime, elections, replication, commitment, recovery, progress and integration. Each English module document explains its full responsibility, connections and bounded steps. The controller carries one generated candidate forward: generate code and proof, run independent checks, repair or redesign within cumulative budgets, then continue the selected plan. Previously completed tasks are rechecked. Human review is after delivery, with no approval file or interactive checkpoint inside the loop.

The default task is STATE-INIT. Selecting RUNTIME-PERSIST adds it after initialization; PROGRESS-FOUNDATION adds the early local progress proof. Four other ready steps cover generating/proving/replaying/measuring reconciliation; 19 later steps still lack runnable checks. No command currently produces a complete Raft. Every module remains incomplete.

Initialization runs all eight of its independent test cases and its own proof/replay, without requiring future unimplemented operations to pass. Shared data vocabulary lives in `types.rs`; initialization and persistence proof entrypoints import those types without importing the unrelated reconciliation body. This changes no target and uses no verification filter. Later selected tasks rerun completed-task checks through the existing controller.

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m orchestrator plan --format text
./experiment.sh
python3 -m orchestrator run-plan --tasks RUNTIME-PERSIST --dry-run
python3 -m unittest discover -s orchestrator/tests -v
```

These commands do not start model generation. The default preview makes zero agent calls. Live generation requires an explicit command:

```sh
./experiment.sh run             # STATE-INIT only, gpt-6.1-sol / high
./experiment.sh resume RUN_ID   # Continue the saved generated candidate
./experiment.sh report RUN_ID   # Inspect results without generation
python3 -m orchestrator run-plan --tasks RUNTIME-PERSIST --execute-codex
```

No real Codex task was run during maintenance. A new run copies the current scaffold; resume preserves the saved candidate rather than restarting from seeds. Manually replacing the source scaffold with an implementation changes the starting experiment and must be disclosed. Successful candidates are never automatically merged into this checkout.

## Offline demonstrations and real checks

```sh
PYTHONDONTWRITEBYTECODE=1 python3 orchestrator/tests/real_repair_demo.py \
  --task STATE-INIT --output /tmp/raft-init-from-zero
PYTHONDONTWRITEBYTECODE=1 python3 orchestrator/tests/real_repair_demo.py \
  --task RUNTIME-PERSIST --output /tmp/raft-persist-from-zero
PYTHONDONTWRITEBYTECODE=1 python3 orchestrator/tests/real_repair_demo.py \
  --task REPLICATION-RECONCILE --output /tmp/raft-reconcile-from-zero
```

Use new output directories. These demonstrations explicitly inject scripted candidate edits through a fake subprocess and label generation SIMULATED. Rust, replay and Verus checks actually execute; a controlled defect must be rejected before the repaired candidate passes. The fixture contents are never copied into the starting candidate. Demonstrations test the workflow and checker correspondence, not model capability or complete Raft correctness.

Direct checking uses `python3 crates/raft-verified/check.py build`, `test`, `node-proof`, `proof`, `persist-proof` or `progress-proof`. Build succeeds on the supplied scaffold; behavior tests and corresponding proofs intentionally fail until their implementations are generated. `spec` is only TYPECHECKED. Measurements are eligible only after the required correctness checks; there is no approved performance target.

Cargo and Verus check the same executable source using the installed compatible toolchain selected in `workflow.toml`. No downloads or system-toolchain replacement occur. See [tools](docs/tools.md), [proof entrypoints](crates/raft-proofs/README.md) and [workflow](docs/workflow.md) for scope, budgets, history, regression repair, optional comparable measurements and commands.

## Actual directory layout

```text
raft_verus/
├── Agent.md                   Maintenance objective; excluded from experiment candidates
├── input/                     English requirements, module assignments and roadmap
│   ├── state.md, runtime.md  Shared node state and continuous event/I/O processing
│   ├── elections.md         Complete election lifecycle and internal steps
│   ├── replication.md       Both sides of log replication and internal steps
│   ├── commitment.md        Commit advancement and ordered delivery
│   ├── recovery.md          Crash reconstruction and cluster fault scenarios
│   ├── progress.md          Required conditional liveness and progress refinement
│   ├── integration.md       Real adapters, global proofs and measurements
│   └── proofs.md             Global proof dependencies and local-proof scope
├── workflow.toml             Task inputs, editable paths, checks, tools and budgets
├── experiment.sh             Preview/run/resume/report for node initialization
├── crates/
│   ├── raft-spec/src/        Candidate Verus states, transitions and properties
│   ├── raft-verified/
│   │   ├── src/              Shared data types and failing executable seeds
│   │   ├── tests/            Functional regressions and local replay cases
│   │   ├── examples/         Deterministic replay and core measurement programs
│   │   └── check.py          Installed-tool build/run wrapper
│   ├── raft-proofs/          Local proof entrypoint and build-path explanation
│   └── proof-smoke/          Positive/negative verifier calibration samples
├── orchestrator/             Existing Python controller and CLI
│   ├── wire/                 Internal agent/check subprocess report schemas
│   └── tests/                Controller regressions and scripted process fixtures
├── docs/                     Workflow, tool setup and source references
├── review/                   Review packet, pending decisions and measurements
├── evidence/                 Preserved logs, controller candidates and run records
└── target/                   Generated Cargo artifacts
```

The maintenance workspace originally had empty read-only `.git`, `.agents`, `.codex` and `.aws` directories; those were preserved. That observation describes this local environment, not a normal GitHub clone. Cargo manifests, `AGENTS.md` and `.gitignore` also live at the root. Generated caches and build metadata are not maintained input material.

All LLM-facing material is English, with one physical line per Markdown prose paragraph. Python 3.10+ on Linux/POSIX is supported. Existing jsonschema and TOML support suffice; no new dependencies are required. Candidate copying and input exclusion prevent accidental answer inclusion, not hostile same-user access outside the candidate.

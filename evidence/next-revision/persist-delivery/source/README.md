# Rust Raft with Verus

Status: DRAFT. Use AI to develop one Rust Raft implementation in small steps, with Verus proofs of the same executable code, functional/fault tests and measurements. The existing log reconciliation helper works; a complete Raft node, cluster safety and conditional liveness remain unfinished.

The default experiment now generates **node initialization**, not another helper exercise. Shared state, eight independent initialization tests and a protected Verus target are supplied. `src/node.rs` is an explicit reject-all starting point: it builds, but intentionally fails initialization tests/proof until an agent implements it. A scripted offline run produces a checked candidate for workflow testing; it is not live AI output and is not automatically integrated. See [the review packet](review/REVIEW_PACKET.md) for actual files/results.

## Develop complete modules in checked steps

Start with [the module roadmap](input/roadmap.md): state, runtime, elections, replication, commitment, recovery, progress and integration. Each module has one English Markdown document describing its full responsibility, connections, completion criteria and internal steps. `workflow.toml` selects one named step for each bounded controller call. The prompt includes the module context and that step; it does not ask an agent to finish the entire module at once.

STATE-INIT remains the default generation task. A combined plan continues in the same candidate through RUNTIME-PERSIST and the early PROGRESS-FOUNDATION; four optional steps exercise delivered helper work, and 19 steps remain planned. Every module is still incomplete. Formal targets live in Verus source; specifications remain DRAFT and independent checks do not approve semantics.

These modules share one coupled node state and event/I/O sequence. Local contracts and proofs must compose through shared invariants and node/cluster refinement; separate module checks alone cannot establish Raft correctness. Cross-module effects and caller obligations are part of each scoped change.

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m orchestrator plan --format text
python3 -m orchestrator prompt --task STATE-INIT
python3 -m orchestrator run-task --task STATE-INIT --dry-run
python3 -m unittest discover -s orchestrator/tests -v
python3 -m orchestrator verify --profile reconcile-tests
python3 -m orchestrator verify --profile reconcile-proof
python3 crates/raft-verified/check.py replay
python3 -m orchestrator verify --profile reconcile-bench
```

These commands do not start Codex generation. `validate` checks configuration and references. Tests exercise executable code; `spec` only typechecks the model; `reconcile-proof` checks the local operation against `merged`. Benchmark output is measurement data with no approved performance threshold. `full` reports missing whole-project proof and approval prerequisites.

The runtime and proof paths use the same `src/reconcile.rs`. The small `check.py` wrapper uses the already installed Verus macros and their matching Rust 1.92 only in child processes. It does not install tools or change the user's default Rust. Use this wrapper for Cargo build/test/replay/measurement; plain Cargo under an incompatible default compiler cannot load those macros. See [the build path](crates/raft-proofs/README.md) and [tool setup](docs/tools.md).

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
│   │   ├── src/              Shared node types, constructor seed and reconciliation
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

## Controller use

`orchestrator/` reuses bounded subprocess execution, candidate copies, independent checks. Human review happens after code delivery. Step completion, module completion and whole-project acceptance are separate. The controller reports only selected-step completion; it does not infer module completion from a passing plan. Failed checks produce focused reproduction feedback. A serial plan can retain one candidate across tasks and resume after interruption; it never merges candidate files into this checkout.

The project defaults are now `gpt-6.1-sol` with `high` reasoning effort. The configuration switch permits an explicitly started experiment. Start and resume need no approval file or in-process human review. Review the delivered candidate and results afterward. No real agent has been started during this setup. Global Codex configuration and the system Rust toolchain are unchanged.

```sh
./experiment.sh                 # Preview; zero agent calls
./experiment.sh run             # Generate and prove the initial Node constructor
./experiment.sh resume RUN_ID   # Continue the saved experiment candidate
./experiment.sh report RUN_ID   # Read saved results
```

The launcher selects STATE-INIT using `gpt-6.1-sol` / `high`, at most three calls and two repair rounds. Code and loop proofs are edited together in one candidate; tests, initial-state replay and actual Verus checking diagnose each attempt. Measurements run after required correctness checks pass. Cumulative budgets and prior design/check history survive resume. Persistent failing targets can trigger one bounded redesign; affected completed tasks are invalidated and rechecked during repair. Explicit budget extension and scoped rewind commands are described in docs/workflow.md. No live generation was run while preparing this revision. See [the workflow guide](docs/workflow.md#configured-solhigh-experiment).

The main `check.py test` command includes the intentionally failing initialization seed. The `reconcile-tests` profile selects the original 15 passing regressions. To demonstrate the new generation/repair path offline:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 orchestrator/tests/real_repair_demo.py \
  --task STATE-INIT --output /tmp/raft-node-demo
```

Use a new output directory. Scripted wrong-term code is rejected by real tests and Verus, then a scripted repair is rechecked. Both the candidate and raw evidence are retained; generation is labeled SIMULATED. This cannot show that a real model will succeed. RUNTIME-PERSIST now has a same-Node coordinator, protected targets/tests and a repeatable completion scenario. Run `python3 orchestrator/tests/real_repair_demo.py --task RUNTIME-PERSIST --output /tmp/raft-persist-demo` for a scripted STATE-INIT → persistence defect → repair demonstration with real checks. `python3 -m orchestrator run-plan --tasks RUNTIME-PERSIST --dry-run` previews the combined path; replacing `--dry-run` with `--execute-codex` explicitly starts the later real experiment. No manual solution copy or merge is needed between steps. PROGRESS-FOUNDATION can be selected similarly to include the early local progress proof. Global safety, liveness, legal handler calls, recovery and full dispatch remain open.

Python 3.10+ on Linux/POSIX is supported. The installed `jsonschema` and TOML reader suffice; no new Python dependencies are required. Optional offline install:

```sh
python3 -m pip install --no-index --no-deps --no-build-isolation \
  --target /tmp/raft-workflow-revision ./orchestrator
PYTHONPATH=/tmp/raft-workflow-revision python3 -m orchestrator validate
```

All agent-facing material is English. Markdown explains one topic per file in `input/`; TOML contains execution settings. The small wire JSON schemas are machine reports, not a second author-maintained requirements registry.

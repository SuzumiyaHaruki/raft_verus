# Rust Raft with Verus

Status: DRAFT. This repository develops an independent, fixed-membership Rust
Raft library in small, checked steps. It currently implements log-suffix
reconciliation, with executable tests, a local Verus correspondence proof,
deterministic replay and a core microbenchmark. A complete node, cluster safety,
implementation refinement and conditional liveness remain unfinished. Start with
[the review packet](review/REVIEW_PACKET.md) for actual results and open decisions.

## Develop complete modules in checked steps

Start with [the module roadmap](input/roadmap.md): state, runtime, elections,
replication, commitment, recovery, progress and integration. Each module has one
English Markdown document describing its full responsibility, connections,
completion criteria and internal steps. `workflow.toml` selects one named step
for each bounded controller call. The prompt includes the module context and
that step; it does not ask an agent to finish the entire module at once.

Four steps can recheck or improve delivered local work; 21 planned steps identify
the missing protocol, integration and proof work. Every module is still incomplete.
The next new protocol work is node initialization, followed by persistence
coordination. Formal definitions and proofs remain in Verus source. Both complete
safety and conditional liveness are required project outcomes.

These modules share one coupled node state and event/I/O sequence. Local contracts
and proofs must compose through shared invariants and node/cluster refinement;
separate module checks alone cannot establish Raft correctness. Cross-module
effects and caller obligations are part of each scoped change.

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m orchestrator plan --format text
python3 -m orchestrator prompt --task REPLICATION-RECONCILE
python3 -m orchestrator run-task --task REPLICATION-RECONCILE --dry-run
python3 -m unittest discover -s orchestrator/tests -v
python3 crates/raft-verified/check.py test
python3 -m orchestrator verify --profile reconcile-proof
python3 crates/raft-verified/check.py replay
python3 -m orchestrator verify --profile reconcile-bench
```

These commands do not start Codex generation. `validate` checks configuration and
references. Tests exercise executable code; `spec` only typechecks the model;
`reconcile-proof` checks the local operation against `merged`. Benchmark output is
measurement data with no approved performance threshold. `full` reports missing
whole-project proof and approval prerequisites.

The runtime and proof paths use the same `src/reconcile.rs`. The small
`check.py` wrapper uses the already installed Verus macros and their matching
Rust 1.92 only in child processes. It does not install tools or change the user's
default Rust. Use this wrapper for Cargo build/test/replay/measurement;
plain Cargo under an incompatible default compiler cannot load those macros.
See [the build path](crates/raft-proofs/README.md) and [tool setup](docs/tools.md).

## Actual directory layout

```text
raft_verus/
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
├── experiment.sh             Preview/run/resume/report for the four-step experiment
├── crates/
│   ├── raft-spec/src/        Candidate Verus states, transitions and properties
│   ├── raft-verified/
│   │   ├── src/              Executable interfaces and shared reconciliation logic
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

The original read-only `.git`, `.agents`, `.codex` and `.aws` directories are
preserved. This checkout's `.git` is empty; it is not a usable Git repository.
Cargo manifests, `AGENTS.md` and `.gitignore` also live at the root. Generated
caches and build metadata are not maintained input material.

## Controller use

`orchestrator/` reuses bounded subprocess execution, candidate copies, independent
checks. Human review happens after code delivery. Step completion, module completion and whole-project
acceptance are separate. The controller reports only selected-step completion;
it does not infer module completion from a passing plan. Failed checks produce
focused reproduction feedback. A serial plan
can retain one candidate across tasks and resume after interruption; it never
merges candidate files into this checkout.

The project defaults are now `gpt-6.1-sol` with `high` reasoning effort. The
configuration switch permits an explicitly started experiment. Start and resume
need no approval file or in-process human review. Review the delivered candidate
and results afterward. No real agent has been started during this setup.
Global Codex configuration and the system Rust toolchain are unchanged.

```sh
./experiment.sh                 # Preview; zero agent calls
./experiment.sh run             # Run the four configured steps
./experiment.sh resume RUN_ID   # Continue the saved experiment candidate
./experiment.sh report RUN_ID   # Read saved results
```

The launcher selects reconciliation, its local proof, replay tests and measurement;
it does not activate planned full-node work. It uses the existing Python controller.
See [experiment settings](docs/workflow.md#configured-solhigh-experiment).

Python 3.10+ on Linux/POSIX is supported. The installed `jsonschema` and TOML
reader suffice; no new Python dependencies are required. Optional offline install:

```sh
python3 -m pip install --no-index --no-deps --no-build-isolation \
  --target /tmp/raft-workflow-revision ./orchestrator
PYTHONPATH=/tmp/raft-workflow-revision python3 -m orchestrator validate
```

All agent-facing material is English. Markdown explains one topic per file in
`input/`; TOML contains execution settings. The small wire JSON schemas are
machine reports, not a second author-maintained requirements registry.

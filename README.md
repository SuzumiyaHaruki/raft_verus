# Rust Raft generation with Verus

Status: DRAFT. The generation experiment's default paths remain unfinished seeds. AI must generate one Rust Raft implementation and its Verus proofs incrementally, using functional/fault tests and measurements as independent feedback. A separate, explicitly requested maintenance-authored MadSim feasibility slice now exists behind the `madraft-poc` feature; see its [report](review/MADRAFT_FEASIBILITY_REPORT.md). Global safety, conditional liveness and implementation/cluster refinement remain open.

The experiment supplies English requirements, observable types, behavioral targets, independent tests/proof callers, replay/measurement harnesses and the Python controller. The LLM generates executable logic **and its implementation-layer contracts, representation, invariants and proofs**. Independent callers require that generated design to satisfy the supplied behavioral targets. A proof of a self-written weak contract alone is insufficient. Read [the input boundary](input/starting-point.md) and [actual review results](review/REVIEW_PACKET.md).

`node.rs`, `reconcile.rs` and `persistence.rs` contain deliberately incorrect error-only seeds. `node_state.rs` is an editable starting representation; `contracts.rs` has vacuous local predicates that cannot satisfy the independent callers. The progress target has no supplied induction strategy. Offline solutions, historical evidence and maintenance-only `Agent.md` are excluded from candidates. Launch/resume never installs a solution.

## Incremental development

[The roadmap](input/roadmap.md) describes state, runtime, elections, replication, commitment, recovery, progress and integration. Each English module document explains its full responsibility, connections and bounded steps. The controller carries one generated candidate forward: generate code, local specifications and proofs, run independent checks, repair or redesign within cumulative budgets, then continue the selected plan. Previously completed tasks are rechecked. Human review is after delivery, with no approval file or interactive checkpoint inside the loop.

The default task is STATE-INIT. Selecting RUNTIME-PERSIST adds it after initialization; PROGRESS-FOUNDATION adds the early local progress proof. Four other ready steps cover generating/proving/replaying/measuring reconciliation; 19 later steps still lack runnable checks. No command currently produces a complete Raft. Every module remains incomplete.

Initialization runs eight independent tests and its proof/replay without requiring future unfinished operations to pass. The generated `Node::view()` and `wf()` connect to protected targets over `Observation`; executable observation adapters are also checked. Later selected tasks rerun completed-task checks, including after a representation change.

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
  --task PROGRESS-FOUNDATION --redesign --output /tmp/raft-joint-design
```

Use a new output directory. This explicitly injects SIMULATED edits through a fake subprocess: an alternative Node layout/invariant, wrong initialization with a vacuous local contract, repair, a persistence defect and repair, then a progress proof. Real Rust/Verus checks must reject the defects and pass the repairs in the same candidate. No offline solution is included at candidate creation. This demonstrates the workflow and independent correspondence, not model capability. Omit `--redesign` for the starting layout; select `STATE-INIT` or `REPLICATION-RECONCILE` for their individual demonstrations.

Direct checking uses `python3 crates/raft-verified/check.py build`, `test`, `node-proof`, `proof`, `persist-proof` or `progress-proof`. Build succeeds on the supplied scaffold; behavior tests and corresponding proofs intentionally fail until their implementations are generated. `spec` is only TYPECHECKED. Measurements are eligible only after the required correctness checks; there is no approved performance target.

Cargo and Verus check the same executable source using the installed compatible toolchain selected in `workflow.toml`. Normal checks are offline and do not replace system toolchains. The separate [MadSim experiment](docs/experiments/madraft-poc.md) adds pinned dependencies and an explicit fetch command; its local RequestVote proofs and real simulation checks do not finish the default tasks. See [tools](docs/tools.md), [proof entrypoints](crates/raft-proofs/README.md) and [workflow](docs/workflow.md) for scope, budgets, history, regression repair, optional comparable measurements and commands.

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
│   │   ├── src/              Editable implementation/specification seeds and observable API
│   │   ├── tests/            Functional regressions and local replay cases
│   │   ├── examples/         Deterministic replay and core measurement programs
│   │   └── check.py          Installed-tool build/run wrapper
│   ├── raft-proofs/          Protected consumers and editable auxiliary proof files
│   └── proof-smoke/          Positive/negative verifier calibration samples
├── experiments/madraft-poc/  Optional real MadSim host, fault tests and negative control
├── orchestrator/             Existing Python controller and CLI
│   ├── wire/                 Internal agent/check subprocess report schemas
│   └── tests/                Controller regressions and scripted process fixtures
├── docs/                     Workflow, tool setup and source references
├── review/                   Review packet, pending decisions and measurements
├── evidence/                 Preserved logs, controller candidates and run records
└── target/                   Generated Cargo artifacts
```

The maintenance workspace originally had empty read-only `.git`, `.agents`, `.codex` and `.aws` directories; those were preserved. That observation describes this local environment, not a normal GitHub clone. Cargo manifests, `AGENTS.md` and `.gitignore` also live at the root. Generated caches and build metadata are not maintained input material.

All LLM-facing material is English, with one physical line per Markdown prose paragraph. Python 3.10+ on Linux/POSIX is supported; the controller still uses its existing dependencies. Candidate copying is not hostile same-user isolation. The optional maintenance vote source is visible in normal copies even though its feature is disabled: declare it as supplied code, or explicitly isolate it before claiming a future experiment starts without a supplied protocol solution. No real generation was run with it.

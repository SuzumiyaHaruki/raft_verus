# Joint MadRaft implementation and proof experiment — DRAFT

This project supplies an unfinished MadRaft student scaffold, fixed safety requirements and a bounded Python experiment loop. The experiment Codex generates and repairs the implementation and proofs together. Maintenance work improves that loop; it does not supply a Raft solution. See [RESULT.md](RESULT.md) for the current system revision and actual maintenance checks.

| Path | Purpose |
| --- | --- |
| `madraft/` | Initial student source, original lab tests, license and a local Verus smoke entry. It remains the input to every new experiment. |
| `requirements/` | Scope, five fixed safety definitions, environment, test inventory, independent callers and execution-coverage responsibilities. |
| `scripts/run.py` | Fresh workspace/session creation, cumulative budgets, independent checks, feedback and bounded same-session continuation/redesign. |
| `scripts/check.py` | Real Cargo/Verus checking, required-test coverage, per-obligation diagnostics and full acceptance. |
| `scripts/prompts/` | Short start, continuation and redesign prompts. They are passed by the controller, not copied from an old experiment. |
| `scripts/tests/` | Offline system regressions and real Rust/Verus micro-examples. No complete Raft solution fixture. |
| `docs/` | Operator-only source provenance. |
| `runs/<name>/` | One experiment's editable MadRaft copy, isolated Codex home, prompts, generation logs, progress and feedback. |
| `runs/.controller/<name>/` | Preparation-side origin, fixed inputs, lock, cumulative state and independent check evidence. It is outside the agent's writable workspace. |
| `target/` | Shared dependency/tool cache. Experiment protocol build outputs are not reused as inputs. |

This README, maintenance tests, operator reports and provenance are not supplied to the agent. `AGENTS.md`, the requirements, original MadRaft documentation/source and the current controller prompt are its authored materials. Git tracks `runs/fresh-system-01`, including its source, proofs, prompts and check evidence; credentials, the Codex home and build caches remain excluded. Old `runs/sol-high-01` remains available locally and in Git history, and is not used by new experiments.

## Start and continue

Use Python 3.10+ and the already installed tools configured in `tools.ini`. The default remains `gpt-6-astra` with `high` reasoning; overrides are explicit. The following command selects `gpt-6.1-sol` with `high` without changing the global configuration.

```bash
# Maintenance regressions only; simulated agent calls are labeled SIMULATED.
python3 -m unittest discover -s scripts/tests -v

# Preview: no candidate, tools or model invocation.
python3 scripts/run.py new runs/fresh-system-01 --model gpt-6.1-sol --effort high

# Real experiment, explicitly authorized by the operator.
python3 scripts/run.py new runs/fresh-system-01 --model gpt-6.1-sol --effort high --execute

# Continue only this new-system experiment, preserving its thread and spending.
python3 scripts/run.py resume runs/fresh-system-01 --execute

# Explicitly extend the cumulative limits; this does not reset prior spending.
python3 scripts/run.py resume runs/fresh-system-01 --total-seconds 7200 --max-rounds 12 --execute

# Independent check from the preparation checkout.
python3 scripts/check.py final --workspace runs/fresh-system-01 --timeout 3600 --json
```

A new run accepts an unused direct child of `runs/`. Its input whitelist contains the initial MadRaft source, fixed requirements, AGENTS, tool configuration and local check helpers. Nested run output, previous solutions, patches, sessions and build products are excluded. An origin record stores the input version, Git revision when exposed, exact input copies/file list, tool capabilities, selected model and budgets. Resume requires that new-system origin record, the same workspace and unchanged system inputs/tools. Historical session files alone cannot start a new-system continuation.

Each experiment gets its own `CODEX_HOME`. Only existing authentication and execution rules are linked; personal configuration, instructions, memories and sessions are excluded. The controller probes the installed CLI help and actual new/resume argument shapes before generation. It uses `--no-daemon`, `--ignore-user-config`, workspace-write sandboxing and no interactive approval. It never silently substitutes a model or installs a toolchain. `MADRAFT_VERUS`, `MADRAFT_RUST_BIN`, `MADRAFT_RUSTUP_HOME` and `MADRAFT_CARGO_HOME` select explicit local tool paths.

These controls isolate inputs and writes under the available Codex sandbox. They do not provide an OS-level prohibition on reading every other directory or public reference. The instructions explicitly forbid other experiment answers and operator sessions; strict information-flow isolation is not claimed. Authentication providers unsupported by the existing file-based login produce a recorded blocker, not a silent fallback.

## Automatic loop and termination

After capability checks and an initial-scaffold check, Codex works in a finite slice. The controller stops/reaps that process group, checks a source-only snapshot using preparation-side requirements, tests and helpers, then feeds actual diagnostics and this run's attempted approaches back to the same conversation. The agent chooses implementation order, local contracts, invariants and proof decomposition. Repeated unchanged check gaps trigger a bounded redesign prompt; they do not reset the workspace or import another answer.

Defaults in `tools.ini` are 3,600 cumulative seconds, at most 600 seconds per model slice, 300 seconds per independent check phase, eight total rounds, two unchanged rounds before redesign, and one redesign. Checks and tool probes consume the same cumulative wall-time budget. Resume does not reset time, round, stagnation or redesign counters. A process-group watcher kills workers if the controller dies; an interrupted phase keeps its whole reserved allowance charged. OS locking prevents two controllers writing the same run. A subsequent resume rechecks the candidate before further generation and never reuses stale PASS results.

| Terminal state | Meaning |
| --- | --- |
| `MACHINE_CHECKS_PASSED` | Complete required tests, all five top-level callers, executable refinement, snapshot/observation and nonempty-witness checks passed on one stopped candidate, including the aggregate caller. Semantic adequacy/TCB boundaries remain disclosed. |
| `INCOMPLETE` | Required work remains and time, rounds or bounded redesign are exhausted, or the controller was interrupted. |
| `BLOCKED` | A tool, dependency, authentication or execution environment prevents continuing real checks/generation. |
| `NEEDS_REVIEW` | Fixed input, tool, test or trust-boundary differences need resolution; the controller does not quietly weaken them. |

An agent exit code or a statement in its RESULT.md cannot produce success. The authoritative ledger is preparation-side `state.json`; the readable terminal report is the experiment's `runs/EXPERIMENT.md`. Each round retains its prompt, raw JSONL/stderr, source diff, candidate progress note and independent diagnostics. Check snapshots contain only the checked project source and fixed requirements, so the recorded commands remain inspectable without copying session homes or build trees. Human review is after delivery; no approval file or per-round approval checkpoint exists.

## Checks and their limits

```bash
python3 scripts/check.py build
python3 scripts/check.py inventory
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py proof madraft/proof/progress.rs
python3 scripts/check.py top election_safety
python3 scripts/check.py top execution_link
python3 scripts/check.py top snapshot_observation
python3 scripts/check.py top nonempty_execution
python3 scripts/check.py safety
python3 scripts/check.py final --seeds 1,2,3,4,5 --timeout 3600
```

Local commands check the calling workspace; `--workspace` lets the preparation-side checker inspect a delivered workspace against its independent standards. `safety` is an alias of the complete `final` acceptance; use `top NAME` for proof-only feedback. Both complete commands and each post-generation controller check use the full configured seed set, currently 1–5, on one source snapshot. The initial-scaffold probe uses the initial-election test at seed 1. Required test names are checked individually, including all lab families and three supplied host regressions. The original ignored empty upstream test remains disclosed; empty selections, dropped families or newly ignored required tests do not pass. Candidate changes to test sources, fixed requirements, check helpers, Cargo inputs or tool configuration are reported, and the independent snapshot uses the preparation-side standards.

Structured results include check scope, candidate/input revision, actual command, selected tool environment, process outcome, raw diagnostic path and missing coverage. Compiler errors, failed proof obligations, timeouts, missing dependencies and absent entries are distinct. Partial top-level checks verify an entire imported unit; there is no function-filter shortcut. Full acceptance always runs the combined caller, so a Python table of partial PASS results cannot substitute for mathematical composition.

The nonempty witness and limited source-link/vacuity inspection reject some known empty or disconnected constructions. They do not prove that arbitrary generated `execution`, `observe` or `refines_executable` definitions faithfully describe all running behavior. Review constructors, handlers, every allowed path, event completeness, crash history, snapshot decoder fidelity, caller preconditions and exact host contracts as specified in `requirements/coverage.md`. A machine pass is not a claim of full implementation-level correctness, application-level correctness or conditional liveness. Liveness remains explicitly OPEN.

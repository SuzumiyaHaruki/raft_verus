# Raft review packet

Status: DRAFT. Updated 2026-10-27. All 26 Raft obligations remain OPEN; no decision is approved. This revision renames the input directory, flattens the Python package and restores Verus to persistent storage. It does not implement or prove Raft.

## Files to review

| File or directory | Purpose |
|---|---|
| [input/requirements.md](../input/requirements.md) | Scope and exclusions |
| [input/properties.md](../input/properties.md) | Six safety and three liveness conclusions only |
| [input/assumptions.md](../input/assumptions.md) | Applicability and 14 environmental/resource premises |
| [input/io.md](../input/io.md) | Library/host contract, durability, errors, delivery and application |
| [input/trust.md](../input/trust.md) | Eight trusted components |
| [input/model.md](../input/model.md), [Verus source](../crates/raft-spec/src/lib.rs) | Candidate state, transitions, predicates and explicit gaps |
| [input/proofs.md](../input/proofs.md) | 26 OPEN obligations, 49 coverage mappings and dependency graph |
| [input/instructions.md](../input/instructions.md), `input/T*.md` | Common English instructions and eight individual task descriptions |
| [workflow.toml](../workflow.toml) | The single maintained workflow configuration |
| [Python package](../orchestrator/), [tests](../orchestrator/tests/) | Markdown loading, prompt assembly, Codex adapter and independent checks |
| [Rust interface](../crates/raft-verified/src/lib.rs), [smoke samples](../crates/proof-smoke/) | Runnable interface/token helpers and tool calibration examples |
| [Pending decisions](OPEN_DECISIONS.md) | Ten unapproved recommendations with rationale, alternatives and impacts |
| [Traceability](REQUIREMENTS_TRACEABILITY.md), [workflow](../docs/workflow.md), [tools](../docs/tools.md) | Requirement mapping, execution boundaries and tool restoration commands |

## Current layout and tool restoration

All English LLM material is now under `input/`. Each file retains its topic and DRAFT status. Python modules live directly in `orchestrator/`, alongside `wire/` and `tests/`. The old nested source/package directories are removed; imports, packaging, protected paths, documentation and tests use the new locations. From the repository root, `python3 -m orchestrator` works without PYTHONPATH. The optional installed console command remains `raft-workflow`.

The former Verus installation was under `/tmp/raft-verus-tools/`; that directory was missing. The evidence does not identify who removed it or when. Verus, bundled vstd/Z3 and isolated Rust 1.92.0 have now been restored under `/home/nitro/.local/share/raft-verus/`, and `workflow.toml` points to that persistent location. The default Rust remains 1.88.0. No shell profile, global PATH or system libc was changed. See [installation records](../evidence/layout-and-tools/installation.json).

The process runner, independent checking, budgets, path protection and external human approval remain in place. No new hash protocol, frozen contract, baseline or gate was added. Existing approvals/evidence do not automatically carry over to changed paths and tool declarations. Previous raw evidence is preserved unchanged; obsolete authoring directories have not been recreated.

## Checks for this revision

[Command records](../evidence/layout-and-tools/checks.json) contain actual arguments, times, exit codes and raw output paths. All commands below were executed with the new layout. No real Codex generation was launched.

| Check | Actual result | Evidence |
|---|---|---|
| Offline Python suite | 56 tests passed; exit 0. Fake subprocess results remain SIMULATED | [Test output](../evidence/layout-and-tools/offline-suite.stderr) |
| Offline package installation | Installed module, console command and wire resources work from outside the repository | [Module](../evidence/layout-and-tools/installed-validate.stdout), [command](../evidence/layout-and-tools/installed-command.stdout), [resources](../evidence/layout-and-tools/installed-resources.stdout) |
| Validation and plan | Structural PASS; eight tasks remain unapproved | [Validation](../evidence/layout-and-tools/final-validate.stdout), [plan](../evidence/layout-and-tools/final-plan.stdout) |
| Prompt preview and dry-run | All eight tasks succeeded; 0 real Codex generation calls | [Example prompt](../evidence/layout-and-tools/prompt-T01-MODEL.stdout), [dry-run](../evidence/layout-and-tools/dry-run-T01-MODEL.stdout); all eight commands are indexed |
| Real positive Verus smoke | PASS: 2 verified, 0 errors; Verus exit 0 | [Independent result](../evidence/layout-and-tools/verify-smoke-positive.stdout) |
| Real negative Verus smoke | Expected rejection: 0 verified, 1 error; Verus exit 1 | [Independent result](../evidence/layout-and-tools/verify-smoke-negative.stdout) |
| Real Verus model check | TYPECHECKED: 0 verified, 0 errors; Verus exit 0; no Raft obligation discharged | [Independent result](../evidence/layout-and-tools/verify-spec.stdout) |
| Rust helper tests / formatting | 2 tests passed / formatting passed; exit 0 | [Tests](../evidence/layout-and-tools/rust-tests.stdout), [formatting](../evidence/layout-and-tools/rust-format.stderr) |
| Independent Rust interface check | PASS, build only | [Result](../evidence/layout-and-tools/verify-interface-check.stdout) |
| Full Raft check | BLOCKED before verifier launch; implementation/proofs/isolation are incomplete | [Result](../evidence/layout-and-tools/verify-full.stdout) |
| Saved evidence report | Read successfully; full acceptance remains false | [Report](../evidence/layout-and-tools/report.stdout) |

Directory names and import paths changed; proof targets, assumption definitions and task scopes are unchanged. All 26 proof obligations remain OPEN. The Verus source change only updates its documentation path. The [final layout/reference audit](../evidence/layout-and-tools/final-layout-audit.stdout) passed; it also checked English materials, local links and persistent executable paths.

## Historical Verus evidence

These are actual bootstrap results from 2026-10-26, not new checks of this revision:

| Check | Recorded result | Evidence |
|---|---|---|
| Positive smoke | 2 verified, 0 errors; exit 0 | [Independent record](../evidence/bootstrap/final-verify-smoke-positive.stdout) |
| Negative smoke | Expected rejection; Verus exit 1 | [Independent record](../evidence/bootstrap/final-verify-smoke-negative.stdout) |
| Raft model | TYPECHECKED: 0 verified, 0 errors; no discharged Raft obligation | [Independent record](../evidence/bootstrap/final-verify-spec.stdout) |
| Python/Rust tests | 53 offline Python tests and 2 Rust helper tests passed | [Command index](../evidence/bootstrap/checks.json) |

The authorized official Verus release was `0.2026.01.23.1650a05` (commit `1650a05d64219fdd1d150f32d0d7d5b19b8defcd`), with bundled vstd, isolated Rust 1.92.0 and Z3 4.12.5. Default Rust was unchanged. Earlier installation/API failures remain in `evidence/model-*`. The temporary `/tmp/raft-verus-tools/` directory was absent on 2026-10-27; the persistent restoration and new real checks are recorded above. [Tool setup](../docs/tools.md) gives the commands and provenance. verus-tla compatibility and a full transitive-license audit remain NOT_RUN/BLOCKED.

## Open work and decisions

The model defines actual Init/Next and target predicates. A complete inductive invariant, temporal environment encoding, interface error semantics, executable Raft core, cluster composition, safety proofs, liveness proofs and executable refinement remain incomplete. Natural-number models do not establish bounded-machine liveness. Ordered delivery does not promise arbitrary exactly-once application effects.

All 26 obligations and their dependencies are in [proofs.md](../input/proofs.md). The pending decisions cover API/coordinator boundaries, finite membership, checked integers/resources, conditional liveness, request retries, application recovery, storage/TCB, tools, isolation and external approval. Recommendations are preserved for human review; simplification approves none of them.

The full profile is intentionally BLOCKED. The `spec` profile cannot complete T01's proof obligations: typechecking is insufficient. Before future generation, review local proof entrypoints/completion criteria, boundaries/assumptions/TCB, checker policy, tool setup and isolation. The local controller is not resistant to hostile same-user processes; formal execution needs external permission isolation. Fake fixture results remain SIMULATED and cannot close real obligations.

## Commands

From the repository root:

```sh
python3 -m orchestrator validate
python3 -m orchestrator plan
python3 -m orchestrator prompt --task T01-MODEL
python3 -m orchestrator run-task --task T01-MODEL --dry-run
python3 -m unittest discover -s orchestrator/tests -v
cargo test --workspace --offline
cargo fmt --all -- --check
python3 -m orchestrator verify --profile interface-check
python3 -m orchestrator verify --profile spec
python3 -m orchestrator verify --profile full
python3 -m orchestrator report --run-id ACTUAL_RUN_ID
```

`spec` uses the restored tools and only establishes model typechecking. `full` reports the known block. No real Codex task, recursive agent run, automatic approval, merge or release was performed. Stop here for human review.

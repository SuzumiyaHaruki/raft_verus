# Generate and verify Rust Raft

Status: DRAFT. Python launches Codex to complete a supplied MadRaft-inspired MadSim foundation and generate matching Verus specifications and proofs. The goal is a fixed-membership Rust Raft implementation satisfying Election Safety, Leader Append-Only, Log Matching, Leader Completeness, State Machine Safety and conditional election/commitment/delivery liveness. Maintenance prepares materials and independent checks; it does not preimplement the experiment.

## Start a new experiment

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
./experiment.sh                 # Preview only: zero model calls
./experiment.sh run             # A NEW experiment; gpt-6.1-sol / high
./experiment.sh resume RUN_ID   # Explicit continuation of that experiment
./experiment.sh report RUN_ID   # Inspect recorded results
```

Every run creates a unique `evidence/runs/RUN_ID/candidate/` from the supplied foundation. It inherits no previous experiment's implementation, proof strategy, reports or budget usage. Stages and repairs within one experiment share that candidate. Only resume reuses it. The source checkout is not overwritten or automatically merged. Human review follows delivery, with no approval file or interactive checkpoint inside execution.

The default selects STATE-INIT → RUNTIME-PERSIST → MADRAFT-VOTE → MADRAFT-ELECTION → MADRAFT-REPLICATION → MADRAFT-RECOVERY. Python-launched Codex generates initialization, persistence coordination, voting and the remaining protocol behavior with matching local specifications/proofs. The supplied MadSim host provides transport, storage and scheduling; it contains no completed Raft protocol solution. See [starting-point.md](input/starting-point.md) and the [module roadmap](input/roadmap.md).

**The current workflow is not complete Raft acceptance.** The selected steps check bounded behavior and local initialization, persistence, vote, merge and durable-reply proofs. Independent whole-execution refinement, global safety and conditional-liveness consumers remain incomplete; `raft-safety` and `full` report BLOCKED. All global obligations remain OPEN. Finite MadSim histories and successful helper proofs cannot replace these required results.

## Direct offline checks

```sh
python3 -m unittest discover -s orchestrator/tests -v
python3 -m orchestrator verify --profile madraft-poc-runtime
python3 -m orchestrator verify --profile madraft-poc-verus
python3 -m orchestrator verify --profile core-vote-tests
python3 -m orchestrator verify --profile core-proof
python3 -m orchestrator verify --profile full
```

The scaffold builds, but initialization, persistence, voting and `handle_event` fail their independent behavior/proof checks until generated. Previous vote-slice successes do not describe this starting point. Cargo and Verus check the same executable source with the installed matching tools. Offline fixtures simulate generation and remain outside candidates; actual Rust/Verus checking is recorded separately. Independent targets remain supplied inputs; offline solutions remain excluded from candidates.

## Directories

| Path | Contents |
|---|---|
| `input/` | English requirements, safety/liveness targets, assumptions, interfaces and bounded module assignments |
| `crates/raft-verified/` | Protocol scaffold, generation slots, independent behavior tests and local replay/measurement programs |
| `crates/raft-spec/`, `crates/raft-proofs/` | Formal targets, independent same-source consumers and auxiliary proof files |
| `experiments/madraft-poc/` | Actual pinned MadSim host, transport/storage/timer adapters, fault scenarios and replay |
| `orchestrator/` | Python invocation, checking, budgets, repair history and CLI; `tests/` contains offline regressions |
| `docs/` | Current workflow, tool setup and useful source references |
| `review/` | Current maintenance review and pending semantic decisions; obsolete reports are removed |
| `evidence/` | Current calibration/check logs and separately created experiment runs; no historical maintenance archive |
| `target/madraft-cargo/` | Required offline dependency cache, excluded from candidates; other build products are disposable |

`Agent.md` is maintenance-only. `AGENTS.md` instructs experiment candidates. `workflow.toml` selects tools, model, tasks, scopes and checks; `experiment.sh` is the launch shortcut. Local metadata, maintenance reports, evidence, build products and offline solutions are not candidate inputs. See [workflow details](docs/workflow.md), [tools](docs/tools.md) and the source checkout's [current review](review/REVIEW_PACKET.md).

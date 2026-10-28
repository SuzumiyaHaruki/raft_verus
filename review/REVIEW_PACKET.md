# Generate voting and persistence during the experiment

Status: DRAFT. The user clarified that voting and protocol persistence must also be implemented and verified by Python-launched Codex. The previous maintenance-authored vote slice was incorrectly retained as experiment input. It has now been removed. No real Codex generation, recursive agent, merge, push, download or global toolchain change occurred.

## Changes

- `crates/raft-verified/src/madraft.rs`: deleted the completed vote and durable-completion algorithms and their implementation proof bodies. Retained only the required API vocabulary and explicit unimplemented bodies with local contract seeds. The independent vote/completion targets remain unchanged.
- Removed the separate `feasibility_follower` constructor. Both MadSim hosts, protocol tests and the independent vote caller now use the shared, unimplemented `Node::new`. Test expectations and constructor postconditions are retained. The vote proof caller imports the actual constructor/contracts/persistence source, so generated helper calls participate in real verification.
- `workflow.toml`: reuse STATE-INIT and RUNTIME-PERSIST as default prerequisites, then run MADRAFT-VOTE → MADRAFT-ELECTION → MADRAFT-REPLICATION → MADRAFT-RECOVERY in one fresh candidate. MADRAFT-VOTE replaces the former integration-only assignment and requires the independent initialization/vote/completion targets as well as the public durable-reply target. Later changes recheck earlier completed tasks using the existing controller.
- `Agent.md`, `AGENTS.md`, `input/`, README and supporting documentation now distinguish supplied host infrastructure from generated protocol code. All protocol behavior, including initialization, voting and persistence coordination, is generated within the recorded run. LLM-facing text is English; Markdown prose paragraphs remain on one physical line.
- `experiments/madraft-poc/negative.py`: the controlled defect demonstration now requires an already passing generated candidate. It never restores a maintenance solution. Its edit is SIMULATED; its checks are real. A missing implementation stops the demonstration with its actual failed prerequisite evidence.

The MadSim host still executes transport, timers, atomic whole-image writes and application/cursor transactions. These are disclosed environment mechanisms. Decisions about term/vote/log updates, pending operations and authorized replies belong to generated Raft code. The host is inspired by the pinned MadRaft reference; no upstream MadRaft protocol implementation is imported. See [host provenance](../experiments/madraft-poc/NOTICE.md).

The checkout's `.git` metadata is not usable as a Git repository (`git status` reports “not a git repository”). It was left untouched. Existing raw evidence was preserved, but the old vote-slice PASS results are not results for this new scaffold. No implementation archive was created.

## Actual results

Current evidence is under [evidence/checks/protocol-scaffold](../evidence/checks/protocol-scaffold/). [summary.json](../evidence/checks/protocol-scaffold/summary.json) records each profile's actual argv, tool versions, result and diagnostic paths. Temporary builds were removed after checking.

| Check | Actual result |
|---|---|
| `python3 -m unittest discover -s orchestrator/tests -v` | **122 passed**, no skips, 55.273 seconds. [Log](../evidence/checks/protocol-scaffold/controller.log). Includes the six-step preview and real Verus rejection of missing vote/completion behavior even after SIMULATED initialization is supplied in a temporary test candidate. |
| `python3 -m orchestrator validate` | **PASS**, structural checking only. |
| `./experiment.sh` | **DRY_RUN**, STATE-INIT → RUNTIME-PERSIST → MADRAFT-VOTE → MADRAFT-ELECTION → MADRAFT-REPLICATION → MADRAFT-RECOVERY; zero model calls, gpt-6.1-sol/high. |
| `interface-check`, `madraft-poc-build` | **PASS**, actual Rust and pinned MadSim builds of the scaffold. |
| `node-tests` | **FAIL**, 4 passed / 4 failed: the unimplemented constructor rejects valid memberships. |
| `node-proof` | **FAIL**, 23 verified / 1 error: independent initialization target rejected. |
| `persist-tests`, `persist-proof` | **FAIL**, 0 passed / 6 failed; Verus 29 verified / 3 errors. Runtime cases stop at the missing constructor; the independent coordinator caller rejects the unimplemented persistence contracts. |
| `madraft-poc-runtime`, `madraft-poc-verus` | **FAIL**, 0 passed / 8 failed; Verus 29 verified / 3 errors. Runtime scenarios stop at construction; formal constructor/vote/completion consumers reject the missing implementation. |
| `core-vote-tests`, `core-proof` | **FAIL**, 0 passed / 3 failed; Verus 30 verified / 1 error. The public event path remains unimplemented. |
| Direct `negative.py` demonstration | **BLOCKED**, exit 2 after actual runtime and Verus prerequisite failures. No defect/repair success is claimed; those phases did not run. [Prerequisite evidence](../evidence/checks/protocol-scaffold/negative-control/summary.json). |
| `raft-safety`, `full` | **BLOCKED**, missing independent global proof entrypoints. |

These failures are the required rejection of an unfinished starting scaffold, not successful protocol verification. Verification counts include dependencies. The initialization regression uses an explicitly SIMULATED offline fixture excluded from experiment candidates; it supplies no vote solution and actual Verus still rejects vote/completion. Original diagnostic output is retained without translation or fabrication.

## Remaining work and decisions

All 26 global obligations remain OPEN. Election Safety, Leader Append-Only, Log Matching, Leader Completeness, State Machine Safety, conditional election/commitment/delivery liveness and executable refinement are not proved. The current selected steps still lack complete independent whole-execution safety/liveness consumers; this revision does not claim to close that separate workflow gap. Formal assumptions, host storage/application semantics, capacity bounds, TCB and independent-checker isolation remain DRAFT in [OPEN_DECISIONS.md](OPEN_DECISIONS.md). No target or environment assumption was weakened to obtain a pass.

A passing generated vote candidate will be needed to exercise the revised controlled defect and repair phases. That requirement is not satisfied by this scaffold; the demonstration currently records only the genuine failed prerequisites.

## Commands

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m unittest discover -s orchestrator/tests -v
./experiment.sh
python3 -m orchestrator verify --profile madraft-poc-build
python3 -m orchestrator verify --profile node-proof
python3 -m orchestrator verify --profile persist-proof
python3 -m orchestrator verify --profile madraft-poc-verus
python3 -m orchestrator verify --profile core-proof
./experiment.sh run
./experiment.sh resume RUN_ID
./experiment.sh report RUN_ID
```

`run` is the next explicit generation command and was not executed here. It starts with STATE-INIT in a fresh directory, proceeds to RUNTIME-PERSIST, then voting and the remaining configured steps. Within a passing generated candidate, `python3 experiments/madraft-poc/negative.py` tests the controlled early-reply defect. Review follows delivery; no interactive checkpoint or approval file was added to execution.

# Raft workflow review packet

Status: DRAFT. Updated 2026-10-27. Review delivered files with the user after this turn. No new Raft semantics, assumptions or correctness claims are approved.

## What changed

The maintenance objective is now in [Agent.md](../Agent.md), linked by AGENTS.md: AI generates one coupled Rust Raft implementation; Verus must check the actual executable against adequate targets. Develop code/proof together, compare results, repair in small steps, and retain global safety/liveness/refinement obligations. Agent.md is excluded from experimental candidates and prompt context.

The launcher now selects **STATE-INIT**, not the four existing helper steps. It requests `gpt-6.1-sol` with `high`, at most three calls/two repairs, 600 seconds per call and 2,400 seconds per task. Preview starts no agent. Human review remains after delivery; no runtime approval file or interactive checkpoint was introduced.

| Files | Concrete change |
|---|---|
| `input/state.md`, `workflow.toml`, `experiment.sh` | Runnable assignment to implement/prove `Node::new`, with independent checks and bounded scope |
| `crates/raft-verified/src/state.rs`, `src/lib.rs` | Shared initial Node fields and existing DurableState reexport; later handlers extend this same state |
| `crates/raft-verified/src/node.rs` | Explicit unfinished reject-all generation seed; builds but cannot pass initialization acceptance |
| `crates/raft-spec/src/node_init.rs`, `crates/raft-proofs/node_init.rs` | Protected initial-field relation and consumer of the exact Cargo constructor; valid inputs must succeed |
| `crates/raft-verified/tests/node_init.rs`, `examples/initialize.rs`, `check.py` | Eight independent tests, initial-state replay, actual Verus invocation and construction measurements |
| `orchestrator/scheduler.py` | Carry prior attempts, changed paths, independent outcomes and artifact locations into repair prompts; explanations remain agent claims |
| `orchestrator/workspace.py`, `tests/` | Exclude maintenance instructions and scripted solution; regressions for default task, history and invalid proof contracts |
| `orchestrator/tests/real_repair_demo.py` | Reuse the existing subprocess demo for wrong-term rejection and repair with the complete STATE-INIT check selection |
| `README.md`, `input/roadmap.md`, `input/proofs.md`, `input/trust.md`, `docs/`, `OPEN_DECISIONS.md` | Current commands, scope, correspondence limits and pending API/checker decisions |

No model transition or environmental assumption changed. No new dependency, hash, frozen contract, general gate or agent framework was added. Existing scope, mutation/evidence checks, sandboxing and no-merge behavior remain. The exact new ghost-only `init_result` import is recognized by the existing proof-source scanner; other executable conditional compilation remains disallowed. This adjustment and the new independent targets are DRAFT review items under ADR-08/10.

## Actual checks and delivered candidate

| Check | Actual result |
|---|---|
| Controller suite | 95 passed, no skips; includes real Verus rejection of reject-all, weaker postconditions and stronger preconditions |
| Validation / launcher / materials | PASS; default selects only STATE-INIT, zero preview calls; all 25 prompts English with one selected step |
| Source checkout | Build PASS; original 15 Rust regressions PASS; new node tests FAIL and node proof 11 verified / 1 error, intentionally exposing the unfinished constructor |
| Scripted wrong-term candidate | Rust tests FAIL, initial replay FAIL, actual Verus 13 verified / 1 error; measurement deferred |
| Scripted repaired candidate | All 23 Rust tests PASS, replay four observations PASS, node Verus 14 verified / 0 errors, reconciliation Verus 9 / 0 |
| Model definitions | TYPECHECKED, 0 verified; closes no theorem and does not prevent other checks passing |
| Constructor measurements | Four workloads (1/3/5/31 members), five samples each; MEASURED, no threshold or regression claim |
| Whole project | BLOCKED: complete proof entrypoint missing; all 26 global obligations remain OPEN |

Generation in this demonstration is **SIMULATED**, checking is **REAL_LOCAL**. There was no live Codex call, recursion, merge, push or publication. This tests the controller/checking loop, not whether a real model can solve the assignment. Non-Copy Clone warnings remain; constructor proofs do not use those Clone contracts. The 14-function count includes dependencies/derived functions, not 14 Raft theorems.

Read the [scripted run summary](../evidence/core-objective/node-repair-final/summary.json), [checked constructor](../evidence/core-objective/node-repair-final/source/evidence/runs/20261027T074617Z-6c663c2ca961/candidate/crates/raft-verified/src/node.rs), [controller test log](../evidence/core-objective/controller-tests-1.log), [source check log](../evidence/core-objective/source-checks.log), and [material audit](../evidence/core-objective/material-audit.log). The candidate and both attempts' patches, raw outputs and repair prompt are retained under the run named in the summary. The passing candidate has not overwritten the source seed. Previous helper/bootstrap evidence remains unchanged.

Markdown prose paragraphs now occupy one physical line throughout maintained documents. Code blocks, tables and lists retain their structure. Historical evidence is preserved; this formatting change does not alter Rust/Verus source. Structural validation and 19 module/registry tests passed again; see the [format checks](../evidence/core-objective/markdown-checks.log). The [maintenance diff](../evidence/core-objective/changes.patch) compares this delivery with the preserved starting files.

## Commands

```sh
export PYTHONDONTWRITEBYTECODE=1
./experiment.sh                         # Preview; no generation
./experiment.sh run                     # Explicit real STATE-INIT experiment, not run here
./experiment.sh resume RUN_ID
./experiment.sh report RUN_ID
python3 -m unittest discover -s orchestrator/tests -v
python3 orchestrator/tests/real_repair_demo.py --task STATE-INIT --output /tmp/raft-node-demo
```

Use a new output directory for the offline demo. To inspect/recheck the delivered candidate from this repository root:

```sh
cd evidence/core-objective/node-repair-final/source/evidence/runs/20261027T074617Z-6c663c2ca961/candidate
python3 crates/raft-verified/check.py test
python3 crates/raft-verified/check.py node-proof
python3 crates/raft-verified/check.py node-replay
python3 crates/raft-verified/check.py node-bench
```

## Remaining work and decisions

Next runnable generation task: **STATE-INIT**, with a deliberately unfinished source constructor. After review/integration, RUNTIME-PERSIST must extend the same Node; its pending-write representation and independent oracles still need development. No complete dispatcher, election, replication service, commit/delivery, crash recovery or cluster fault experiment is delivered by initialization. Its local relation checks represented initial fields only: no fictitious view of missing pending I/O/peer state, no full cluster initialization/refinement, and no liveness.

[Pending decisions](OPEN_DECISIONS.md) include fresh-storage-only construction, unordered distinct membership, incarnation allocation on recovery, new checker scope, storage/adapters and temporal proof tooling. All remain DRAFT. Allocation failure may abort. Compatible temporal tooling and hostile-process isolation remain missing. Real account/model access is NOT_RUN; system Rust/global Codex settings and the four original read-only directories are unchanged.

IDS informed joint code/proof development and feedback-guided repair, as documented in [sources](../docs/sources.md). No full IDS implementation or Raft correctness claim is made. Stop here for human review.

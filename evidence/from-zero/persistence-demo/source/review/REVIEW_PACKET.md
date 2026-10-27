# Raft continuation revision review packet

Status: DRAFT. Delivered 2026-10-27 for human review after delivery. No new Raft semantics, global assumptions or formal-correctness claim is approved. No real Codex task, recursive agent, merge, push or publication was performed.

## Inspected revision and preserved work

Maintained files initially matched `7edcfa2985b848bffcf7949c139c906c84cfc956`; no newer authored changes were found. The pre-edit backup is `/tmp/raft-next-revision-76hxlxeb/authored.tar.gz`. The maintenance workspace's read-only `.git`, `.agents`, `.codex` and `.aws` remain untouched. Historical evidence remains unchanged. See the [maintained-file diff](../evidence/next-revision/changes.patch) and [changed-file list](../evidence/next-revision/changed-files.json).

The default STATE-INIT experiment, its intentional reject-all constructor, module assignments, same-source Cargo/Verus checking, protected independent targets, scoped edits, subprocess limits and review-after-delivery policy are preserved. The scripted passing constructor remains excluded from experiment candidates. There are 26 named steps, seven ready and 19 planned; every complete Raft module remains incomplete. All 26 global proof obligations remain OPEN.

## Main delivered files

| Files | Result |
|---|---|
| `orchestrator/scheduler.py`, `continuation.py`, `cli.py` | Persistent per-task/plan budgets, attempt/design history, scoped checkpoints, explicit budget extension/rewind, bounded redesign and repair of affected completed tasks |
| `orchestrator/feedback.py`, `performance.py`, `verifier_runner.py` | Distinct functional/scenario/proof/timeout/setup diagnoses; optional comparable-reference evaluation and optimization through correctness rechecks |
| `orchestrator/toolchain.py`, `config.py`, `crates/raft-verified/check.py`, `workflow.toml` | One toolchain selection for runtime and proof, an explicit installation-root override, resolved paths and preserved version calibration |
| `crates/raft-verified/src/state.rs`, `io.rs`, `persistence.rs` | The same Node now holds one pending write and a fail-stop flag; actual submission, completion handling and partial Event dispatcher |
| `crates/raft-spec/src/persistence.rs`, `crates/raft-proofs/persistence.rs`, `persistence_lemmas.rs` | Exact executable contracts, protected consumers and submission/successful-callback storage projection |
| `crates/raft-spec/src/progress.rs`, `crates/raft-proofs/progress.rs`, `progress_lemmas.rs` | Early conditional progress theorem over actual dispatcher contracts, with an explicit service/noninterference premise |
| `crates/raft-verified/tests/persistence.rs`, `examples/persistence.rs`, `examples/persistence.trace` | Six independent coordinator tests, 14-event repeatable host scenario and release microbenchmark |
| Initialization specification, tests, proof entrypoint and offline fixture | New pending fields included; valid membership still must succeed; the production constructor seed remains unchanged |
| `orchestrator/tests/test_continuation.py`, `test_performance.py`, `test_toolchain.py`, `test_node_contract.py` | Continuation, regression, comparison, relocation and exact-target regressions, extending the existing suite |
| `input/`, `docs/`, `review/OPEN_DECISIONS.md` | Concrete assignments, early dispatcher/progress dependencies, pinned research references, commands and pending choices; English prose paragraphs occupy one physical line |

## Actual checks

Generation in retained demonstrations is **SIMULATED**. Rust/Verus/scenario checking in the persistence demonstration is **REAL_LOCAL**. Controller comparison fixtures use synthetic measurements and make no Raft proof or real performance-regression claim.

| Check | Actual result and evidence |
|---|---|
| Final controller suite | **110 passed, no skips**; [full log](../evidence/next-revision/controller-delivery-final.log). Includes actual relocated build/proof with spaces in tool paths and rejection of weakened executable contracts/strengthened preconditions |
| Continuation examples | Six retained examples PASS: exhausted resume, explicit extension and redesign, interrupted launched call, cross-task regression repair during a run and after interruption, scoped rewind, and optimization that first breaks correctness; [summary and artifact directories](../evidence/next-revision/continuation-delivery/summary.json) |
| Source checkout | Build PASS and original 15 runtime regressions PASS. Constructor tests intentionally FAIL for valid memberships; constructor proof is **20 verified / 1 error**. Model is **TYPECHECKED, 0 verified**; [source checks](../evidence/next-revision/source-checks.json) |
| Source persistence/progress proofs | Persistence **29 verified / 0 errors**, including independent callback-projection consumer; [final persistence report](../evidence/next-revision/persist-proof-final.json). Progress **26 / 0**; [progress report](../evidence/next-revision/progress-proof.json) |
| Controlled wrong-kind callback defect | Compilation PASS, coordinator tests and replay FAIL, actual Verus **28 / 1**, with the intended postcondition failure rather than a compiler failure. Measurements deferred |
| Repaired shared candidate | **29 Rust tests PASS**, six coordinator tests PASS, replay **14 events PASS**, initialization proof **23 / 0**, persistence proof **29 / 0**, reconciliation proof **9 / 0**; [complete demonstration](../evidence/next-revision/persist-delivery/summary.json) |
| Measurements | Four initialization workloads and one coordinator workload, five samples each, MEASURED/INCONCLUSIVE. No real reference or performance target configured. Synthetic comparison tests separately distinguish measurement-only, noise and reproduced regression |
| Materials/default launcher | Structural validation PASS; 26 English prompts; constructor seed unchanged; default preview makes zero agent calls. [Material audit](../evidence/next-revision/material-audit-final.json), [validation](../evidence/next-revision/validate-final-delivery.json), [combined preview](../evidence/next-revision/combined-preview-final.json) |
| Whole Raft | BLOCKED: whole-project proof entrypoint, protocol handlers, recovery/cluster integration and global safety/liveness/refinement proofs remain missing |

The final shared candidate is [here](../evidence/next-revision/persist-delivery/source/evidence/runs/20261027T103406Z-192085495b62/candidate). It took three scripted calls across STATE-INIT and RUNTIME-PERSIST, with one repair. The repair changed executable code and a local proof together; independent requirements and tests stayed fixed. Attempt patches, cumulative patches, actual argv, outputs, check results and persisted usage are retained under the run IDs in its summary. Successful completed-step evidence was invalidated by the later defect and restored only after affected checks passed.

Function totals include dependencies and derived functions, not counts of Raft properties. Non-Copy Clone warnings remain; these local proofs do not use those missing Clone contracts. Intermediate debug logs are retained. An early negative test incorrectly expected weakening an auxiliary lemma to fail even when the independent consumer could re-prove the full target automatically; the delivered negative test instead weakens the executable contract or strengthens its precondition and verifies rejection.

## What the results mean

Submission reserves a fresh node-scoped token and retains a proposed image/reply; it is not durable success. The host scenario records its disk update separately from callback delivery. Only a matching successful Persist completion returns the retained reply. Busy, identity/kind mismatch, duplicates, uncertain outcomes and exhaustion have explicit behavior. The partial dispatcher rejects every unsupported Event variant. This is a coordinator/completion scenario, not a crash-recovery or cluster fault test.

The storage projection relates represented fields and successful submission/completion to the existing model's `write`/`storage_ack`. It does not prove arbitrary supplied term/log changes legal Raft actions. Future election/replication callers must establish their actual transition guards, preserve committed state and authorize replies; recovery must reconstruct the real durable image with a fresh incarnation. Real adapters, general Effect/Send integration, full error/recovery refinement and cluster composition remain OPEN.

The progress theorem says that a matching successful callback receiving service after finitely many noninterfering events releases its pending reply. Its premises constrain inputs and require actual dispatcher-step contracts, not eventual output. Installed Verus/vstd handles this unbounded-nat trace encoding and decreasing prefix induction. External verus-tla is unavailable/unverified. Global election, commitment and delivery liveness, finite-resource adequacy, interference and fairness transfer remain mandatory and OPEN.

## Continuation, tools and pending choices

Calls and bounded time reservations are persisted before execution. Normal completion settles actual process time; interruption conservatively retains the reservation, while paused wall time is not charged. Resume retains history and cannot replenish caps. An explicit `extend-plan` records extra allowance; it is not an approval checkpoint. Repeated failing targets trigger at most the configured redesign allowance. Rewind affects only the controller candidate, retains abandoned changes and reruns affected checks. Mixed historical generation labels remain distinct from check provenance.

[Tool observations](../evidence/next-revision/doctor.json): Verus `0.2026.01.23.1650a05`, Rust/Cargo `1.92.0`, Z3 `4.12.5`, Python `3.10.12`, Codex CLI `0.162.0-alpha.17.2`. The system default Rust is still `1.88.0`. Configuration requests `gpt-6.1-sol` / `high`; account/model access is NOT_RUN. No tools were downloaded or global defaults changed. `workflow.toml` plus `RAFT_TOOLCHAIN_ROOT` selects the installation for both checking paths; [tools.md](../docs/tools.md) explains configuration and relocation.

[Pending decisions](OPEN_DECISIONS.md) remain DRAFT: Busy for a pending write; conservative fail-stop handling of both NotPerformed and Unknown with distinct errors; host snapshot/notification ownership; interim optional-message dispatcher output versus final Effects; legal caller/refinement boundaries; recovery incarnation allocation; temporal-library integration and global fairness/resource premises. No performance rule is approved. Local mode still does not provide hostile-process OS isolation. [sources.md](../docs/sources.md) records the exact IDS-derived and ConsensusArena/ConsensusWiki files/revisions read, the concrete mechanisms adopted, and uninspected Laputa/ConsensusVerifier material.

## Commands and next runnable work

The following inspection/check/demo commands were run locally; they do not launch real agents. Use a new output directory for each demonstration.

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 -m orchestrator validate
python3 -m unittest discover -s orchestrator/tests -v
python3 orchestrator/tests/continuation_demo.py --output /tmp/raft-continuation-review
python3 orchestrator/tests/real_repair_demo.py --task RUNTIME-PERSIST --output /tmp/raft-persistence-review
python3 -m orchestrator verify --profile persist-proof
python3 -m orchestrator verify --profile progress-proof
python3 -m orchestrator run-plan --tasks RUNTIME-PERSIST --dry-run
./experiment.sh
```

Recheck the delivered candidate directly from this repository root:

```sh
cd evidence/next-revision/persist-delivery/source/evidence/runs/20261027T103406Z-192085495b62/candidate
python3 crates/raft-verified/check.py test
python3 crates/raft-verified/check.py node-proof
python3 crates/raft-verified/check.py persist-proof
python3 crates/raft-verified/check.py persist-replay
python3 crates/raft-verified/check.py progress-proof
python3 crates/raft-verified/check.py persist-bench
```

The next real experiment still begins with STATE-INIT. After reviewing this delivery, the user can explicitly run the combined path below; it carries the checked initializer into RUNTIME-PERSIST without copying or merging a solution. Selecting PROGRESS-FOUNDATION instead also includes the early progress task. These live commands were **not** run during maintenance.

```sh
python3 -m orchestrator --model gpt-6.1-sol --reasoning-effort high run-plan --tasks RUNTIME-PERSIST --execute-codex
python3 -m orchestrator resume --run-id RUN_ID --execute-codex
python3 -m orchestrator extend-plan --run-id RUN_ID --task RUNTIME-PERSIST --calls 1 --repairs 1 --seconds 600
python3 -m orchestrator rewind --run-id RUN_ID --task RUNTIME-PERSIST --check persist-tests
```

Delivery stops here for human review. There is no approval file, interactive approval checkpoint or automatic candidate integration.

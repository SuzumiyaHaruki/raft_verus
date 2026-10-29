# Current result — DRAFT, 2026-10-28

**Repository preparation delivered; the code-and-proof experiment has NOT been run.** No real Codex generation, recursive agent, merge, push or publication occurred. The supplied project remains a student scaffold, not a completed Raft implementation.

The old custom crates, fixed task pipeline, duplicate interfaces, feasibility implementation, controller and historical reports/evidence were removed. The remaining authored tree is `madraft/`, `requirements/`, `docs/`, `scripts/`, README, AGENTS and `tools.ini`. `runs/` and build/dependency caches are ignored. Metadata, credentials and global toolchains were untouched. Source identities, the checkout limitation and adaptation details are in the operator-only [provenance.md](docs/provenance.md), which is not copied into experiments.

The actual starting point is MadRaft `8ec4565ecd3844f54f52865f2d7b3da4044f657f` with its Apache-2.0 license and all four lab families. MadSim is explicitly upgraded to `519950efb4711464f300ed7edf2967ed62d5f502` / 0.2.34 after the original dependency failed on Rust 1.92 with E0512. [AGENTS.md](AGENTS.md) gives one joint implementation/proof instruction. [Safety definitions and callers](requirements/safety.md) cover all five properties, snapshots and full executable correspondence. The generator chooses the internal implementation and proof strategy.

New experiments are restricted to fresh `runs/<name>/` directories. The launcher copies the scaffold and starts the agent there; implementation edits remain in that workspace's `madraft/`, and explicit resume preserves the same copy. The repository-level README is operator documentation and is no longer copied. The supplied AGENTS instruction now includes the agent's check commands. Only `check.py`, `process.py` and `tools.py` accompany the candidate; the launcher and maintenance tests stay in the preparation checkout. Offline regressions exercised edits in the candidate, confirmed the original source stayed unchanged, and checked that a second fresh experiment inherited no earlier edits. These are file-copy and invocation checks, not a claim of additional OS isolation.

Prompt audit revision: removed the preparation-complete statement, launcher lifecycle explanations, single-agent restrictions and maintenance-only design prohibitions from experiment instructions. New runs use their own persistent `CODEX_HOME` without the operator's personal configuration or AGENTS files; only authentication and execution rules are linked. The prior effective model, `gpt-6-astra`, is now explicit in `tools.ini` with unchanged `high` reasoning. The offline isolation tests use SIMULATED processes and fake credentials; no real agent or authentication request was run.

| Actual command/check | Result | Local raw evidence |
| --- | --- | --- |
| Original upstream `cargo test --offline --no-run`, Rust 1.92 | FAILED in MadSim 0.1.3: 128-bit `TypeId` transmute to `u64` | `runs/preparation/upstream-build.log` |
| `python3 scripts/check.py build` | PASS: adapted student project and tests compile, locked/offline | `runs/preparation/build.log` |
| `python3 scripts/check.py inventory` | 57 upstream test declarations retained, plus 3 host regressions; one upstream empty test is ignored | `runs/preparation/inventory-current.log` |
| `python3 -m unittest discover -s scripts/tests -v` | 13 PASS: 11 offline controller/process tests and 2 actual Verus rejection tests | `runs/preparation/workspace-input-tests.log` |
| `python3 -m unittest discover -s scripts/tests -p test_helpers.py -v` | 12 PASS: offline controller/process tests, including personal-instruction exclusion and persistent isolated-home resume; agent calls SIMULATED | `runs/preparation/prompt-isolation-tests.log` |
| `python3 scripts/check.py test test_support:: --seed 1` | 3 PASS: actual MadSim RPC fault/reconnect, task cancellation and restart/file preservation | `runs/preparation/host-tests.log` |
| `MADSIM_TEST_CHECK_DETERMINISTIC=1 python3 scripts/check.py test test_support:: --seed 1` | 3 PASS with deterministic replay checking | `runs/preparation/deterministic-host.log` |
| `python3 scripts/check.py definitions` | Verus: 9 verified, 0 errors; target-definition regressions only | `runs/preparation/definition-command.log` |
| `python3 scripts/check.py proof` | Verus: 3 verified, 0 errors; existing same-source role observer/defaults only | `runs/preparation/same-source-proof.log` |
| Deliberately wrong role observer; false two-leader safety assertion | Both rejected by real Verus, respectively postcondition and assertion failure | `runs/preparation/negative-accessor.log`, `runs/preparation/negative-election.log` |
| `python3 scripts/check.py final --timeout 120` | INCOMPLETE: 3 host tests passed, 56 student tests failed, 1 upstream placeholder ignored; actual aggregate compilation fails because generated `implementation.rs` is absent | `runs/preparation/final-check.log` |
| `python3 scripts/check.py test definitely_not_a_test --seed 1` | Rejected as zero tests run | `runs/preparation/empty-filter.log` |
| `python3 scripts/run.py new runs/experiment` | PREVIEW only: no workspace created and no agent called | `runs/preparation/preview.log` |

The negative checks are actual Verus runs, not fabricated proof output. Offline launcher tests invoke a Python stand-in explicitly labeled SIMULATED; they do not generate protocol code or claim Rust/Verus success. The role-observer smoke check relocates existing upstream code and supplies no voting, persistence, replication, KV or sharding solution.

| Implementation property | Status |
| --- | --- |
| Election Safety | OPEN: no generated cluster implementation, reachable-state proof or full observation connection. |
| Leader Append-Only | OPEN: no generated replication or faithful compaction proof. |
| Log Matching | OPEN: no generated log/term/prefix invariants or executable preservation proof. |
| Leader Completeness | OPEN: no generated election/commitment/quorum argument across arbitrary executions. |
| State Machine Safety | OPEN: no generated commitment/application/recovery composition proof. |

Snapshot representation definitions exist; their connection to actual create/install/persist/restore/apply paths is OPEN. `madraft/proof/implementation.rs` is intentionally absent and explicitly writable by the generator. All final callers verify the whole imported composition; the missing-file error is not a hard-coded permanent rejection. Conditional liveness and application-level KV/shard correctness remain separate OPEN goals. The five Raft targets do not establish those properties.

Review the DRAFT target adequacy, observer completeness and environment/TCB boundary. In particular, the pinned MadSim filesystem does not simulate unsynced-write loss (`power_fail` is TODO and `sync_all` is a no-op); real-disk durability is not established. State and snapshot files are not an atomic pair. The generator must choose and verify a coherent persistent representation and finite-counter failure behavior without weakening the supplied assumptions. The upstream ignored/commented linearizability placeholders are disclosed coverage gaps, not successes. No available build/proof tool was replaced or simulated; see `runs/preparation/tool-versions.log` for the actual versions.

Start the experiment explicitly from this preparation checkout:

```bash
python3 scripts/run.py new runs/experiment --execute
```

This uses the experiment's configured `gpt-6-astra` model with `high` reasoning unless selected explicitly with `--model`/`--effort`; the chosen overrides persist on resume. The initial cumulative wall-clock budget is 3600 seconds. Continue with `python3 scripts/run.py resume runs/experiment --execute`; increase the cumulative limit explicitly when necessary. After delivery, independently run `python3 scripts/check.py final --workspace runs/experiment --timeout 3600` and review the executable correspondence and TCB. Preparation stops here for human review.

# Joint MadRaft implementation and proof experiment — DRAFT

The experiment Codex completes the unfinished MadRaft project and constructs its Verus proofs. Maintenance changes the inputs, launcher and feedback tools, not the protocol solution. [RESULT.md](RESULT.md) records this maintenance revision and actual checks.

| Path | Purpose |
| --- | --- |
| `madraft/` | Initial student scaffold, original lab tests, attribution/license and a small supplied Verus smoke entry. New experiments copy this source. |
| `requirements/` | Lab scope, five fixed safety definitions, environment, test inventory, generic message/history support and independent proof callers. |
| `scripts/run.py` | Fresh workspace/session creation, cumulative budgets, same-session generation, independent feedback and bounded redesign. |
| `scripts/check.py` | Real Cargo/Verus checks, test coverage, source snapshots, grouped errors and final acceptance. |
| `scripts/prompts/` | Start, continue and redesign messages passed to the experiment session. |
| `scripts/tests/` | SIMULATED controller/checker regressions and real Rust/Verus toy checks; never experimental input. |
| `docs/` | Operator-only provenance. |
| `runs/<name>/` | One experiment's source copy, private Codex home, prompts, logs and reports. |
| `runs/.controller/<name>/` | Preparation-side inputs, cumulative state, lock and independent evidence outside that agent's writable workspace. |
| `target/` | Shared dependency cache, not a previous solution input. |

The agent receives AGENTS.md, fixed requirements and support, original MadRaft documentation/source, tool configuration, local check helpers and the current prompt. This README, operator reports, maintenance tests and old runs are not copied. Git tracks `runs/fresh-system-02`, including its candidate, round records and check logs; private Codex state and build caches are excluded. Other runs remain local and untracked.

```bash
# Offline controller regressions plus actual installed Rust/Verus toy checks.
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check.py foundation --timeout 60

# Preview only: no workspace, tools or model invocation.
python3 scripts/run.py new runs/fresh-system-02 --model gpt-6.1-sol --effort high

# Operator starts a new paid experiment explicitly.
python3 scripts/run.py new runs/fresh-system-02 --model gpt-6.1-sol --effort high --execute
python3 scripts/run.py resume runs/fresh-system-02 --execute

# Explicit cumulative extension; existing spending is retained.
python3 scripts/run.py resume runs/fresh-system-02 --total-seconds 7200 --max-rounds 12 --execute
```

A new run requires an unused direct child of `runs/`. Input version `madraft-joint-proof-3` adds generic message provenance and finite-history composition, changes the correspondence caller and updates scheduling. Resume continues only a run created with this version and unchanged inputs/tools; it cannot load an older answer or session. The private Codex home shares authentication and security rules, not personal instructions, config, sessions or memory. The locally probed CLI uses `--ignore-user-config` and `--no-daemon`. A whitelist and separate working directory are not an OS-level read sandbox: shared tools/dependencies and accessible sibling files remain part of the disclosed boundary.

After capability and scaffold checks, the agent generates one increment and returns by its soft deadline. The controller reaps its process group, copies the stopped candidate, runs independent checks and feeds evidence back into the same conversation. Raw logs, the source diff and candidate-reported progress remain available. Proof gaps are tracked separately from tests; repeating a checked dependency failure can trigger bounded redesign. Repeatedly unrun proofs are reported as a scheduling problem. No interactive approval checkpoint is inserted.

Defaults preserve `gpt-6-astra` / `high` and 3,600 cumulative seconds. Each agent call has a 600-second hard limit and a requested soft handoff 60 seconds earlier. A new round needs at least 120 seconds of useful work plus handoff grace, 300 seconds of feedback allowance and 600 seconds reserved for final checks. The launcher displays these settings; `--model` and `--effort` are explicit overrides. There are at most eight rounds, two repeated checked gaps before redesign, and one redesign. Actual generation, checks and probes all consume the cumulative budget. Resume cannot refund interrupted reservations; a process-group watcher handles controller death and a lock prevents concurrent writers.

| Command | Scope |
| --- | --- |
| `python3 scripts/check.py feedback --timeout 300` | Build/inventory, configured relevant test family at one seed, foundation/definitions, per-target dependencies, aggregate and local proof. Other functional coverage is explicitly unrun. |
| `python3 scripts/check.py test raft::tests::initial_election_2a --seed 1` | One selected functional check. |
| `python3 scripts/check.py proof madraft/proof/progress.rs` | Entire imported local proof unit. |
| `python3 scripts/check.py top execution_link` | One fixed caller and its full dependencies; candidate may supply `proof/execution_link.rs`. |
| `python3 scripts/check.py safety --timeout 300` | Formal support, five goals, correspondence, observations/snapshots, witness and combined caller. No Cargo tests. |
| `python3 scripts/check.py final --workspace runs/fresh-system-02 --timeout 3600` | All retained test families on seeds 1–5 and complete formal acceptance on the same stopped snapshot. |

Mixed checks reserve 40% for build/tests and at least 60% for proofs. Build calls have a 120-second cap; each test family and proof call has a 45-second cap, within the phase deadline. These are configurable checking limits, not performance targets. A hanging family cannot spend the formal reservation. An actual common dependency compile failure is reported once and dependent calls remain OPEN; no cached PASS is inferred. Final requires every active original test by name; the one original ignored placeholder is disclosed. Fewer seeds can be selected explicitly with `--seeds` for a standalone invocation, not silently inside controller acceptance.

Only the complete final check may return `MACHINE_CHECKS_PASSED`. Agent exit 0, partial proof counts and successful tests cannot do so. Other outcomes are `INCOMPLETE`, `BLOCKED` or `NEEDS_REVIEW`, with reasons in `runs/EXPERIMENT.md` and the preparation-side state/results. Changed standards and concurrent source edits prevent acceptance. The final caller consumes explicit initialization/step correspondence, preserves inputs/outputs/send provenance and composes the five model conclusions with the executable history.

Verus checks the supplied definitions and bodies; it cannot automatically decide whether candidate runtime relations, message extraction and observations cover every real path. Source scans are limited diagnostics. Human review still checks actual function contracts/callers, asynchronous coordination, crash and storage histories, snapshot decoding and the host TCB. Machine acceptance is not a claim of full Raft correctness. Conditional liveness remains OPEN. See `requirements/coverage.md` for candidate responsibilities.

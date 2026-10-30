# Joint MadRaft implementation and proof experiment — DRAFT

The experiment Codex completes the supplied student labs and generates the corresponding Verus proof development. Maintenance changes the system and generic examples, not a candidate's Raft answer. Current maintenance evidence and limitations are in [RESULT.md](RESULT.md).

| Path | Purpose |
| --- | --- |
| `madraft/` | Unfinished student scaffold, original tests, documentation, license and supplied role-observer smoke proof. |
| `requirements/` | Fixed lab scope, five Raft safety predicates, environment, generic message/history support, examples and independent callers. |
| `scripts/run.py` | Fresh experiments, same-session continuation, budgets, feedback and bounded redesign guidance. |
| `scripts/check.py` | Independent Cargo/Verus checks on stopped candidate snapshots, diagnostics and final aggregation. |
| `scripts/prompts/` | Common start/continue/redesign messages; the short `behavior.md` addition selects B guidance without duplicating the controller. |
| `scripts/tests/` | Offline SIMULATED system regressions and real Rust/Verus toy checks; never experiment input. |
| `docs/` | Operator-only provenance. |
| `runs/<name>/` | One experiment's source copy, local evidence and private Codex home. |
| `runs/.controller/<name>/` | Authoritative inputs, cumulative state and independent checks outside the experiment's writable workspace. |
| `target/` | Shared dependency/build cache, not a prior solution input. |

Git tracks `runs/calls-A-01` and `runs/calls-B-01`, including their recorded source and check evidence; private Codex state and build caches remain excluded. Other experiments remain local and untracked. Recorded candidates are history, not inputs to new experiments. The new input version is `madraft-joint-proof-4`, controller format `madraft-generate-check-4`. Resume requires this version's own origin and unchanged fixed inputs/controller/tools. The root AGENTS distinguishes system maintenance from the experimental generation role.

New input consists only of AGENTS.md, tool configuration, the generated ignore rules, four local check helpers (`check.py`, `inputs.py`, `process.py`, `tools.py`), initial MadRaft Cargo files/license/documentation/source/smoke proof, and the fixed `requirements/*.md` and `requirements/*.rs`. The controller keeps its own launcher and prompt templates outside the writable experiment and sends the selected prompt. Root README/RESULT, maintenance tests, old candidates, answer patches, sessions and logs are not copied. Private CODEX_HOME shares authentication and security rules only; personal instructions/config/sessions are not loaded. Accessible dependencies and sibling paths are not an OS-level read-isolation guarantee.

A and B use identical scaffold, support examples, fixed requirements, checker, tools, seeds and budgets. A receives general joint-generation guidance. B adds concrete real-behavior/caller guidance and explicit connection-gap feedback. All independent reports disclose gaps for both groups; there is no weaker A checker. `behavior.md` is the only new prompt file, separating the small experimental treatment from the common templates. No A/B outcome is established by maintenance tests.

```bash
cd /home/nitro/Desktop/raft_verus
# Previews: no workspace, tool or paid agent call.
python3 scripts/run.py new runs/calls-A-01 --guidance A --model gpt-6.1-sol --effort high
python3 scripts/run.py new runs/calls-B-01 --guidance B --model gpt-6.1-sol --effort high

# Explicit operator launches, each from a new workspace and session.
python3 scripts/run.py new runs/calls-A-01 --guidance A --model gpt-6.1-sol --effort high --execute
python3 scripts/run.py new runs/calls-B-01 --guidance B --model gpt-6.1-sol --effort high --execute

# Continue only the same v4 experiment; spending/history are retained.
python3 scripts/run.py resume runs/calls-B-01 --execute
python3 scripts/run.py resume runs/calls-B-01 --total-seconds 7200 --max-rounds 12 --execute
```

The controller sends an increment to one Codex conversation, waits for a normal handoff or enforces the hard limit, reaps its process group, snapshots stopped files and independently checks them. It returns actual errors, open model goals, connection units and functional gaps to that same conversation. It does not ask for per-round approval or launch another coding agent itself.

Reports separate (1) Verus formula results, (2) nominated actual-function/caller proof-unit checks and their unreviewed coverage claims, and (3) complete task acceptance. A candidate supplies one short connection table in PROGRESS.md, with actual function references, proof entries and open premises. The checker checks references/imports and runs entire nominated units under `--no-cheating`; a table, import or relation projection is not semantic evidence that every real caller is verified. Missing units remain OPEN even if model formulas pass. A snapshot formula over no snapshot behavior does not close the actual functionality. Final MACHINE_CHECKS_PASSED means machine checks passed, semantic review NOT_PERFORMED and implementation acceptance NOT_ESTABLISHED. Human review follows delivery, with no approval files/checkpoints inside generation.

| Command | Actual scope |
| --- | --- |
| `python3 -m unittest discover -s scripts/tests -v` | Offline controller/checker tests plus installed real Rust/Verus toy checks. Missing real tools are recorded BLOCKED, never replaced with simulation. |
| `python3 scripts/check.py foundation --timeout 60` | Real generic provenance, history, executable-call composition and recovery examples. |
| `python3 scripts/check.py top election_safety` | Fixed individual caller and all imported bodies. |
| `python3 scripts/check.py proof madraft/proof/progress.rs` | Candidate's entire local proof unit. |
| `python3 scripts/check.py safety --timeout 300` | Formal definitions, foundation, all goals, connection units and combined caller; no Cargo tests. |
| `python3 scripts/check.py feedback --timeout 300` | Bounded functional and formal feedback, not final acceptance. The controller supplies changed paths and family history. |
| `python3 scripts/check.py final --workspace runs/calls-B-01 --timeout 3600` | Every original test family at seeds 1–5, all formal/connection units and unified proof composition on one snapshot. |

Mixed checks reserve 40% for build/tests and at least 60% for formal work. Build is capped at 120 seconds; each test family/proof unit at 45 seconds. Feedback selects the configured number of oldest-attempted families first (`feedback_rotation = 1`), then relevant families inferred from changed directories. Raft changes also select host regressions; application changes select their family and Raft. Unknown impact broadens to all families. Tests that did not start stay oldest; attempted timeouts rotate so one hung family cannot starve the rest. Raw records retain selection, seed, timeout and unrun reasons. Standalone feedback has no controller history and starts the deterministic rotation at Raft. Final always selects all families and configured seeds; no old PASS is reused after edits.

Defaults: model `gpt-6-astra`, effort `high`, guidance B, total 3,600 seconds, hard slice 600, handoff grace 60, minimum useful work 120, feedback allowance 300, final reserve 600, at most eight rounds and one redesign instruction. A normal round reserves feedback and final; a closing round reserves final only. With 950 seconds left, it can grant 350 seconds then go directly to final. If actual final cost leaves enough useful budget, another increment can run and be rechecked. An unchanged candidate after final stops with an explicit reason rather than repeating identical checks. Maximum rounds, cumulative budget, repeated unrun proof checks and genuine environment failures are explicit stopping conditions; a stubborn theorem alone does not exhaust the whole experiment when redesign guidance has been used.

Time accounting includes workspace preparation, agent processes, checks, capability probes and controller overhead. Idle time while the operator is not running the controller is excluded. Reservations are recorded separately from measured cost; interrupted reservations remain charged with actual duration unknown. Reports record source input fingerprint, input/controller/prompt versions, model/effort, raw returned token counters, interventions, stop reason and unused budget. Counter scope is UNKNOWN unless explicitly established; cumulative/per-call counters are not summed and dollar cost is unknown. One fresh generation experiment, its continuation rounds and its test seeds are different units.

The requested content fingerprint identifies copied, possibly uncommitted inputs where Git metadata/version labels alone cannot identify exact bytes; it does not authorize results or replace ordinary tests. Checks retain exact source/requirements snapshots, command/environment/seed and raw output; controller records actual tool versions. Historical positive results are always rerun. Only a shared compilation failure inside one immutable check invocation can refer to an earlier raw failure. Cross-candidate result records, changed standards or concurrent edits prevent machine acceptance.

Remaining semantic boundary: Verus proves submitted statements, not automatic adequacy of candidate runtime relations. Review actual callers, all state-change sources, async release ordering, durable recovery, snapshot decoding, observation coverage and primitive host contracts. Protocol decisions cannot be trusted merely because they live in a host wrapper. Conditional liveness and broader application correctness remain OPEN. These maintenance checks establish system behavior and toy examples, not generated MadRaft correctness or an A/B improvement claim.

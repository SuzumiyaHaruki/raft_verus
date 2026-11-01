# Joint MadRaft implementation and proof experiment — DRAFT

The experimental Codex starts from the unfinished MadRaft labs and generates executable implementation and Verus proofs together. Maintenance changes the controller, inputs and generic examples; it does not supply a Raft solution. Current maintenance results and limitations are in [the election-start maintenance report](review/ELECTION_START_MAINTENANCE.md); [RESULT.md](RESULT.md) retains earlier controller results.

| Path | Purpose |
| --- | --- |
| `madraft/` | Initial student scaffold, original tests, license and supplied role-observer smoke proof. |
| `requirements/` | Fixed functional scope, five safety predicates, environment, execution/provenance support and independent final callers. |
| `scripts/run.py` | One session's bounded generation, independent feedback, same-run checkpoints, scoped rollback and redesign. |
| `scripts/check.py` | Real Cargo/Verus checks of copied candidates; current-obligation, connection and final diagnostics. |
| `scripts/prompts/` | Short start/continue/redesign instructions and concrete actual-behavior guidance. |
| `scripts/tests/` | SIMULATED controller regressions and separately reported real Rust/Verus toy checks. Never experiment input. |
| `tasks/request_vote/` | Optional receiver assignment: English instructions, fixed specifications/callers/tests, unfinished candidate preparation and maintenance-only bad fixtures. |
| `tasks/election_start/` | Optional receiver plus campaign assignment: English inputs, one authoritative core, fixed mixed-history/recovery callers and real RPC tests; actual algorithm and proofs remain unfinished. |
| `review/request_vote/` | Actual receiver-task maintenance results and reproduction evidence; never experiment input. |
| `review/election_start/` | Readable maintenance logs, source inputs and negative-case patches; never experiment input. |
| `docs/` | Operator provenance. |
| `runs/<name>/` | One experiment's own source, progress and evidence; its private Codex home is excluded from Git. |
| `runs/.controller/<name>/` | This experiment's authority inputs, state, checked snapshots, attempt journal and failed rollback versions. |
| `target/` | Local dependency/build cache and temporary maintenance builds; ignored for new files, not proof evidence. |

The run selected for Git tracking is `runs/election-start-01`; other experiments remain local. Recorded candidates, prompts, sessions and patches are never fresh-run inputs. The full input/controller versions remain `madraft-joint-proof-5` and `madraft-generate-check-5`; scoped input versions are `madraft-request-vote-1` and `madraft-election-start-1`. Existing experiments remain unchanged and are not warm-start inputs to a new experiment.

A fresh workspace receives only the allowlisted initial MadRaft source, AGENTS, tool configuration, four local check helpers and fixed requirements. It does not receive root README/RESULT, maintenance tests, old results or controller prompt templates. The private CODEX_HOME shares existing authentication/security rules, without loading personal configuration/instructions/sessions. This is input isolation, not a claim of OS-level read isolation from every accessible sibling path.

Run one new exploration with the existing model/tool configuration:

```bash
cd /home/nitro/Desktop/raft_verus
# Preview only: no workspace, tool invocation or model call.
python3 scripts/run.py new runs/full-next-01 --total-seconds 7200
# The operator separately starts this one fresh experiment.
python3 scripts/run.py new runs/full-next-01 --total-seconds 7200 --execute
```

`tools.ini` retains model `gpt-6-astra`, reasoning `high` and existing local tools. The default total is 7,200 seconds, with 600-second work slices, 300-second feedback caps, 600-second final reserve, 120-second minimum work, 60-second soft-handoff grace and 64 maximum rounds. The round limit is a guard, not a target. Agent work (including its tools), independent microcheck replays, feedback/final checks and controller overhead all consume the same wall-clock budget. Resume uses the same run path, for example `python3 scripts/run.py resume runs/full-next-01 --execute`; it retains spending, failures, checkpoints and session identity. No time is restored by source rollback. The existing optional A/B flag remains compatible, but no comparison campaign is required; default guidance B uses the improved workflow.

An optional `--task request-vote` selects a bounded generation assignment, described in [TASK.md](tasks/request_vote/TASK.md). Preparation copies the untouched root scaffold and applies only declarations, unfinished Core/proof entries and thin wiring to the original MadRaft RequestVote handler. The local agent receives English task instructions; it must generate the voting algorithm and proofs. Default `full` mode and its acceptance goals remain unchanged. Task scope and input version are recorded at creation; resume cannot switch between full and local acceptance. The task uses the configured model/effort and the same 7,200-second cumulative budget. No previous run or maintenance fixture is input.

The earlier receiver-task maintenance delivery, actual checks and remaining boundaries are recorded in [the maintenance report](review/REQUEST_VOTE_MAINTENANCE.md). The existing `runs/request-vote-01` is preserved; use a new directory for another experiment.

```bash
# Preview; no tools, candidate or agent are started.
python3 scripts/run.py new runs/request-vote-next-01 --task request-vote
# After reviewing the maintenance delivery, the operator can start one new run.
python3 scripts/run.py new runs/request-vote-next-01 --task request-vote --execute
# Continue only this same run and conversation.
python3 scripts/run.py resume runs/request-vote-next-01 --execute
# Maintenance only: real tools and deliberately bad fixtures, never an agent.
python3 tasks/request_vote/maintain.py
```

The receiver checker separately reports rules, finite output history, confirmed-save recovery, written-but-uncompleted recovery and actual MadRaft RPC/file tests. Its final runs all task tests on all configured seeds and every local proof entry on one candidate. `REQUEST_VOTE_CHECKS_PASSED` is a local result; full MadRaft stays INCOMPLETE, all five global safety properties and conditional liveness stay OPEN. This task retains explicitly unverified serialization, I/O and RPC host assumptions; it does not claim arbitrary crash recovery.

The new `--task election-start` assignment regenerates the receiver and adds active campaign start/self-vote in one shared Core; see [its task](tasks/election_start/TASK.md). Only matching successful save completion may authorize real RequestVote RPCs. Independent observations combine actual self-choices with actual affirmative replies across mixed finite calls and the two bounded recovery windows. No vote counting or leader promotion is supplied. Eight formal groups and sixteen required tests on seeds 1–5 must pass on one candidate for `ELECTION_START_CHECKS_PASSED`; that label still leaves all global properties OPEN and semantic review outstanding. Formal acceptance verifies candidate bodies and lemmas, without `--verify-root` or `--no-verify`. All supplied prompts and generated feedback are English.

```bash
# Preview this one new experiment; no agent is invoked.
python3 scripts/run.py new runs/election-start-01 --task election-start
# Start only after reviewing the maintenance delivery.
python3 scripts/run.py new runs/election-start-01 --task election-start --execute
# Reproduce maintenance with real tools and explicitly wrong fixtures, no agent.
python3 tasks/election_start/maintain.py
```

The following per-obligation/block and full-acceptance discussion applies to default full mode.

Each work slice selects one obligation in a small flat INI block inside `PROGRESS.md`; see [the format and coverage rules](requirements/coverage.md). It names actual functions, a conclusion, its upper purpose, open premises, ordered proof dependencies, an entry/parent unit and Rust test filters. The controller constructs commands from those fields and never executes arbitrary candidate shell text. A single CLI slice can contain several small `check.py obligation` calls. Local attempt snapshots are replayed independently (up to three per handoff, budget permitting); unreplayed candidate results remain untrusted. The stopped current candidate is always independently checked as well.

The next prompt prioritizes the selected obligation's first unfinished dependency, relevant real errors and newly passing related units. Two substantive handoffs with buildable protocol changes/local results and no checked initialization/step nomination prioritize establishing that execution connection. This is a scheduling heuristic, not automatic semantic coverage. Missing entries lead to interface work; compilation failures, proof failures, timeouts and unavailable tools remain distinct. Unrelated helper counts cannot close a dependency.

A nominated local proof unit runs even if its outer async file is not imported. Reports retain its actual PASS/FAIL, imported-function hints and separate outer-path gaps. An imported parent unit may be checked while its call semantics and premises still need review. The checker does not infer caller correctness from names, imports, a coverage table, a test-only wrapper or equality of two model relations. Protocol judgments in async wrappers remain proof obligations, not trusted host behavior.

Independent attempts append to controller `attempts.jsonl`, mirrored at `runs/<name>/runs/attempts.jsonl`, with source snapshots, diffs, real results and first obstacles. Useful snapshots become checkpoints labeled BUILDABLE, LOCAL_UNIT_CHECKED or PARENT_UNIT_CHECKED_PREMISES_UNREVIEWED, with separate build status. When abandoning a path, the agent may request a listed `cp-N` in the current block. The controller restores only the named source/proof scope plus transitive proof dependencies, preserves the failed version/diff/logs, retains unrelated files and rechecks affected work. Protected tests/requirements, another experiment and outside-workspace paths cannot be restored. Files restored successfully are not treated as reverified.

Three actual failed proof attempts without new related checked units request a redesign of that obligation. Defaults allow two redesigns per obligation and six in the run. Renaming an id or overlapping actual function/entry keeps the history; arbitrary semantic identity still requires review. At the limit the agent must select another necessary dependency or stop honestly. Missing files and unavailable/timeout checks do not spend mathematical redesign allowance. Normal soft handoff and the hard process watchdog remain; an unchanged candidate after final checks stops rather than repeatedly running the same final.

| Command | Scope |
| --- | --- |
| `python3 scripts/check.py obligation --timeout 120` | Selected dependency/entry/parent units and selected tests on one snapshot; partial outcomes stay partial. |
| `python3 scripts/check.py feedback --timeout 300` | Current increment, rotating functional families and formal feedback. |
| `python3 scripts/check.py proof madraft/proof/progress.rs` | Entire named local proof unit, including all imported bodies. |
| `python3 scripts/check.py top election_safety` | Fixed independent caller with its actual imported dependencies. |
| `python3 scripts/check.py safety --timeout 300` | Formal checks, including current/local/connection units; no Cargo tests. |
| `python3 scripts/check.py final --timeout 600` | Every original functional family at every configured seed, all fixed formal goals and the aggregate, on one candidate. |
| `python3 -m unittest discover -s scripts/tests -p 'test_[cho]*.py' -v` | SIMULATED controller/checker regressions. |
| `python3 -m unittest discover -s scripts/tests -p test_verification.py -v` | Actual installed Rust/Verus positive and negative toy checks. Missing tools are BLOCKED, not simulated success. |

Tests retain a separate functional phase (40% of mixed-check time) and cannot consume the reserved formal phase. The selected proof dependencies run first in the formal phase. Functional families rotate by last attempt and changed-source relevance; final always uses the full family/seed set. Default seeds are 1–5. No earlier PASS is reused across snapshots; an identical proof unit within one invocation may share that invocation's actual result/log. Input fingerprints retain the existing evidence binding and are not approval or semantic proof.

Complete acceptance still requires all five Raft properties, executable correspondence, nonempty execution, snapshot/observation obligations and every retained functional check. Local/model proof success and agent exit 0 cannot finish the experiment. Even MACHINE_CHECKS_PASSED leaves semantic review NOT_PERFORMED and implementation acceptance NOT_ESTABLISHED: inspect actual callers, model coverage, ghost/durable history, observations and host trust boundaries. Conditional liveness and application correctness beyond the five properties remain OPEN. Human review follows delivery; generation has no interactive approval checkpoint.

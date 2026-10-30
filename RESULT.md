# P0/P1/P2 experiment-system maintenance — DRAFT

Delivered `madraft-joint-proof-5` / `madraft-generate-check-5` on 2026-10-30. This changes the experiment system and generic examples, not a MadRaft answer. No paid experiment, other agent, merge, push, publication or global tool/configuration change was performed.

Root Git metadata is not exposed: `.git` is empty/read-only, so root HEAD/dirty status cannot be queried. The available publication checkout `/tmp/raft-verus-publish-ab` is at `a402bd5e2ce26350f1b05feb98f1a97c437f2cba`; all its tracked working-file bytes matched the root at the start. No reset occurred. Final comparisons preserve all 31 scaffold files, all 11,042 tracked files in the two old A/B candidates, fixed safety/environment/scope and independent final callers. [Preservation evidence](runs/maintenance-p012/preservation.json).

| Requirement | Implemented mechanism |
| --- | --- |
| P0 | `check.current_obligation()` reads one flat block in PROGRESS.md; `Checks.obligation()` checks ordered prerequisites, the nominated entry and parent. `run.next_obligation()` prioritizes the first blocker. `Loop.inspect()` prioritizes execution connection after two substantive handoffs without checked initialization/step evidence. Missing interfaces, compilation, proof failure and checking failures have distinct feedback. |
| P1 | `Checks.unit()` runs the entire Verus unit even when an outer function is not imported. `connections()` retains its result/log and separately lists outer gaps. Prompts require actual protocol judgments, caller premises, authorized inputs, outputs and recovery to be checked. Names/imports remain hints, not execution proofs. |
| P2: attempts/redesign | `record_attempt()` retains versions, aliases, failures and newly checked units. Historical PASS or a byte-identical renamed unit is not new progress. `replay_attempts()` independently rechecks up to three saved local attempts per handoff; candidate JSON outcomes are not trusted. Three proof failures without related progress request redesign, bounded by two per obligation and six per run. Missing files, compilation, timeouts and tool errors do not count as mathematical failure. |
| P2: rollback | `remember()` reuses independent snapshots with build/local/parent labels and open premises. `rollback()` preserves failed code/proof/diff/notes, restores this run's named files and transitive proof dependencies, retains unrelated files and invalidates current results before writing. It then rechecks. Spending, session, failure journal and redesign counts never roll back. Cross-run/outside/protected paths are rejected. |
| 2H | Defaults are 7,200 cumulative seconds and 64 rounds. Existing 600/300/600-second work/feedback/final limits, 120-second minimum work, 60-second handoff, functional/proof separation and closing-round reservation remain. Agent tools, independent replays/checks and controller overhead share the budget. Model `gpt-6-astra` / `high` and tool paths remain unchanged. |

Removed the old global missing-goal stagnation counter and revised start/continue/redesign prompts, AGENTS, coverage instructions and README. Added only a generic request-authorized counter example; existing provenance, induction, actual-function and recovery examples remain. No Raft solution or prescribed architecture was supplied. The accessible IDS [DSA](https://github.com/skydiscover-ai/skydiscover/blob/0d932b690670a7e544388ad362e9876c8bd256a0/skydiscover/synthesize/workflow/agents/2-synthesis-loop/dsa.md) and [ISA](https://github.com/skydiscover-ai/skydiscover/blob/0d932b690670a7e544388ad362e9876c8bd256a0/skydiscover/synthesize/workflow/agents/2-synthesis-loop/isa.md) instructions informed concrete attempts, applicable rollback and failure-driven redesign; their role framework was not imported.

| Executed maintenance check | Actual result |
| --- | --- |
| Offline controller/checker regressions | **76 tests passed**, with explicitly SIMULATED processes/outcomes. Includes resume, both redesign caps, rename handling, scoped rollback, independent replay, proof budget isolation, oldest-family rotation despite selected tests and continuation after round eight. [Log](runs/maintenance-p012/offline-system-tests.log). |
| Real Rust/Verus test methods | **13 passed**, separately from the fake suite. Rust/Cargo 1.92.0 and Verus 0.2026.01.23.1650a05. [Test log](runs/maintenance-p012/real-verus-tests.log), [raw sources, commands and diagnostics](runs/maintenance-p012/real-tools-1793343669370000244). |
| Generic foundation | **15 verified, 0 errors**, also checked through the actual CLI. [Verus log](runs/maintenance-p012/real-tools-1793343669370000244/foundation.log), [CLI output](runs/maintenance-p012/foundation-cli.log). |
| Function/model mismatch | Correct +1: **15 verified, 0 errors**. Actual +2: **14 verified, 1 error**, postcondition failure; unchanged model-only unit still verifies 10 obligations. [Positive](runs/maintenance-p012/real-tools-1793343669370000244/actual-handler-plus-one-accepted.log), [negative](runs/maintenance-p012/real-tools-1793343669370000244/actual-handler-plus-two-rejected.log). |
| Recovery and request authorization | Correct cases each **15 verified, 0 errors**. Missing recovery premise and unauthorized requestless mutation each **14 verified, 1 error**, respectively caller precondition and postcondition failures. [Recovery negative](runs/maintenance-p012/real-tools-1793343669370000244/recovery-caller-rejected.log), [request negative](runs/maintenance-p012/real-tools-1793343669370000244/requestless-local-mutation-rejected.log). |
| Layered checker with real Verus | Correct local unit PASS while outer async path/parent stay OPEN; defective executable unit yields `verification_failure`. Both overall results are INCOMPLETE. [Positive](runs/maintenance-p012/real-tools-1793343669370000244/selected-good/results.json), [negative](runs/maintenance-p012/real-tools-1793343669370000244/selected-defect/results.json). |
| Retained negative controls | Same-source Rust defect/repair, fabricated delivery, false election/observation, imported failing bodies and external bodies behave as required. Expected negative outcomes are not successful proofs. |
| Fresh preview/argument parsing | Preview succeeded with 7,200 seconds, 64 rounds and 48 allowlisted inputs; no workspace/session created. `--execute --help` parsed the execution command without calling an agent. [Preview](runs/maintenance-p012/preview.log), [inputs](runs/maintenance-p012/new-inputs.json). |

An initial real-test fixture failed because its temporary configuration used the wrong dependency-cache path; the fixture was corrected and the real suite rerun. Earlier raw evidence remains locally available. No unavailable real check was replaced by a fake verifier. The passing evidence is the directory linked above.

```bash
cd /home/nitro/Desktop/raft_verus
python3 -m unittest discover -s scripts/tests -p 'test_[cho]*.py' -v
python3 -m unittest discover -s scripts/tests -p test_verification.py -v
python3 scripts/check.py foundation --timeout 60

# Preview only; executed during maintenance.
python3 scripts/run.py new runs/joint-proof-2h-01 --total-seconds 7200
# Operator starts this one fresh experiment; NOT executed during maintenance.
python3 scripts/run.py new runs/joint-proof-2h-01 --total-seconds 7200 --execute
```

Semantic limits: dependencies and connection claims are agent proposals. Python does not prove that a nominated function is the real caller, that its parent establishes premises, or that relations cover all async/recovery/snapshot paths. Literal module inspection cannot infer every affected caller; additional rollback paths must be named and rechecked. Progress detection is conservative at proof-unit granularity: it recognizes simple renames, not arbitrary semantic equivalence or every stronger theorem in a previously passing file. These remain review responsibilities.

Full acceptance retains every functional family on seeds 1–5, all five safety targets, execution/observation/snapshot/nonempty obligations and the combined final unit on one source snapshot. Even MACHINE_CHECKS_PASSED leaves semantic review outstanding. No complete MadRaft experiment or full Raft proof was run; two-hour scheduling was tested with fake processes/clocks. All five implementation-level safety goals and conditional liveness remain OPEN. The next experiment must generate its own solution from the untouched scaffold. Delivery stops for review without adding an interactive approval checkpoint.

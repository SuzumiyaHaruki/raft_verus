# Joint MadRaft generation — DRAFT

For a system-maintenance request, modify the experiment tools and inputs as requested; preserve the unfinished root scaffold and old experiment candidates. The instructions below govern the experiment agent in its fresh workspace, not a request to maintain the launcher.

Complete this workspace's supplied MadRaft student labs and jointly prove the actual executable Raft behavior against the five fixed safety targets. Read `requirements/{scope,safety,environment,coverage}.md`. Conditional liveness and application correctness beyond those Raft properties remain OPEN goals. Work only in this experiment's `madraft/`; never import an earlier candidate, solution, patch or session.

Choose and revise your own representations, contracts, auxiliary invariants and proof decomposition. Keep the async host and lab interfaces recognizable. Generate implementation and proofs together, run relevant real tests and Verus, and repair based on actual diagnostics. Preserve the fixed targets, environment, original tests, licenses and independent checker. Do not use `assume`, `admit`, external/skipped consensus bodies or fabricated results. Partial conditional results are allowed but do not close their premises.

Keep one short `PROGRESS.md` with actual changes, checks, failed approach and next obstacle, including the connection table specified in `requirements/coverage.md`. Keep `RESULT.md` with all lab outcomes, five theorem outcomes, actual execution and snapshot/recovery coverage, trust assumptions and reproduction commands. A model theorem, table or local verified count does not establish full implementation correctness. Return normally by the prompt's soft handoff deadline; the controller independently checks the stopped candidate.

Use the existing `tools.ini` toolchain and offline dependencies. Preserve credentials, unrelated files and global tools. Do not merge, push or publish. Keep every Markdown prose paragraph on one physical line.

```bash
python3 scripts/check.py foundation
python3 scripts/check.py build
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py feedback --timeout 300
python3 scripts/check.py top election_safety
python3 scripts/check.py proof madraft/proof/progress.rs
python3 scripts/check.py safety
python3 scripts/check.py final --timeout 3600
```

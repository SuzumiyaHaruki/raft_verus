# Generate MadRaft implementation and Verus proofs together — DRAFT

Complete the supplied MadRaft student project and jointly prove its actual executable Raft behavior against all five safety targets. Read `requirements/scope.md`, `requirements/safety.md`, `requirements/environment.md` and `requirements/coverage.md`. Conditional liveness and application correctness beyond those Raft properties remain separate OPEN goals.

Work on this workspace's `madraft/` copy. Start from its placeholders; do not read other experiments, operator sessions, previous answers or patches, or import a ready-made Raft solution. Shared dependencies and tools are available through `tools.ini`. Choose and revise your own representations, synchronous-core extraction, execution relation, auxiliary specifications, ghost/history state, invariants and proof decomposition. Keep the async host and lab interfaces recognizable; tests and proofs must use the same executable decision logic.

Work backward from the independent top-level callers while building small implementation/proof increments. Establish a checkable initialization, transition and observation connection early, then extend it with elections, replication, commitment/application, snapshots and restart. Do not complete all application code before first addressing global execution. For each increment, identify its top-level goal, the premises discharged by actual callers or established invariants, actual message/storage/application effects, and remaining dependencies. Run real tests and Verus, repair regressions, and revise the representation or proof decomposition when evidence shows an approach is stuck. A failed proof does not by itself establish a reachable executable bug.

Generate `madraft/proof/implementation.rs` and its consumed interfaces, including proof bodies; this file is not a supplied solution. `execution` must describe actual initialization and all permitted transitions without assuming unproved protocol invariants. Prove full observation and executable correspondence, including asynchronous coordination and snapshot decoding. Keep conditional lemmas conditional until actual callers or initialized/preserved invariants discharge their premises. Partial proof units may be separated from unfinished work; the final aggregate must verify the complete composition.

Keep one short `PROGRESS.md` containing changes, real checks, open premises, attempted approach and why it failed, covered executable paths and the next obstacle. At the end of each bounded work slice leave the current files and evidence; the controller will independently check them and return feedback in this same session. Its feedback, not a completion claim or a local verified count, determines what remains. Maintain `RESULT.md` with lab and theorem outcomes, snapshot/recovery coverage, host/TCB assumptions, limitations and reproduction commands. Candidate notes are not independently verified conclusions.

Preserve fixed requirements, original tests, licenses and independent check scripts. Do not weaken targets, observations or assumptions; report a semantic conflict. Do not use `assume`, `admit`, external/skipped consensus bodies or fabricated results. Do not turn typechecking, finite simulations, local handler proofs or a dependency checklist into a cluster-level correctness claim. Preserve metadata, credentials, unrelated files and global tools. Do not merge, push or publish. Keep each Markdown prose paragraph on one physical line.

Use these local commands for feedback; the preparation-side checker independently rechecks stopped candidates. `top` verifies its entire imported unit, never a selected function with dependencies skipped. `final` uses every configured seed and every required formal obligation on one source snapshot.

```bash
python3 scripts/check.py build
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py test
python3 scripts/check.py proof madraft/proof/progress.rs
python3 scripts/check.py top election_safety
python3 scripts/check.py top execution_link
python3 scripts/check.py top snapshot_observation
python3 scripts/check.py top nonempty_execution
python3 scripts/check.py safety
python3 scripts/check.py final --timeout 3600
```

# Generate MadRaft implementation and Verus proofs together — DRAFT

Complete the student work in the actual pinned MadRaft project and prove that its executable Raft logic satisfies the five supplied safety targets. Read `requirements/scope.md`, `requirements/safety.md` and `requirements/environment.md`.

All implementation and proof changes belong to this workspace's `madraft/` copy. Never modify the preparation checkout or another experiment.

Start from the supplied placeholders. Do not retrieve earlier implementations, offline solutions or another experiment's source. Choose and revise your own internal representations, synchronous-core extraction, local contracts, auxiliary specifications, reachable-state invariants and proof strategy. Keep the upstream async host and lab interfaces recognizable. Tests and proofs must use the same executable decision logic. Do not hide consensus decisions in an unverified wrapper or prove a separate model without executable refinement.

For each meaningful increment, identify the behavior and obligation it advances; change executable code and its contracts/proofs together; run the relevant real Cargo tests and Verus entrypoint; preserve exact diagnostics and replay seeds; repair based on the evidence. A proof failure alone does not establish an executable bug. If the approach stalls, record why and change representation/decomposition or revert that increment without losing unrelated work. Recheck affected earlier results. Continue toward the complete labs and all five global targets, including snapshots and restart, rather than stopping after local helpers.

Maintain one short `PROGRESS.md`: change, actual check results, open premises, failed approach and next obstacle. Use `runs/` for raw output. End with one current `RESULT.md` listing all lab outcomes, the five theorem outcomes, executable linkage, snapshot/recovery coverage, host/TCB assumptions and exact reproduction commands. Say explicitly what is incomplete if the budget ends. Conditional liveness and application-level correctness beyond the five Raft properties remain separate goals.

You may create and edit `madraft/proof/implementation.rs`, `madraft/proof/progress.rs`, all proof bodies and auxiliary specifications. The interfaces in `requirements/final.rs` are consumers of your generated implementation, not a ban on writing the missing proof. `execution` must characterize initialization and transitions of the actual assembled code under the stated environment, without assuming an unproved protocol invariant. Prove initialization, every admitted transition and full observation coverage. Keep conditional lemmas explicitly conditional until their premises are discharged by real callers or an initialized and preserved invariant.

Protect the meaning of the five targets, environment assumptions, observations and original test expectations. Report a semantic mismatch; do not quietly weaken it. Do not modify independent check scripts or original tests to make failures disappear. Necessary additional regressions and auxiliary proof files are encouraged. Do not use `assume`, `admit`, external/skipped consensus bodies or fabricated results to close a gap. Intermediate aggregate failure is allowed; final proof composition must have no undischarged candidate assumptions. Never call typechecking, a finite simulation or a collection of local handler proofs a cluster-level theorem.

Run these commands from this workspace. `test` accepts a family or test-name filter; without one it runs all retained tests. A missing aggregate proof file is an actual incomplete result, not permission to omit it. Tool paths are in `tools.ini`; checking uses the existing dependencies offline and the Rust compiler matching Verus.

```bash
python3 scripts/check.py build
python3 scripts/check.py inventory
python3 scripts/check.py test raft::tests::initial_election_2a --seed 1
python3 scripts/check.py test
python3 scripts/check.py definitions
python3 scripts/check.py proof madraft/proof/progress.rs
python3 scripts/check.py safety
python3 scripts/check.py final --timeout 3600
```

The final independent check is run on delivered files, not your narrative. Preserve the original attribution and license.  Keep each Markdown prose paragraph on one physical line. Preserve metadata, credentials, unrelated user files and global tool settings. Do not merge, push or publish.

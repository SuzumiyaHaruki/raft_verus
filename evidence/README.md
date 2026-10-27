# Reading execution evidence

Start with [the review packet](../review/REVIEW_PACKET.md). Evidence contains actual command output and generated controller records, not another requirements set to edit or send wholesale to an LLM.

- `joint-design/`: current joint implementation/specification revision, controller tests, seed checks and SIMULATED redesign/repair with real Rust/Verus checks.
- `from-zero/`, `next-revision/`, `core-objective/` and earlier named directories: historical deliveries and checking results for their recorded source versions.
- `runs/<run-id>/`: controller candidates, attempts and independent checks from explicit runs.
- Root `model-*` files: earlier tool observations, model checks, verifier calibration and licenses; some remain referenced by configuration.

Read a run's `conclusion.json` or `plan-state.json` first. `*.check.json` records the independent outcome; stdout/stderr logs retain actual output; `*.process.json` records the command and process result. Attempts retain prompts, agent claims, patches and repair feedback. Existing bindings associate checks with their inputs. Candidate/build directories are generated artifacts.

Old results do not establish correctness of changed source or checking policy. SIMULATED generation is distinct from REAL_LOCAL checking; a local pass is not whole-Raft acceptance. Historical evidence is preserved. Temporary debugging records may describe failed approaches; the review packet identifies final results.

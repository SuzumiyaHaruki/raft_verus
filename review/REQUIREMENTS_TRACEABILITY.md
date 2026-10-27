# Current requirement mapping — DRAFT

The current experiment generates a Rust Raft implementation and its implementation-layer Verus specifications/proofs together from the disclosed scaffold. Historical bootstrap and maintenance deliveries remain in evidence; they do not describe the current source as a working Raft. Actual results and next commands are in [REVIEW_PACKET.md](REVIEW_PACKET.md).

| User requirement | Maintained implementation and check |
|---|---|
| One coupled Raft, developed incrementally | [Module roadmap](../input/roadmap.md), module assignments and `workflow.toml`; one persistent candidate, selected-step completion only |
| Generate behavior, local contracts, representation and proof together | [Starting point](../input/starting-point.md), editable `node_state.rs`, `contracts.rs`, operation and auxiliary proof files; no offline answers in real candidates |
| Connect proofs to executable behavior and independent requirements | [Proof entrypoints](../crates/raft-proofs/README.md), protected observation/behavior consumers and same-source Cargo/Verus; test/proof pairs reject controlled defects |
| Repair, redesign and continue with history/budgets | `orchestrator/scheduler.py`, `continuation.py`, `feedback.py`; cumulative accounting, saved attempts, bounded redesign and regression repair |
| Functional, scenario, formal and performance feedback | Independent `tests/`, replay examples, local proof profiles and repeated measurements; no invented performance target |
| Separate task completion, typechecking and whole-project acceptance | `verifier_runner.py`, scheduler conclusions and [proof checklist](../input/proofs.md); typechecking closes no theorem, local passes close no global Raft obligation |
| Independent checking and truthful provenance | Protected checker/oracles, scoped edits, actual subprocess records and `SIMULATED` versus `REAL_LOCAL`; source errors are repairable with proof NOT_RUN |
| Review after delivery | [Pending decisions](OPEN_DECISIONS.md) and review packet; explicit launch/resume, no approval file or interactive checkpoint inside the loop |
| Readable English task inputs | `input/` contains one topic per Markdown file; selected module/step prompts remain English, with one physical line per prose paragraph |

[Properties](../input/properties.md) names the safety/liveness targets; [assumptions](../input/assumptions.md) and [trust](../input/trust.md) record premises and TCB. [proofs.md](../input/proofs.md) is the single maintained list of all 26 OPEN global obligations, their dependencies and coverage. Do not duplicate their status in another registry.

Controller tests are in `orchestrator/tests`. In particular, `test_joint_design.py` checks alternate layouts/invariants, weak contracts, false invariants, disconnected observation contracts and Verus source errors using real installed tools. `test_starting_point.py` checks that unfinished future operations do not block initialization and that offline solutions remain excluded. Existing process, policy, adapter, continuation, measurement and module tests retain their coverage.

Earlier Desktop requirement documents and their historical outcomes are recorded in prior review packets under `evidence/`. The current source still lacks complete elections, replication, commitment, recovery, global refinement and liveness; no new semantic choice is approved by this mapping.

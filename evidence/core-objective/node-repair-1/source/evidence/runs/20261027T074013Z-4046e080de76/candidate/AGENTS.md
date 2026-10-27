# Rust Raft development — DRAFT

Maintenance agents: read `Agent.md` when present. Experiment candidates omit that
file and follow their selected English assignment in `input/`.

Build an independent, fixed-membership Rust Raft library with Verus safety and
conditional-liveness proofs. Read [README.md](README.md), the selected
module assignment linked from [input/roadmap.md](input/roadmap.md) and
[review/REVIEW_PACKET.md](review/REVIEW_PACKET.md).
The next generation task builds shared node initialization; a complete Raft node
is still unfinished.

- `crates/raft-verified` holds executable code, tests, replay and measurements;
  `crates/raft-spec` defines the candidate model; `crates/raft-proofs` and local
  proof blocks establish correspondence. Test and prove the same executable logic.
- `input/` explains requirements and complete modules with bounded steps;
  `workflow.toml` selects a module document, named step, scoped
  inputs, edits and independent checks. Reuse `orchestrator/`, standard libraries
  and existing interfaces. Do not build a generic orchestration framework.
- Start with `python3 -m orchestrator validate` and the selected task's commands.
  Run relevant tests and proofs after code changes. A typecheck is not a proof;
  step completion is not module completion or whole-project acceptance. Record missing checks honestly.
- All material supplied to an LLM must be English. Preserve original diagnostic
  evidence unchanged; label English summaries and link to the raw output.
- Preserve user work and the read-only `.git`, `.agents`, `.codex` and `.aws`.
  Do not reset or clean the checkout. Do not add hashes, frozen contracts, baselines
  or gates without a concrete failure scenario and why ordinary mechanisms do not
  suffice. Preserve existing safeguards; binding rationale is in
  [docs/workflow.md](docs/workflow.md).
- Specifications, contracts and new tools remain DRAFT pending review. Proved
  helper lemmas and auxiliary invariants are ordinary development. Public behavior,
  top-level targets, assumptions, TCB and acceptance-policy changes require review;
  never weaken a target to make a failing check pass. Do not use `assume`, `admit`,
  skipped verification or fabricated evidence to cover gaps.
- Ordinary repairs cannot edit their independent oracles or checker policy.
  Fixtures simulate generation; only actual Rust/Verus runs supply real checking
  evidence. AGENTS.md and candidate copies are not OS permission isolation.
- Human review is after code delivery, for discussion and revision with the user.
  Local start/resume needs no approval file or human checkpoint. Explicit execution
  settings, sandboxing, scoped edits and independent checks still apply; source
  specifications remain DRAFT and a successful run is not permission to merge.
- This revision permits the requested incremental slice and controller work.
  Configure explicitly launched experiments without running real agents during
  this setup. Do not recursively invoke Codex, merge, push or publish. Stop at
  the updated review packet for human review.

Use the minimal complete implementation and inspect callers, tests and formats
before editing, following [Ponytail's guide](https://github.com/DietrichGebert/ponytail/blob/main/AGENTS.md).

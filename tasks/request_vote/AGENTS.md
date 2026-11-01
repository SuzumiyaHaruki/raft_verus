# MadRaft RequestVote receiver generation — DRAFT

Read `tasks/request_vote/TASK.md`, the fixed `api.rs`, `caller.rs` and `rpc_tests.rs`. Complete `madraft/src/raft/vote_core.rs` and `madraft/proof/vote_lemmas.rs`; choose your own private representation, auxiliary specifications, invariants and lemmas. Generate code and proof together and repair using real diagnostics. Never import another experiment's code, patches or session.

Use the actual path `RaftHandle::new → start_rpc_server → RaftHandle::request_vote → Raft::request_vote → Core`. Do not create another service, move protocol decisions into the unverified host or change fixed targets, tests or checker. Do not use assume, admit, external_body, skipped verification or nontermination as a proof. Keep the disclosed host premises; proving RPC internals or the async executor is outside this task.

Build one actual behavior with its call premises, outputs and proof, then close receiver rules, arbitrary finite history and the two recovery entries. Local completion is not Election Safety: all five global properties, full functionality and conditional liveness remain OPEN. Preserve other student TODOs.

Keep a short `PROGRESS.md` with failures, attempts, checks and next obstacle; use `RESULT.md` to separate four local proofs, RPC/file tests, host premises and global OPEN goals. Follow the prompt's soft handoff deadline and return normally; the controller independently checks the stopped candidate. Do not change global tools or authentication, commit or publish. All authored LLM-facing material must be English. Keep Markdown prose paragraphs on one physical line.

```bash
python3 scripts/check.py obligation --task request-vote --timeout 120
python3 scripts/check.py safety --task request-vote --timeout 240
python3 scripts/check.py feedback --task request-vote --timeout 300
python3 scripts/check.py final --task request-vote --timeout 600
```

A suspected fixed-input defect is not permission to edit acceptance. In PROGRESS.md (madraft/PROGRESS.md for scoped tasks), use one `<!-- fixed-input-objection -->` JSON object followed by `<!-- /fixed-input-objection -->` with `file`, `test` (exact failing test or check), `conflict`, `evidence` (current independent log or local minimal reproducer), `reproduce` (command for human review only), `needs_fixed_change: true`, `recommendation` (unapplied), `candidate_revision` (from feedback) and `unresolved`. The controller may rerun the fixed failing item once, capped at 60 seconds charged to the cumulative budget, then stop NEEDS_REVIEW. It never executes your proposed shell command or adopts your suggested test change. Insufficient evidence gets one clarification opportunity; repeated unchanged checked inputs/failures stop for review. Report edits and unreferenced reproducer files are retained but are not implementation/proof progress.

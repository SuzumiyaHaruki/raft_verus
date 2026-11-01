# Joint MadRaft receiver and campaign generation — DRAFT

Generate the actual receiver, campaign start, self-vote and corresponding Verus proofs inside this workspace's `madraft/`. Read `tasks/election_start/TASK.md`, fixed api.rs/caller.rs and requirements/{scope,safety,environment,coverage}.md. The supplied entries are unfinished; never import earlier code, patches, sessions or proofs.

Choose private representations, auxiliary invariants, ghost state and proof decomposition yourself. Receive, start, recovery and term/role queries share one authoritative Core. The fixed observer records actual accepted self-choices and affirmative replies; a self-choice is not successful I/O, a counted vote or leadership. Protocol decisions, term changes, outbound contents and authorization belong to the same Cargo/Verus executable core. The host only serializes, performs real I/O, reports results and forwards unchanged actions.

Preserve fixed specifications, caller, original tests, requirements, licenses and host wiring. Do not use assume, admit, external/skipped protocol bodies or nontermination. Edit `madraft/src/raft/vote_core.rs`, `madraft/proof/vote_lemmas.rs` and your added implementation/proof dependencies. Report a reproducible fixed-interface contradiction instead of weakening it. Do not implement tallying, leader promotion, heartbeats, replication or arbitrary crash recovery.

Advance in actual behavioral increments: implement calls, inspect their outputs and premises, run independent checks, repair from diagnostics. Keep short `madraft/PROGRESS.md` and `madraft/RESULT.md` separating receiver rules, campaigns, mixed history, authorization, receiver/self recovery, RPC/file tests, host premises and global OPEN goals. Retain failures and same-run checkpoints; rollback never restores budget. Return normally by the soft handoff deadline.

Use existing tools.ini and offline tools; do not change global configuration, merge or publish. ELECTION_START_CHECKS_PASSED is local only; all five global properties, full MadRaft/2A, snapshots, applications and conditional liveness remain unfinished. All authored LLM-facing material must be English. Keep each Markdown prose paragraph on one physical line.

```bash
python3 scripts/check.py obligation --task election-start --timeout 180
python3 scripts/check.py safety --task election-start --timeout 240
python3 scripts/check.py final --task election-start --timeout 600
```

A suspected fixed-input defect is not permission to edit acceptance. In PROGRESS.md (madraft/PROGRESS.md for scoped tasks), use one `<!-- fixed-input-objection -->` JSON object followed by `<!-- /fixed-input-objection -->` with `file`, `test` (exact failing test or check), `conflict`, `evidence` (current independent log or local minimal reproducer), `reproduce` (command for human review only), `needs_fixed_change: true`, `recommendation` (unapplied), `candidate_revision` (from feedback) and `unresolved`. The controller may rerun the fixed failing item once, capped at 60 seconds charged to the cumulative budget, then stop NEEDS_REVIEW. It never executes your proposed shell command or adopts your suggested test change. Insufficient evidence gets one clarification opportunity; repeated unchanged checked inputs/failures stop for review. Report edits and unreferenced reproducer files are retained but are not implementation/proof progress.

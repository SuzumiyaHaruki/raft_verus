# Raft bootstrap — DRAFT

Read [README.md](README.md) and [review/REVIEW_PACKET.md](review/REVIEW_PACKET.md) first. Preserve existing user work. Do not modify the read-only `.git`, `.agents`, `.codex` or `.aws` directories.

All content supplied to an LLM must be in English: instructions, prompts, task definitions, contracts, assumptions, TCB entries, proof registries, source comments, diagnostics and referenced documentation, including review documents. This is a requirement, not a preference. Keep original evidence unchanged; do not feed non-English raw evidence to an LLM. Use a clearly labeled English explanation when needed, without presenting a translation as raw output.

By default, do not introduce hashes, frozen contracts, baselines or gates. An exception requires a concrete failure scenario and an explanation of why Git, versions, primary keys, transactions, uniqueness constraints, types and ordinary tests are insufficient. Preserve existing safeguards. Authentication, data security, irreversible operations and releases still follow project requirements. The narrowly required evidence/approval bindings are justified in [docs/workflow.md](docs/workflow.md). A draft is not approval.

Use the smallest complete implementation. Inspect callers, tests, configuration and data formats first; prefer reuse and the standard library over a generic framework. Follow the applicable guidance in [Ponytail AGENTS.md](https://github.com/DietrichGebert/ponytail/blob/main/AGENTS.md).

- Keep new specifications, contracts and tools DRAFT. Recommend decisions; never approve them for the user.
- This round delivers a bootstrap only. Do not launch real Codex tasks through the workflow, recursively invoke Codex, automatically merge or publish.
- Offline tests use executable fake fixtures. SIMULATED evidence cannot close Raft obligations.
- Do not hide gaps with `assume`, `admit`, skipped verification or fabricated results. Report OPEN/BLOCKED/NOT_RUN honestly.
- Changes to verification boundaries, assumptions, the TCB or acceptance rules require human review. AGENTS.md is not permission isolation.
- Stop at the review packet. Do not automatically proceed to full Raft implementation or proof development.

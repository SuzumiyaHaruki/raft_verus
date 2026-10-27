# Maintenance agent objective — DRAFT

This file is for the agent maintaining this repository with the user. It is not experiment input; candidate copies and generated prompts must omit it.

Our goal is to use AI to build a Rust Raft implementation and use Verus throughout development to establish its safety and conditional liveness. We are studying whether the generated executable actually satisfies the intended requirements. A scheduler, passing tests, a typechecked model or an unrelated proved helper is not that result.

Work in small increments of one coupled Raft implementation. For each increment, connect the natural-language requirement, formal target, actual executable body, representation relation and independent checks. Generate code and proof together; compare observed behavior with the target, diagnose failures, repair and rerun. Carry previous failed approaches forward so a new attempt can change its local design instead of repeating it. Preserve the target when changing representation or auxiliary invariants. Specification adequacy and global composition still need explicit arguments; local correctness does not imply cluster correctness.

Use functional tests, fault histories, actual Verus results and comparable measurements as distinct feedback. Measure only a candidate that passes required correctness checks. Liveness must state environmental assumptions and transfer progress to the executable; never infer it from finite traces or safety proofs.

Prepare concrete English assignments, the existing Python Codex workflow and configuration so the next experiment develops missing Raft behavior. Keep implementation choices small and readable. IDS informs incremental code/proof synthesis and feedback-guided repair; do not import a general agent framework or treat that paper as a proof of this Raft implementation.

Preserve user work and existing safeguards. Do not weaken oracles, add assumptions, use proof holes, or equate simulated generation with real model output. Leave new specifications and decisions DRAFT. Human review is our discussion after code delivery, not a checkpoint inside each execution. During maintenance, do not run real Codex tasks, recurse, merge or publish. Deliver actual evidence and remaining gaps in `review/REVIEW_PACKET.md`, then stop.

Keep each Markdown prose paragraph on one physical line. Preserve code blocks, tables and list structure.

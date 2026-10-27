# Trusted components

Status: DRAFT. These are trusted boundaries, not proved protocol facts. Actual tool observations are historical; no candidate version is automatically approved.

| ID | Trusted component | Claims and review |
|---|---|---|
| TCB-VERUS | Verus frontend, verification-condition generation and trusted implementation; doctor records the actual local version | All formal proofs. Review toolchain installation and versions |
| TCB-VSTD | Trusted definitions, axioms and library proofs in the installed vstd; its version must match Verus | Proofs using vstd. Inspect local source and licenses |
| TCB-SMT | The selected SMT solver and its Verus interface; unknown and timeout are not proofs | All SMT checks. Check versions and rejection of negative examples |
| TCB-RUST | Rust compiler, linker, runtime and target semantics | Actual executable behavior. Toolchain manifest; no fabricated version pinning |
| TCB-HOST | OS, process isolation, filesystem, hardware, clocks and declared resource limits | Integrity of actual execution and checking evidence. Local mode does not claim protection against same-UID tampering |
| TCB-ADAPTERS | Actual network identity and codecs, stable storage and timer adapters; no actual adapter or trusted external body is provided | A-S-ID/STORAGE/COMPLETION, A-L-*. Register and review concrete locations when introduced |
| TCB-APPLICATION | Host application persistence and deduplication satisfying A-S-APP | S-APPLY only, not library-level delivery properties. Application not provided; composition obligations remain OPEN |
| TCB-CHECKER | Candidate Python checker, schemas, configuration and evidence records; not a mathematical axiom | Conclusion recording and coverage decisions. Independent checks, offline controller tests and human review of delivered code; no runtime approval file or automatic semantic approval |

No unapproved source trust entries are allowed. No new assume/admit/external_body is permitted to manufacture a proof. verus-tla compatibility is unverified; do not introduce a substitute axiom library. Tool provenance and limitations are in ../docs/tools.md and ../docs/sources.md.

STATE-INIT reuses these boundaries. Cargo erases proofs from the same `node.rs`
that Verus checks; the protected initial-field relation and consumer are checker
inputs, not new axioms. The checker additionally recognizes the exact ghost-only
`use crate::initialization::init_result;` import. Review this policy adjustment
under ADR-08/10. Vec allocation and compiler/tool correctness remain trusted;
non-Copy Clone derives are not used by the constructor proof. No new host or
liveness assumption is introduced.

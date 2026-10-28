# Trusted components

Status: DRAFT. These are trusted boundaries, not proved protocol facts. Actual tool observations are historical; no candidate version is automatically approved.

| ID | Trusted component | Claims and review |
|---|---|---|
| TCB-VERUS | Verus frontend, verification-condition generation and trusted implementation; doctor records the actual local version | All formal proofs. Review toolchain installation and versions |
| TCB-VSTD | Trusted definitions, axioms and library proofs in the installed vstd; its version must match Verus | Proofs using vstd. Inspect local source and licenses |
| TCB-SMT | The selected SMT solver and its Verus interface; unknown and timeout are not proofs | All SMT checks. Check versions and rejection of negative examples |
| TCB-RUST | Rust compiler, linker, runtime and target semantics | Actual executable behavior. Toolchain manifest; no fabricated version pinning |
| TCB-HOST | OS, process isolation, filesystem, hardware, clocks and declared resource limits | Integrity of actual execution and checking evidence. Local mode does not claim protection against same-UID tampering |
| TCB-ADAPTERS | Network identity/codecs, storage and timers; supplied MadSim implementations are in experiments/madraft-poc/src/lib.rs and src/cluster.rs, without a formal adapter proof or production-storage claim | A-S-ID/STORAGE/COMPLETION, A-L-*. Register and review concrete locations when introduced |
| TCB-APPLICATION | Host application persistence and deduplication satisfying A-S-APP | S-APPLY only, not library-level delivery properties. The MadSim host models atomic application-plus-cursor persistence and duplicate delivery; production application and composition obligations remain OPEN |
| TCB-CHECKER | Candidate Python checker, schemas, configuration and evidence records; not a mathematical axiom | Conclusion recording and coverage decisions. Independent checks, offline controller tests and human review of delivered code; no runtime approval file or automatic semantic approval |

No unapproved source trust entries are allowed. No new assume/admit/external_body is permitted to manufacture a proof. verus-tla compatibility is unverified; do not introduce a substitute axiom library. Tool provenance and limitations are in ../docs/tools.md and ../docs/sources.md.

STATE-INIT reuses these boundaries. Cargo erases proofs from the same `node.rs` that Verus checks; the protected initial-field relation and consumer are checker inputs, not new axioms. The checker additionally recognizes the exact ghost-only `use crate::initialization::init_result;` import. Review this policy adjustment under ADR-08/10. Vec allocation and compiler/tool correctness remain trusted; non-Copy Clone derives are not used by the constructor proof. No new host or liveness assumption is introduced.

The persistence slice also reuses these boundaries. Its exact ghost-only import of `issue_result`, `complete_result` and `dispatch_result` is listed by the checker; arbitrary conditional executable paths remain rejected. Independent consumers, event tests and source projections are DRAFT checker inputs. The driver models a host disk separately from callbacks; it does not verify a real storage/network/recovery adapter. Non-Copy Clone is used only in host/test construction, never assumed by the coordinator proofs. The local progress theorem explicitly exposes successful notification service and noninterference as its premises; it adds no new global Raft liveness assumption or proof axiom. Review these checker/API additions under ADR-01/07/08/10.

The active cluster host executes the generated handle_event source but is outside its Verus proof. Its codec, event scheduling, fresh-incarnation assignment, atomic whole-image disk and application transaction are supplied test infrastructure under TCB-ADAPTERS/TCB-APPLICATION; their semantics remain pending ADR-05/07 review. Tests use the real MadSim scheduler and no separate protocol engine. No simulation result proves host correctness, global safety or liveness.

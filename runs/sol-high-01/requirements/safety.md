# Safety targets and executable correspondence — DRAFT

Use the supplied Verus definitions in [targets.rs](targets.rs). [target_checks.rs](target_checks.rs) checks examples of those definitions; [final.rs](final.rs) independently calls the implementation proofs you generate. 

| Target               | Required meaning                                                                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Election Safety      | Across the entire history, a group elects at most one distinct leader per term.                                                                     |
| Leader Append-Only   | While a node remains leader in the same term, its logical log only extends. Physical compaction preserves the represented prefix.                   |
| Log Matching         | Within a group, entries with equal index and term imply equal prefixes through that index, including command contents, even across different times. |
| Leader Completeness  | A prefix committed in term `t` appears at its original indices in every higher-term elected leader's log in that group.                           |
| State Machine Safety | All applications at a given group and index agree on the entry, including applications before crashes and those represented by installed snapshots. |

Implement and prove at most one durable vote per term, up-to-date candidate logs, conflicting-log rejection/truncation, majority agreement and the current-term restriction for advancing leader commitment, ordered application, and faithful snapshot boundaries. These are protocol obligations, not trusted environment premises.

Generate code and proofs together. Provide `madraft/proof/implementation.rs` with the interfaces consumed by [final.rs](final.rs), using `crate::targets` and importing the executable functions tested from `madraft/src/`. Describe all finite execution prefixes from actual initialization under [environment.md](environment.md). Prove initialization, transition preservation, executable correspondence and host-call preconditions; do not assume the desired safety properties or an unproved invariant.

Cover every role change, actual commit-cursor advancement, application, snapshot operation and crash/restart path, including effects in async wrappers. Preserve group and node identities, terms, original indices, exact command bytes and pre-crash history. Logical logs include their justified compacted prefixes. Do not filter execution down to passing traces or prove a disconnected model.

Prove that resident, created, installed, persisted, restored and applied snapshots faithfully represent an earlier committed prefix: boundary index and term agree, and the actual decoder yields the application state obtained by executing that prefix. Connect this claim to the real service and storage paths, retained suffix boundaries and committed/applied-prefix contract. A snapshot need not preserve a reversible command history.

Run `python3 scripts/check.py safety` to verify the complete proof composition. Definition checks and local lemmas do not establish implementation safety. Keep incomplete properties OPEN, and report gaps in executable coverage and the trust boundary for review.

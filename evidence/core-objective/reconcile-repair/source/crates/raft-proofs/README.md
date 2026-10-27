# Local proof entrypoints

Status: DRAFT. `reconcile.rs` imports the existing candidate model and the exact
`raft-verified/src/reconcile.rs` implementation. Its independently protected
`check_reconcile_contract` delegates to the executable operation and establishes
correspondence to `model::merged`, preservation of the prefix through `previous`,
and unchanged state on reported input errors. The field-preserving representation
is defined in `raft-spec/src/reconcile_view.rs`, outside ordinary repair scope.
Weakening the executable's postcondition cannot satisfy this independent consumer.
This reuses the existing protected checker boundary: finite runtime tests cannot
establish this universal contract, and source versioning cannot establish that a
changed annotation preserves its meaning. `reconcile_lemmas.rs` holds editable,
proved consequences for empty and matching input; it adds no trusted assumption.
The scan has a decreasing incoming-suffix length. These are local results;
they do not close the complete handler, global safety or conditional-liveness
obligations in [the proof checklist](../../input/proofs.md).

Run `python3 crates/raft-verified/check.py proof` from the repository root.
The command runs installed Verus with `--no-cheating`, without proof filtering.
For runtime tests, `check.py test` gives Cargo the same installed release's vstd
and macro library, plus its matching Rust 1.92 compiler and rustdoc in the child
process environment. The Verus macro erases specifications and proof blocks;
the executable body and types are the same source used by verification. There is
no generated alternate Raft implementation. The existing system default remains
unchanged. `RAFT_VERUS` and `RAFT_RUSTUP_HOME` can select another explicitly
installed compatible location; missing files produce BLOCKED instead of a pass.

The operation compares terms, as `merged` does. It deliberately keeps the old
command when overlapping terms match; the term-equality/log-content invariant
needed by the eventual full protocol remains unproved. The future caller must
check message and predecessor terms, committed-prefix constraints, conversion of
wire `u64` indices into `usize`, and persistence/response timing. The function
checks predecessor bounds and index-sum overflow itself. Rust Vec allocation
failure can abort; this is not a fallible-allocation or resource-liveness proof.

The trusted runtime boundary includes Rust/LLVM and the installed Verus vstd
contracts for Vec indexing, split_off, truncate and append. Clone derives support
tests/workload construction; the verified operation never calls or assumes them.
This Verus release warns that it does not attach contracts to the non-Copy Clone
derives. No local proof depends on those missing contracts.
`crates/proof-smoke` remains tool calibration, not Raft evidence.

`node_init.rs` imports `src/node.rs` and the shared initial-state types, exactly as
Cargo does. Its protected caller demands `init_result` for every input: valid
membership must succeed with the fields of `fresh_node`; invalid membership must
fail. The source reject-all seed intentionally fails. Run `check.py node-proof`
in a generated candidate. This local initial-field relation does not map missing
pending I/O or peer state, and cannot discharge full node/cluster refinement. The
only additional ghost import allowed by the checker is the exact `init_result`
specification import; executable conditional compilation remains rejected.

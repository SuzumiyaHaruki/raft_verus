# Current proof coverage

`python3 scripts/check.py proof madraft/proof/progress.rs` checks the same executable source used by the Cargo tests. The current entrypoint verifies 89 items with `--no-cheating`. No consensus function uses `assume`, `admit`, an external body or a skipped proof. This is local verification, not an aggregate proof of the five supplied targets.

| Source | Verified coverage |
| --- | --- |
| `src/raft/state.rs` | Initial term/role/vote; finite campaigning; monotone term adoption; same-term vote preservation; freshness; leader admission and winning guards. |
| `src/raft/log.rs` | Exact command copying/equality; indexed terms; appending; protected-prefix repair; complete follower batch repair/rejection/backtracking; compaction and logical prefix concatenation. |
| `src/raft/quorum.rs` | Counts distinct vote/member slots; resets and records votes; selects current-term majority commits; proves conditional quorum intersection. |
| `src/raft/cursor.rs` | Ordered application from the committed resident suffix; snapshot authorization; exact retained suffix and cursor transformations. |
| `src/raft/core.rs` | Initialization and preservation of the assembled synchronous core representation, including member bounds, cursor bounds, term bounds, self-votes, local leader vote counts and replication cursor bounds. Functional contracts cover persistent-image extraction/restoration, votes, elections, proposal extension, outgoing replication and follower committed-prefix preservation. Persistent-state changes require the dirty flag. Pending snapshot application blocks command emission until acknowledgment of the exact current snapshot. |
| `initialization.rs` | The real constructor refines a fresh target view; a conditional finite-map base case establishes only the zero-event history. Host assembly and all nonempty transitions remain open. |
| `src/raft/storage.rs` | Primary-pair generation checks, newest coherent generation selection, checked generation increments and the conditional repair-authority guard. |

The representation invariant is initialized and preserved by the core handlers. It does not assert that a peer's match index has a matching-prefix certificate, that vote replies have durable origins, that committed prefixes are present in future leaders, or that snapshots decode to the fold of a committed prefix. Those require a cluster/message/storage/history invariant that has not been supplied. The two-vote and repair-authority lemmas retain explicit premises; they are not unconditional safety theorems.

The async host in `src/raft/raft.rs` uses the core for consensus decisions and the verified storage generation guards. It serializes mutation, persistence and publication with a gate, publishes applications after persistence, and preserves a coherent backup while replacing primary files. Real filesystem/decoder regressions cover every write boundary and a repeated crash during recovery. Formal correspondence for these host/storage operations, including interrupted operations, remains open.

`implementation.rs` and the full `Execution`/`execution`/`observe` refinement consumed by `requirements/final.rs` are missing. Consequently `python3 scripts/check.py safety` and `final` fail. The five global properties, full observation shape/coverage and actual service snapshot fidelity are OPEN. A passing local entrypoint, definition check or finite simulation is not substituted for them.

# Executable Raft development

Status: DRAFT. The experiment starts with failing protocol bodies and generates Rust code plus implementation-layer Verus contracts and proofs. `src/state.rs`, `types.rs` and `io.rs` provide observable vocabulary; `node_state.rs` supplies an editable Node layout, view, representation invariant and adapters. `contracts.rs` supplies vacuous local predicates that must be replaced or reorganized to satisfy the independent consumers. The separate `madraft.rs` follower constructor and durable vote handler are explicitly supplied prior feasibility code. The active MadSim extension calls the missing `protocol.rs::handle_event`, which Python-launched Codex must generate; `step` remains the legacy coordinator interface.

`check.py build` succeeds on the scaffold; behavioral tests and corresponding independent proofs fail until generation. The default MADRAFT plan generates the shared constructor through STATE-INIT, which checks eight initialization cases, its proof and replay. There is no separate completed constructor. Independent `tests/protocol.rs` cases constrain the generated event path. RUNTIME-PERSIST adds six tests, proof and replay over the same Node; reconciliation has 13 operation/replay cases. Tests use `observe()`, not internal fields. `from_observation()` is only the disclosed boundary-test adapter, including serial exhaustion; it does not implement recovery. Completed selected tasks are rechecked after representation/contract changes. See [inputs](../../input/starting-point.md) and [actual results](../../review/REVIEW_PACKET.md).

From the repository root:

```sh
python3 crates/raft-verified/check.py build
python3 crates/raft-verified/check.py test
python3 crates/raft-verified/check.py proof
python3 crates/raft-verified/check.py replay
python3 crates/raft-verified/check.py bench
```

These are real local checks. The wrapper uses already installed Verus and its matching isolated Rust 1.92, without downloads or changes to the user default. Default Cargo alone lacks the installed Verus macro dependency flags. See [the proof build path](../raft-proofs/README.md) for the shared-source connection.

The saved [trace](examples/reconcile.trace) contains seed 17 and all seven explicit events. Replay emits expected and observed log terms for every event, including an invalid predecessor; it is local operation coverage, not a network/crash simulator. To replay another explicit trace:

```sh
python3 crates/raft-verified/check.py replay -- /absolute/path/to/scenario.trace
```

The measurement example runs in release mode at log lengths 0, 16, 256 and 4096, with matching-tail, first-conflict and missing-suffix workloads. It emits five samples of 32 operations each, machine information, workload size and timing boundary. Construction is outside the timer; consuming the incoming vector and timer overhead are included. Output log destruction is outside the timer. Timing is data only: there is no approved performance threshold or reference version and no claim of cluster throughput. Run it without competing heavy jobs. The independent controller binds reports to source/check inputs; a standalone run's working-tree label is not immutable revision evidence.

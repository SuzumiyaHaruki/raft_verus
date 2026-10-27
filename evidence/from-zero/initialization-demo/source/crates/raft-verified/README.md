# Executable Raft development

Status: DRAFT. This is the experiment scaffold, with shared types, API contracts and intentionally incorrect bodies in `node.rs`, `reconcile.rs` and `persistence.rs`. No working Raft operation is supplied. AI generates executable code and proof together in a saved candidate. The previous unused OperationSequence helper and its two tests are preserved only as an excluded offline fixture. All independent initialization, reconciliation, replay and persistence integration oracles remain intact.

`check.py build` should succeed on the scaffold. `test`, `proof`, `node-proof`, `persist-proof` and `progress-proof` must not be interpreted as passing before generation. STATE-INIT checks all eight initialization cases with `test --test node_init`, its local proof and replay; it does not require future operation tests to pass. RUNTIME-PERSIST then adds its six independent tests, proof and replay. Reconciliation has 13 operation/replay cases. Completed selected tasks are rechecked during continuation. See [the experiment inputs](../../input/starting-point.md) and [actual results](../../review/REVIEW_PACKET.md).

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

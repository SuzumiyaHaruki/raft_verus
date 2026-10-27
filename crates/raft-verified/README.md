# Executable Raft development

Status: DRAFT. `reconcile_log` is the suffix operation intended for a future AppendEntries handler. The rest of the library still provides interface types and token helpers plus shared initial-state types; it is not a functioning Raft node. `node.rs` is the next generation task's reject-all seed. `check.py test` now includes eight initialization oracles, so it intentionally fails on that seed. Existing regressions remain available through the `reconcile-tests` profile. STATE-INIT candidates must pass all 23 tests, `node-proof`, `node-replay`, existing reconciliation proofs and `node-bench`. The offline repair command and actual candidate are linked in the root review packet.

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

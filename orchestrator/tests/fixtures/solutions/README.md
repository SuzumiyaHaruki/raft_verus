# Offline solutions — SIMULATED generation only

These files are explicit scripted test data, excluded with all of `orchestrator/tests` from candidates and input manifests. The real controller never installs them. `real_repair_demo.py` supplies edits to a fake process, then runs actual Rust/Verus checks.

`reconcile.rs` and `persistence.rs` provide executable examples; `contracts.rs` provides their local specification example. `redesigned_state.rs` demonstrates a different Node layout and nontrivial representation invariant. `persistence_lemmas.rs` and `progress_lemmas.rs` supply proof strategies; the constructor fixture is `../node_init_solution.rs`.

These are not real-model results or complete Raft proofs. Never retrieve them as a real experiment's starting implementation or generated answer.

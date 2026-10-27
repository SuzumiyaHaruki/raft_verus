# Reconciliation measurements — DRAFT

Actual core microbenchmark, executed serially after tests and proofs on 2026-10-27. No other heavy task from this session was running. Background host activity is not isolated; the samples show substantial variation. This is measured data, not a performance-pass or regression verdict.

Command: `python3 -m orchestrator verify --profile reconcile-bench`. The checker runs `python3 crates/raft-verified/check.py bench`, which builds the shared source with installed Rust 1.92.0 (`ded5c06cf`, 2025-12-08), Cargo 1.92.0, and the installed Verus macro dependencies in release mode. No toolchain was installed or replaced.

Checked run: `20261027T053913Z-f967275348bf`. The existing input binding is `f1cc010a6c193eb18b4d48cca24790b6de8efd35d785746361e391e22c5fbbc0`. There is no local Git HEAD in this checkout. [Independent result](../../evidence/revision/final-reconcile-bench.stdout) records the source/checker binding; [tool versions](../../evidence/revision/final-doctor.stdout) identify the compiler. The measured source and harness remain in this repository.

Machine: Intel(R) Core(TM) Ultra 5 245KF; architecture x86_64; kernel 6.8.0-138-generic. Five samples per case, 32 operations per sample; 32-byte commands, batch size 1, concurrency 1. Each sample is the mean nanoseconds per operation of its 32 calls. The table reports all five samples and their median.

Timing includes the reconciliation call, timer overhead and incoming-vector consumption. Input construction and final log destruction are excluded. This measures a core helper, not proposal acceptance, commitment, delivery, storage durability or cluster throughput.

| Log entries | Workload | Samples (ns/operation) | Median (ns/operation) |
|---|---|---|---|
| 0 | matching_tail | 58.00, 50.69, 51.41, 50.94, 59.88 | 51.41 |
| 0 | first_conflict | 303.44, 129.69, 129.19, 129.06, 130.72 | 129.69 |
| 0 | missing_suffix | 136.09, 129.53, 124.50, 105.00, 106.50 | 124.50 |
| 16 | matching_tail | 184.22, 157.78, 157.78, 152.94, 156.12 | 157.78 |
| 16 | first_conflict | 329.53, 287.81, 290.56, 289.09, 285.44 | 289.09 |
| 16 | missing_suffix | 425.31, 342.00, 334.94, 337.69, 337.75 | 337.75 |
| 256 | matching_tail | 1694.53, 8867.41, 1572.09, 1502.53, 1359.72 | 1572.09 |
| 256 | first_conflict | 6259.00, 5661.41, 6277.28, 5196.38, 6373.38 | 6259.00 |
| 256 | missing_suffix | 3008.53, 2315.81, 2247.28, 2618.25, 2584.41 | 2584.41 |
| 4096 | matching_tail | 47242.12, 30407.19, 20394.06, 34478.06, 26611.62 | 30407.19 |
| 4096 | first_conflict | 137116.34, 140627.06, 218356.31, 177395.47, 299060.41 | 177395.47 |
| 4096 | missing_suffix | 499098.78, 502062.19, 302529.88, 233170.59, 257362.00 | 302529.88 |

Process resources: `{"elapsed_seconds": 0.273262379, "kind": "MEASUREMENT_RESOURCES", "peak_rss": "VmHWM:\t    3016 kB", "scope": "measurement process including workload setup"}`.

No approved reference version or threshold exists. The 256-entry matching case and several 4096-entry cases vary markedly between samples; short-call timing, allocation/cache behavior and host scheduling can influence these data. Attribution is unknown. Repeat comparable measurements before drawing optimization conclusions. Correctness tests and proofs must be rerun after any algorithm change.

[Complete raw output](../../evidence/runs/20261027T053913Z-f967275348bf/reconcile-bench.stdout.log) includes resource observations and full metadata; [command record](../../evidence/revision/commands.json) includes arguments, duration and exit status.

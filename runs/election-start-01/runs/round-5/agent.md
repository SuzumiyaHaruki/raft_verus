Saved the handoff in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md).

- Build and all eight proof entries pass, including all four recovery entries.
- Each seed 1–5 passes 15/16 tests. No tool timeouts occurred.
- The remaining assertion incorrectly expects a closed observation channel to time out. The exact upstream correction is documented, unapplied because tests are protected.
- No alternate active vote-changing path was found. Failures and cumulative accounting remain retained.

Local acceptance remains **INCOMPLETE**. Serialization, truthful I/O/readback, retention, forwarding and cancellation remain host premises. All five global safety targets, full labs, snapshots and liveness remain **OPEN**.
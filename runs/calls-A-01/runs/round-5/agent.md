Fixed a reproduced snapshot-delivery bug: later compaction could invalidate an earlier queued snapshot and cause an application gap.

- New regression and crash tests pass on seeds 1–2.
- Retained Raft, KV and shard tests pass on seed 1.
- Verus: 41 verified, 0 errors.
- Global safety and full executable correspondence remain OPEN.

Saved evidence and next steps in [PROGRESS.md](/home/nitro/Desktop/raft_verus/runs/calls-A-01/PROGRESS.md).
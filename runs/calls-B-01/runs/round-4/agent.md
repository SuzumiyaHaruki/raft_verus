Saved shard migration, retry, deduplication and recovery work, plus updated reports.

- Shard suite: 11 pass; deletion-size test fails at 117689 > 117000 bytes.
- Full Raft suite passes on seed 5.
- Verus: 33 verified, 0 errors.

Next: reduce redundant retained storage. Global safety, correspondence and snapshot-fidelity proofs remain OPEN.
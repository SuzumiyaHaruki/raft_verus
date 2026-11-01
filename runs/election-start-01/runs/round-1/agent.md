Implemented the shared Core and proofs. All eight formal checks pass; 15/16 tests pass on every seed.

Final acceptance remains **INCOMPLETE**: the protected restart test cancels the channel’s sole sender, then incorrectly expects receiving from that closed channel to time out. Preserved the test and added a reproducible diagnostic.

[RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) records evidence, reproduction commands and host premises. Serialization, truthful I/O/readback and forwarding remain unverified. All five global safety targets remain OPEN.
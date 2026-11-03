# Student implementation scope — DRAFT

The source is MadRaft at `8ec4565ecd3844f54f52865f2d7b3da4044f657f`. Complete the missing work identified by its source comments and retained tests. The supplied structs and empty request types are upstream placeholders, not an approved final data representation.

| Lab                 | Required work                                                                                                                                                                                   |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Raft 2A             | Leader election, randomized timeouts, term handling, voting and heartbeat behavior.                                                                                                              |
| Raft 2B             | Log replication, conflict repair, concurrent proposals, agreement and commit/application authorization.                                                                                          |
| Raft 2C             | Persist and restore consensus state through crashes and partitions; preserve durable voting and log decisions.                                                                                   |
| Raft 2D             | Snapshot creation, installation, compacted-log boundaries and recovery; retain all snapshot fault tests.                                                                                         |
| KV 3A/3B            | Complete server state, client routing/retries and replicated operations, then snapshot/recovery and size constraints. Derive request deduplication and retry behavior from the lab expectations. |
| Shard controller 4A | Complete replicated shard configuration operations and preserve the supplied `Config` structure.                                                                                               |
| Sharded KV 4B       | Complete clients, server state, configuration changes, shard transfer, retries, snapshots and recovery.                                                                                          |

A group has a fixed server set for its lifetime. The shard controller changes ownership of shards among groups; the upstream lab does not ask you to implement joint-consensus membership changes inside an individual Raft group. Give the controller group and each data group distinct stable identities, including across restart. Group identity and Raft term/index must not be conflated with a shard configuration number. Retiring a group does not erase its observable history.

Complete all listed labs and retained tests. Formal targets and executable correspondence requirements are in [safety.md](safety.md); environment assumptions and trust boundaries are in [environment.md](environment.md).

# Original lab test inventory — DRAFT

The pinned upstream contains 57 declared `#[madsim::test]` tests: 56 active and one upstream-ignored empty linearizability placeholder. Their names and expectations are retained. The ignored `shardkv::tests::unreliable3_4b` is not passing coverage. The originally commented-out KV linearizability test remains commented; no added test is counted as upstream coverage. `scripts/check.py inventory` lists compiled tests, including the three added host-plumbing regressions.

| Source | Count | Scope |
| --- | ---: | --- |
| `madraft/src/raft/tests.rs` | 23 | 2A elections; 2B agreement/conflict repair; 2C persistence/faults; 2D snapshots |
| `madraft/src/kvraft/tests.rs` | 19 | 3A KV/concurrency/partitions/persistence; 3B snapshots and size |
| `madraft/src/shard_ctrler/tests.rs` | 2 | 4A shard configurations and balancing |
| `madraft/src/shardkv/tests.rs` | 13 | 4B shard migration, faults, snapshots and concurrent clients |

## raft

```text
raft::tests::initial_election_2a
raft::tests::reelection_2a
raft::tests::many_election_2a
raft::tests::basic_agree_2b
raft::tests::fail_agree_2b
raft::tests::fail_no_agree_2b
raft::tests::concurrent_starts_2b
raft::tests::rejoin_2b
raft::tests::backup_2b
raft::tests::count_2b
raft::tests::persist1_2c
raft::tests::persist2_2c
raft::tests::persist3_2c
raft::tests::figure_8_2c
raft::tests::unreliable_agree_2c
raft::tests::figure_8_unreliable_2c
raft::tests::reliable_churn_2c
raft::tests::unreliable_churn_2c
raft::tests::snapshot_basic_2d
raft::tests::snapshot_install_2d
raft::tests::snapshot_install_unreliable_2d
raft::tests::snapshot_install_crash_2d
raft::tests::snapshot_install_unreliable_crash_2d
```

## kvraft

```text
kvraft::tests::basic_3a
kvraft::tests::concurrent_3a
kvraft::tests::unreliable_3a
kvraft::tests::unreliable_one_key_3a
kvraft::tests::one_partition_3a
kvraft::tests::many_partitions_one_client_3a
kvraft::tests::many_partitions_many_clients_3a
kvraft::tests::persist_one_client_3a
kvraft::tests::persist_concurrent_3a
kvraft::tests::persist_concurrent_unreliable_3a
kvraft::tests::persist_partition_3a
kvraft::tests::persist_partition_unreliable_3a
kvraft::tests::snapshot_rpc_3b
kvraft::tests::snapshot_size_3b
kvraft::tests::snapshot_recover_3b
kvraft::tests::snapshot_recover_many_clients_3b
kvraft::tests::snapshot_unreliable_3b
kvraft::tests::snapshot_unreliable_recover_3b
kvraft::tests::snapshot_unreliable_recover_concurrent_partition_3b
```

## shard_ctrler

```text
shard_ctrler::tests::basic_4a
shard_ctrler::tests::multi_4a
```

## shardkv

```text
shardkv::tests::static_shards_4b
shardkv::tests::join_leave_4b
shardkv::tests::snapshot_4b
shardkv::tests::miss_change_4b
shardkv::tests::concurrent1_4b
shardkv::tests::concurrent2_4b
shardkv::tests::concurrent3_4b
shardkv::tests::unreliable1_4b
shardkv::tests::unreliable2_4b
shardkv::tests::unreliable3_4b
shardkv::tests::challenge1_delete_4b
shardkv::tests::challenge2_unaffected_4b
shardkv::tests::challenge2_partial_4b
```

Run a family with `python3 scripts/check.py test FAMILY::tests:: --seed 1`, or all families with `python3 scripts/check.py test`. Set `MADSIM_TEST_NUM=100` for repeated seeds or `MADSIM_TEST_CHECK_DETERMINISTIC=1` for the simulator replay check. A simulator result is finite testing, never a universal safety or liveness proof.

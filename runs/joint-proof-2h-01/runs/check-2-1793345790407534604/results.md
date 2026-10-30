# Independent check result

INCOMPLETE; semantic review NOT_PERFORMED; implementation acceptance NOT_ESTABLISHED.

| Goal | Formula | Connection evidence | Implementation task |
| --- | --- | --- | --- |
| election_safety | OPEN | OPEN | OPEN |
| leader_append_only | OPEN | OPEN | OPEN |
| log_matching | OPEN | OPEN | OPEN |
| leader_completeness | OPEN | OPEN | OPEN |
| state_machine_safety | OPEN | OPEN | OPEN |
| execution_link | OPEN | OPEN | OPEN |
| snapshot_observation | OPEN | OPEN | OPEN |
| nonempty_execution | OPEN | OPEN | OPEN |
| safety | OPEN | OPEN | OPEN |

Formula scope: candidate-defined relations and observations. A projection lemma or an empty snapshot model does not establish actual program coverage.

- initialization: OPEN / outer_paths_open. Actual constructors establish the execution initial state and callee preconditions.
  Local unit: PASS; unconnected paths: madraft/src/raft/raft.rs::new.
- transitions: OPEN / outer_paths_open. Actual handlers and their callers authorize every safety-relevant state change.
  Local unit: PASS; unconnected paths: madraft/src/raft/raft.rs::request_vote.
- outputs: OPEN / outer_paths_open. Actual messages, storage completions and application outputs match the history, including release order.
  Local unit: PASS; unconnected paths: madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote.
- recovery: OPEN / outer_paths_open. Actual durable decoding and recovery re-establish subsequent callers' preconditions and history relation.
  Local unit: PASS; unconnected paths: madraft/src/raft/raft.rs::restore.
- snapshots: OPEN / missing_entry_or_dependency. Actual snapshot creation, installation, decoding and recovery match observations.
  Local unit: OPEN; unconnected paths: madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot.

Current increment: PARENT_UNIT_CHECKED_PREMISES_UNREVIEWED; first obstacle: {"reason": "outer_paths_open", "paths": ["madraft/src/raft/raft.rs::new", "madraft/src/raft/raft.rs::request_vote"], "premises": "Async constructor and filesystem completion composition, actual network provenance, deployment group identity, crash histories and cluster induction remain open. A matching operation token alone is not durable evidence. Local operation histories cover only the implemented vote segment."}


Placeholder source diagnostics (not a semantic completeness check): madraft/src/kvraft/client.rs::call, madraft/src/kvraft/server.rs::apply, madraft/src/raft/raft.rs::apply, madraft/src/raft/raft.rs::cond_install_snapshot, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::start, madraft/src/shard_ctrler/server.rs::apply, madraft/src/shardkv/client.rs::append, madraft/src/shardkv/client.rs::get, madraft/src/shardkv/client.rs::put, madraft/src/shardkv/server.rs::apply, madraft/src/shardkv/server.rs::new

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

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

- initialization: OPEN / not checked. Actual constructors establish the execution initial state and callee preconditions.
- transitions: OPEN / not checked. Actual handlers and their callers authorize every safety-relevant state change.
- outputs: OPEN / not checked. Actual messages, storage completions and application outputs match the history, including release order.
- recovery: OPEN / not checked. Actual durable decoding and recovery re-establish subsequent callers' preconditions and history relation.
- snapshots: OPEN / not checked. Actual snapshot creation, installation, decoding and recovery match observations.

Current increment: PREPARATION; first obstacle: {"check": "current_entry", "entry": "madraft/proof/implementation.rs", "reason": "verification_failure", "log": "/home/nitro/Desktop/raft_verus/runs/joint-proof-2h-01/runs/checks/1793347140587728870/current_entry.log"}


Placeholder source diagnostics (not a semantic completeness check): madraft/src/kvraft/client.rs::call, madraft/src/kvraft/server.rs::apply, madraft/src/raft/raft.rs::apply, madraft/src/raft/raft.rs::cond_install_snapshot, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::start, madraft/src/shard_ctrler/server.rs::apply, madraft/src/shardkv/client.rs::append, madraft/src/shardkv/client.rs::get, madraft/src/shardkv/client.rs::put, madraft/src/shardkv/server.rs::apply, madraft/src/shardkv/server.rs::new

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

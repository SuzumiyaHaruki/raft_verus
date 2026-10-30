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

- initialization: OPEN / connection_source_not_imported. Actual constructors establish the execution initial state and callee preconditions.
- transitions: OPEN / connection_source_not_imported. Actual handlers and their callers authorize every safety-relevant state change.
- outputs: OPEN / connection_source_not_imported. Actual messages, storage completions and application outputs match the history, including release order.
- recovery: OPEN / connection_source_not_imported. Actual durable decoding and recovery re-establish subsequent callers' preconditions and history relation.
- snapshots: OPEN / connection_source_not_imported. Actual snapshot creation, installation, decoding and recovery match observations.

Placeholder source diagnostics (not a semantic completeness check): 

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Repeated checked gaps and the previous attempted approaches are shown below. Reconsider the relevant representation, implementation or proof decomposition in this same workspace; preserve unrelated progress and fixed requirements. Do not import an earlier answer. Running out of checking time or missing tools calls for scheduling/environment repair, not an invented protocol counterexample.

Distinguish an executable bug, a model allowing impossible behavior, a local condition that cannot be preserved, a missing caller proof and solver difficulty. An unsafe initialized model trace cannot be removed merely by asserting an invariant that its steps do not preserve. Record why the old approach failed and the change being tried. Run the relevant real tests and proof unit with all dependencies, save the evidence and return normally by the soft deadline.

Workspace: /home/nitro/Desktop/raft_verus/runs/calls-B-01. Soft handoff deadline: 2026-10-30T04:55:23+00:00 (540.0 seconds from now). Save code and PROGRESS.md, then return normally by this deadline. The hard watchdog is 600.0 seconds; it is a last resort, not a planned handoff. There is no CLI pause signal. Stop starting long checks near handoff.

Cumulative budget remaining: 2482.7 seconds. Round: 3/8.

Diagnostics refer to the stopped candidate. Edit this workspace’s madraft/ files, not check snapshots.

Choose a concrete required behavior or an actual-execution connection for this increment. State the running functions and callers involved, where input/messages/storage results originate, who establishes each callee precondition (including after recovery), how state changes and outputs enter the proved history, and the dependency or missing functionality this work will close. Give the next real check command. Consecutive proof-focused increments are allowed when their implementation-level purpose is explicit.

When a model theorem passes but an actual path remains disconnected, work on that connection instead of declaring the target complete. Defining runtime_step to equal model_step, importing a function, or proving a test-only wrapper is insufficient. Verify actual constructors, callers and call compositions. Internal Insert/Commit-like actions still need their actual authorization; majority support and correct log contents are protocol obligations, not environment facts. Model every real output, including obsolete or erroneous ones.

Keep protocol decisions in verified code or retain an explicit gap: voting, log selection, commit/application authorization, output release and recovery interpretation cannot be trusted merely because they occur in an async wrapper. Primitive network/storage/scheduler contracts remain the stated boundary. Extend execution and observations when functionality grows. Snapshot formulas over absent snapshot behavior do not close snapshot work. Recovery may reattach ghost history, but must derive its relation from real durable state and established history, then prove subsequent caller preconditions.

Use the connection rows and independent formula/unit results to identify the next unclosed path. Candidate descriptions remain unverified; a passing nominated proof unit is not automatic evidence that all actual callers or paths are covered.

Independent candidate revision: check-2-1793335555416045549

Actual check outcomes:
build: PASS — checked
inventory: PASS — checked
tests_seed_1_shard_ctrler_tests: OPEN — test_or_build_failure
tests_seed_1_raft_tests: PASS — checked
tests_seed_1_test_support: PASS — checked
remaining_functional_coverage: OPEN — not_run_in_feedback
foundation: PASS — checked
definitions: PASS — checked
execution_link: OPEN — missing_entry_or_dependency
election_safety: OPEN — missing_entry_or_dependency
leader_append_only: OPEN — missing_entry_or_dependency
log_matching: OPEN — missing_entry_or_dependency
leader_completeness: OPEN — missing_entry_or_dependency
state_machine_safety: OPEN — missing_entry_or_dependency
snapshot_observation: OPEN — missing_entry_or_dependency
nonempty_execution: OPEN — missing_entry_or_dependency
linkage_review: OPEN — missing_implementation_entry
connection_initialization: OPEN — connection_source_not_imported
connection_transitions: OPEN — connection_source_not_imported
connection_outputs: OPEN — connection_source_not_imported
connection_recovery: OPEN — connection_source_not_imported
connection_snapshots: OPEN — connection_source_not_imported
safety: OPEN — missing_entry_or_dependency
proof: PASS — checked

Affected: tests_seed_1_shard_ctrler_tests
{"category": "functional_failure", "message": "test_or_build_failure", "dependency": "tests_seed_1_shard_ctrler_tests", "detail": "d_ctrler::tests::basic_4a ... FAILED\ntest shard_ctrler::tests::multi_4a ... FAILED\n\nfailures:\n\n---- shard_ctrler::tests::basic_4a stdout ----\n\nthread '<unnamed>' (33633) panicked at src/kvraft/client.rs:59:13:\nnot yet implemented: handle RPC results\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\ncontext: node=0 \"madsim-main\", task=0 (spawned at /home/nitro/Desktop/raft_verus/target/madraft-cargo/git/checkouts/madsim-42d1abda922981f9/519950e/madsim/src/sim/runtime/mod.rs:129:19)\nnote: run with `MADSIM_TEST_SEED=1` environment variable to reproduce this error\n\n---- shard_ctrler::tests::multi_4a stdout ----\n\nthread '<unnamed>' (33635) panicked at src/kvraft/client.rs:59:13:\nnot yet implemented: handle RPC results\ncontext: node=0 \"madsim-main\", task=32 (spawned at /home/nitro/Desktop/raft_verus/target/madraft-cargo/git/checkouts/madsim-42d1abda922981f9/519950e/madsim/src/sim/runtime/mod.rs:129:19)\nnote: run with `MADSIM_TEST_SEED=1` environment variable to reproduce this error\n\n\nfailures:\n    shard_ctrler::tests::basic_4a\n    shard_ctrler::tests::multi_4a\n\ntest result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 61 filtered out; finished in 0.00s\n\n"}
Raw logs: /home/nitro/Desktop/raft_verus/runs/.controller/calls-B-01/check-2-1793335555416045549/tests_seed_1_shard_ctrler_tests.log

Affected: remaining_functional_coverage
{"category": "not_run", "message": "not_run_in_feedback", "dependency": "remaining_functional_coverage", "detail": "Final must run every retained test family on every configured seed."}
Raw logs: 

Affected: execution_link, election_safety, leader_append_only, log_matching, leader_completeness, state_machine_safety, snapshot_observation, nonempty_execution, safety
{"category": "not_run", "message": "missing_entry_or_dependency", "dependency": "madraft/proof/implementation.rs", "detail": ""}
Raw logs: 

Affected: linkage_review
{"category": "missing_implementation_entry", "message": "missing_implementation_entry", "dependency": "linkage_review", "detail": "Create the execution/observation/refinement interface and its real dependencies; do not supply assumed invariants."}
Raw logs: 

Affected: connection_initialization
{"category": "connection_source_not_imported", "message": "connection_source_not_imported", "dependency": "connection_initialization", "detail": ""}
Raw logs: 

Affected: connection_transitions
{"category": "connection_source_not_imported", "message": "connection_source_not_imported", "dependency": "connection_transitions", "detail": ""}
Raw logs: 

Affected: connection_outputs
{"category": "connection_source_not_imported", "message": "connection_source_not_imported", "dependency": "connection_outputs", "detail": ""}
Raw logs: 

Affected: connection_recovery
{"category": "connection_source_not_imported", "message": "connection_source_not_imported", "dependency": "connection_recovery", "detail": ""}
Raw logs: 

Affected: connection_snapshots
{"category": "connection_source_not_imported", "message": "connection_source_not_imported", "dependency": "connection_snapshots", "detail": ""}
Raw logs: 

Formula / actual-connection / implementation-task outcomes:
{
  "election_safety": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "leader_append_only": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "log_matching": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "leader_completeness": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "state_machine_safety": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "execution_link": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "snapshot_observation": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "nonempty_execution": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  },
  "safety": {
    "formula_status": "OPEN",
    "formula_scope": "Candidate-defined relations and observations; not automatic runtime coverage.",
    "connection_status": "OPEN",
    "checked_connection_units": [],
    "open_connections": [
      "initialization",
      "transitions",
      "outputs",
      "recovery",
      "snapshots"
    ],
    "implementation_task": "OPEN"
  }
}

initialization: OPEN (connection_source_not_imported). Actual constructors establish the execution initial state and callee preconditions.
Candidate declaration, not accepted coverage: {"functions": "madraft/src/raft/raft.rs::new, madraft/src/raft/raft.rs::try_new, madraft/src/raft/state.rs::fresh", "entry": "madraft/proof/progress.rs", "open_premises": "Local fresh checked; actual async constructor, group identity and runtime initial relation remain unproved."}

transitions: OPEN (connection_source_not_imported). Actual handlers and their callers authorize every safety-relevant state change.
Candidate declaration, not accepted coverage: {"functions": "madraft/src/raft/raft.rs::start, madraft/src/raft/raft.rs::append_entries, madraft/src/raft/raft.rs::advance_commit, madraft/src/raft/state.rs::commit_from_matches", "entry": "madraft/proof/progress.rs", "open_premises": "Checked real-vector count and cursor authorization; actual log mutation, peer acknowledgment provenance and async caller composition remain OPEN."}

outputs: OPEN (connection_source_not_imported). Actual messages, storage completions and application outputs match the history, including release order.
Candidate declaration, not accepted coverage: {"functions": "madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_heartbeats, madraft/src/raft/raft.rs::apply", "entry": "madraft/proof/progress.rs", "open_premises": "Actual message/storage/application output histories and release order remain unproved."}

recovery: OPEN (connection_source_not_imported). Actual durable decoding and recovery re-establish subsequent callers' preconditions and history relation.
Candidate declaration, not accepted coverage: {"functions": "madraft/src/raft/raft.rs::restore, madraft/src/raft/state.rs::restore_successor", "entry": "madraft/proof/progress.rs", "open_premises": "Actual decoding, durable-history relation, restored log invariants and subsequent async call composition remain OPEN."}

snapshots: OPEN (connection_source_not_imported). Actual snapshot creation, installation, decoding and recovery match observations.
Candidate declaration, not accepted coverage: {"functions": "madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::install_snapshot, madraft/src/raft/raft.rs::cond_install_snapshot, madraft/src/raft/state.rs::can_snapshot", "entry": "madraft/proof/progress.rs", "open_premises": "Runtime implemented/tested; decoder fidelity, committed-prefix observation, bounded persistence and full caller proof remain OPEN."}

Still-present functional placeholders (source diagnostic, not exhaustive): madraft/src/kvraft/client.rs::call, madraft/src/kvraft/server.rs::apply, madraft/src/shard_ctrler/server.rs::apply, madraft/src/shardkv/client.rs::append, madraft/src/shardkv/client.rs::get, madraft/src/shardkv/client.rs::put, madraft/src/shardkv/server.rs::apply, madraft/src/shardkv/server.rs::new

Next: select an unresolved required behavior or proof dependency; record the chosen next action in PROGRESS.md. Use python3 scripts/check.py top NAME for an isolated unit, feedback for bounded mixed checks, or safety for formal composition. Exact commands, raw errors and uncovered test names are in the independent results.json. Final requires all test families/seeds on one stopped candidate.

Verus checks the supplied definitions and imported bodies. Source linkage and nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, caller preconditions and the declared host TCB. Conditional liveness remains OPEN.

Historical functional coverage (not current-candidate acceptance):
{
  "raft::tests::": {
    "last_attempt": 2,
    "last_completed": 2,
    "last_result": "PASS",
    "candidate_revision": "check-2-1793335555416045549",
    "log": "/home/nitro/Desktop/raft_verus/runs/.controller/calls-B-01/check-2-1793335555416045549/tests_seed_1_raft_tests.log",
    "uncovered_tests": []
  },
  "kvraft::tests::": {
    "last_attempt": 1,
    "last_completed": 1,
    "last_result": "OPEN",
    "candidate_revision": "check-1-1793334999796098025",
    "log": "/home/nitro/Desktop/raft_verus/runs/.controller/calls-B-01/check-1-1793334999796098025/tests_seed_1_kvraft_tests.log",
    "uncovered_tests": [
      "kvraft::tests::basic_3a",
      "kvraft::tests::concurrent_3a",
      "kvraft::tests::many_partitions_many_clients_3a",
      "kvraft::tests::many_partitions_one_client_3a",
      "kvraft::tests::one_partition_3a",
      "kvraft::tests::persist_concurrent_3a",
      "kvraft::tests::persist_concurrent_unreliable_3a",
      "kvraft::tests::persist_one_client_3a",
      "kvraft::tests::persist_partition_3a",
      "kvraft::tests::persist_partition_unreliable_3a",
      "kvraft::tests::snapshot_recover_3b",
      "kvraft::tests::snapshot_recover_many_clients_3b",
      "kvraft::tests::snapshot_rpc_3b",
      "kvraft::tests::snapshot_size_3b",
      "kvraft::tests::snapshot_unreliable_3b",
      "kvraft::tests::snapshot_unreliable_recover_3b",
      "kvraft::tests::snapshot_unreliable_recover_concurrent_partition_3b",
      "kvraft::tests::unreliable_3a",
      "kvraft::tests::unreliable_one_key_3a"
    ]
  },
  "test_support::": {
    "last_attempt": 2,
    "last_completed": 2,
    "last_result": "PASS",
    "candidate_revision": "check-2-1793335555416045549",
    "log": "/home/nitro/Desktop/raft_verus/runs/.controller/calls-B-01/check-2-1793335555416045549/tests_seed_1_test_support.log",
    "uncovered_tests": []
  },
  "shard_ctrler::tests::": {
    "last_attempt": 2,
    "last_completed": 2,
    "last_result": "OPEN",
    "candidate_revision": "check-2-1793335555416045549",
    "log": "/home/nitro/Desktop/raft_verus/runs/.controller/calls-B-01/check-2-1793335555416045549/tests_seed_1_shard_ctrler_tests.log",
    "uncovered_tests": [
      "shard_ctrler::tests::basic_4a",
      "shard_ctrler::tests::multi_4a"
    ]
  }
}

Newly passing checked units, with other gaps retained: functional:raft::tests::

Previous round 1: COMPLETED; independent result INCOMPLETE.

Candidate-reported approach/dependencies (UNVERIFIED):
on.rs` is absent. All other lab families remain untested and unfinished.

Failed approaches and repairs: the first ballot proof omitted the loop's unchanged-state invariant and a sequence-to-set lemma; the first call-composition proof exposed insufficiently precise vote/term contracts. These were strengthened and checked without assumptions. A panic-based incomplete-recovery test failed because MadSim treats task panic as a simulation failure; a fallible internal constructor now exposes the actual recovery error before any handler starts. One intermediate check reported candidate drift while contracts were edited; final 2A seed runs and the final added-test run completed without source edits.

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/raft.rs::new, madraft/src/raft/raft.rs::try_new, madraft/src/raft/state.rs::fresh | madraft/proof/progress.rs | Local fresh contract checked; async constructor is not imported or verified; establish deployment identity and runtime initial relation. |
| transitions | madraft/src/raft/raft.rs::start_ticker, madraft/src/raft/raft.rs::request_vote, madraft/src/raft/raft.rs::send_vote_request, madraft/src/raft/state.rs::vote, madraft/src/raft/state.rs::elected | madraft/proof/progress.rs | Local core and tally checked; genuine reply authorization and all async callers remain unproved; replication and commit are absent. |
| outputs | madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_heartbeats, madraft/src/raft/raft.rs::request_vote | madraft/proof/progress.rs | No actual send/storage-completion history relation; persistence-before-reply composition remains unproved. |
| recovery | madraft/src/raft/raft.rs::restore, madraft/src/raft/state.rs::restore_successor | madraft/proof/progress.rs | Local record validation and recovered vote call checked; durable provenance, decode/history relation and subsequent async caller composition remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/progress.rs | Both runtime snapshot operations remain placeholders; no snapshot decoder, observation or fidelity proof. |
<!-- /connections -->

Next required increment: implement actual 2B proposal/AppendEntries/conflict repair and current-term majority commitment, replacing the empty-log freshness comparison and heartbeat-only RPC with real log metadata; keep authorization in the shared checked core and extend persistence with exact command bytes. In parallel with that functionality, define the execution/observation adapter rather than claiming local lemmas establish history-wide election safety. Next real check: `python3 scripts/check.py test raft::tests::basic_agree_2b --seed 1`, followed by `python3 scripts/check.py proof madraft/proof/progress.rs`. Conditional liveness and application correctness beyond the fixed properties remain OPEN.


Previous round 2: COMPLETED; independent result INCOMPLETE.

Candidate-reported approach/dependencies (UNVERIFIED):
ll targets and all five connection areas OPEN (`runs/checks/1793335372069019703`). No other lab family was rechecked this round.

Failed approaches: initial full suite passed 18/23, exposing snapshot placeholders; implementing snapshots initially produced a Rust borrow error, repaired by calculating the next boundary before mutably filling the vector. At 100 ms replication polling, seed 5 exceeded the retained 120 simulated-second limit in unreliable snapshot installation (22/23 tests passed); changing polling to 50 ms repaired this without changing tests or limits, and all five full suites passed afterward. Round 1's exact vote contracts, ballot loop invariants and incomplete-record rejection remain intact.

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/raft.rs::new, madraft/src/raft/raft.rs::try_new, madraft/src/raft/state.rs::fresh | madraft/proof/progress.rs | Local fresh checked; actual async constructor, group identity and runtime initial relation remain unproved. |
| transitions | madraft/src/raft/raft.rs::start, madraft/src/raft/raft.rs::append_entries, madraft/src/raft/raft.rs::advance_commit, madraft/src/raft/state.rs::commit_from_matches | madraft/proof/progress.rs | Checked real-vector count and cursor authorization; actual log mutation, peer acknowledgment provenance and async caller composition remain OPEN. |
| outputs | madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::send_heartbeats, madraft/src/raft/raft.rs::apply | madraft/proof/progress.rs | Actual message/storage/application output histories and release order remain unproved. |
| recovery | madraft/src/raft/raft.rs::restore, madraft/src/raft/state.rs::restore_successor | madraft/proof/progress.rs | Actual decoding, durable-history relation, restored log invariants and subsequent async call composition remain OPEN. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::install_snapshot, madraft/src/raft/raft.rs::cond_install_snapshot, madraft/src/raft/state.rs::can_snapshot | madraft/proof/progress.rs | Runtime implemented/tested; decoder fidelity, committed-prefix observation, bounded persistence and full caller proof remain OPEN. |
<!-- /connections -->

Next concrete work: reclaim obsolete durable images safely and make the `state`/`snapshot` paths reflect current bounded storage without trusting multi-file atomicity, then implement replicated KV operations/deduplication and real snapshot decoding. Also move actual log transitions and reply authorization into the checked core before attempting aggregate history claims. Next real check: `python3 scripts/check.py test kvraft::tests::basic_3a --seed 1`; after storage changes repeat `python3 scripts/check.py test raft::tests:: --seed 5` and `python3 scripts/check.py proof madraft/proof/progress.rs`. Conditional liveness, application correctness and all fixed history goals remain OPEN.


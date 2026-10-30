# Progress

Round 4 establishes an actual handler-to-history connection. The real Raft object now stores `execution::Runtime`, which privately owns the checked Machine and a ghost execution journal. Runtime::new and Runtime::recover establish valid initial histories; Runtime::begin_vote and Runtime::release_vote append their actual inputs, state boundaries and outputs, including rejected submissions and absent replies, and prove validity is preserved. These methods are called by the real RPC path, not just a test wrapper. The parent derives every-prefix validity and traces every released reply to an earlier recorded request in that same incarnation. Recovery is explicitly a new incarnation tagged with the actual decoded image; cross-crash history preservation is still OPEN.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The actual RPC caller carries an immutable request/operation context across storage and checks the returned receipt against that context before releasing a reply, preserving the checked handler history.
goal = execution_link
kind = step
functions = madraft/src/raft/execution.rs::begin_rpc, madraft/src/raft/execution.rs::finish_rpc, madraft/src/raft/execution.rs::begin_vote, madraft/src/raft/execution.rs::release_vote, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/implementation.rs
parent = madraft/proof/execution_connection.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/durable_vote_reply.rs
premises = This checked connection covers actual per-incarnation handler histories. Async dispatch and filesystem-buffer/handle correspondence, genuine network sends, group identity and cross-crash cluster histories remain open. implementation.rs is a partial entry and does not export the final cluster adapter/theorem interface.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/execution.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/implementation.rs | Actual Runtime constructor establishes its journal; async ownership and stable group identity remain open. |
| transitions | madraft/src/raft/execution.rs::begin_vote, madraft/src/raft/execution.rs::release_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Actual methods append exact transitions and preserve history validity; genuine network provenance and async caller remain open. |
| outputs | madraft/src/raft/execution.rs::release_vote, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote | madraft/proof/execution_connection.rs | Every recorded released reply has an earlier real handler request; storage effect/dispatch and actual RPC sends remain open. |
| recovery | madraft/src/raft/execution.rs::recover, madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/implementation.rs | Exact recovered image, follower role and empty pending context establish a new journal; linkage to prior incarnations remains open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 3 independent replay retained local/parent proof passes and three vote regressions; full Raft and sharded-KV families still fail on missing features. Round 4's first attempt built and passed vote regressions but exposed a ghost-pattern ownership compilation error and missing parent entry. Borrowing the operation fixed ownership; constructing the parent then exposed a real missing recovery postcondition (follower role). Machine::recover now exports that established fact, plus empty pending context and serial zero required for exact recovered initialization. The final obligation check passed build, three vote regressions, both dependencies, the actual implementation connection entry and its request-origin parent (runs/checks/1793346780193238882/results.json). initial_election_2a still fails because no leader is elected. Combined progress proof passed after these changes (runs/checks/1793346864059357151/results.json).

Failed approaches retained: truncate/write cannot provide available recovery across its gap, so empty/undecodable images stop recovery; recoverable generations remain future work. Cross-module reveal of a closed spec was replaced by an owner-exported theorem in round 2. Derived executable enum equality was replaced by pattern matching in round 3. A concurrent notes edit invalidated one check invocation and was repaired by a stable rerun. Round 4 initially moved an owned operation in ghost pattern matching, then lacked follower-role information at recovery; both were repaired from actual diagnostics, with no weakened history predicate, assumptions, skipped consensus bodies or rollback.

Scope and next dependency: the new implementation.rs is a real executable connection entry, not the final all-cluster interface. Its ghost journal preserves all observed calls within an incarnation and every recorded reply now has a prior recorded request; it does not yet record actual network envelopes or every storage callback, and restored histories are not joined to their predecessors. Next connect those callbacks and genuine sends into a cluster history, retaining disk states and pre-crash observations. Checked election initiation/collection is still missing. All five fixed global safety targets, full labs, snapshot fidelity, conditional liveness and application correctness remain OPEN.

Round 5: repairing the concrete State/Input interface diagnostics by adding explicit cluster adapter data types without inventing global step definitions. The real request wrapper now obtains and carries a private ReplyContext from checked begin_rpc through await, then calls checked finish_rpc to reject mismatched storage completions. First check pending.

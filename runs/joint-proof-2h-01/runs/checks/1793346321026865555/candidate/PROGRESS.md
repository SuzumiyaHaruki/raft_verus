# Progress

Round 3 retains the checked constructor, voting core, private Machine, operation-history and recovery results. The actual persist loop now takes ownership of WriteVote and dispatches checked VoteWrite actions: Create, Write(exact image), Sync, Release or Stop. Each callback is made in the corresponding successful real filesystem branch; failure stops the operation and releases no reply. Only the Ready phase produces a private WrittenVote receipt. Its checked release_reply method calls Machine::complete_vote and preserves the exact image/operation/request linkage. The shared code is imported by both Cargo and the nominated proofs; async dispatch and host-effect adequacy remain OPEN rather than being treated as a consensus TCB exemption.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The real RPC persistence driver obtains create/write/sync actions from checked code and releases the pending reply only through the completed operation, with an explicit filesystem-effect premise.
goal = execution_link
kind = caller
functions = madraft/src/raft/storage.rs::new, madraft/src/raft/storage.rs::action, madraft/src/raft/storage.rs::complete, madraft/src/raft/storage.rs::release, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/storage_correspondence.rs
parent = madraft/proof/durable_vote_reply.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/vote_operation_history.rs
premises = Actual async dispatch must establish the specified submitted filesystem action and returned result, including exact buffer and handle; serde and pinned filesystem effects are explicit host contracts. Network provenance, group identity, arbitrary crash/restart histories and cluster induction remain open. A receipt alone is not durable evidence.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/protocol.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/vote_operation.rs | Checked core constructor; async initialization, restore and stable group identity remain open. |
| transitions | madraft/src/raft/protocol.rs::begin_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_operation_history.rs | Exact request/submission transition checked; actual delivery and async wrapper preconditions remain open. |
| outputs | madraft/src/raft/storage.rs::action, madraft/src/raft/storage.rs::complete, madraft/src/raft/storage.rs::release, madraft/src/raft/storage.rs::release_reply, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote | madraft/proof/durable_vote_reply.rs | Checked sequencing, receipt and reply composition; actual async filesystem dispatch and RPC send provenance remain open. |
| recovery | madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/durable_vote_reply.rs | Recover now establishes exact saved-image equality; same-term conflicting request rejection follows conditionally. Durable history and decoder caller remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 2 independent replay preserved local/parent passes and both vote tests; Raft and shard-controller families still failed on unfinished features. Round 3 first attempt built and passed vote tests but hit unsupported derived enum equality inside verified code and a missing parent entry. Pattern matching repaired the equality interface, and the parent was constructed. The second and third obligation attempts passed build, both prerequisite units, storage correspondence and durable-vote parent; the final attempt also passed all three vote_tests, including early/out-of-order/failure receipt rejection and the actual RPC persistence/recovery regression (runs/checks/1793346139572676157/results.json). initial_election_2a still fails with no leader. Combined progress proof is being checked separately.

Failed approaches retained: truncate/write cannot provide available recovery across its gap; the current path stops on empty/undecodable images rather than forgetting votes. Recoverable generations remain future work. Round 2's cross-module reveal of a closed spec was replaced by an owner-exported theorem. Round 3's executable enum equality was replaced with pattern matching; no assumption/specification bypass was introduced. No rollback or earlier candidate was used.

Parent scope: filesystem(action,result,before,after) names a submitted Create/Write/Sync action and its actual successful result under the pinned in-memory simulator contract. The relation records the create/write gap as Incomplete, allows arbitrary effects on error, and gives sync no extra durability power. A finite storage-history induction establishes the exact image at Ready; durable_reply composes that conclusion with the checked pending reply transition. Machine::recover establishes image equality, and restored_vote_conflict uses that postcondition to reject a different same-term candidate. These conditional results do not prove the real async loop supplies the required host transitions.

Next dependency: connect the real persist loop's file handle, action buffer and returned result to the filesystem relation, including crash/cancellation between host operations; connect the released RPC reply to the actual delivered request and prior sends. Then extend the same execution interface with checked election initiation/collection. No full implementation.rs adapter or nonempty cluster witness exists yet. All five fixed targets, full labs, snapshot fidelity, conditional liveness and application correctness remain OPEN.

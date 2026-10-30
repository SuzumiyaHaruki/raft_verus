# Progress

Round 2 preserves the checked State constructor, RequestVote contracts, local vote-history exclusion and recovery lemma. The real Raft runtime now owns a private `Machine`: `new` calls State::new; `begin_vote` invokes State::request_vote, retains its exact request/reply and issues an immutable owned WriteVote; `complete_vote` validates operation number and all image fields before releasing the retained reply. The async wrapper borrows that operation while serializing/writing it, returns no reply on error, and passes the same owned operation to completion only after successful I/O. Checked Machine::recover now performs version and deployment member validation before exposing recovered state. These are actual runtime calls, but their full async composition remains OPEN.

<!-- obligation -->
```ini
[obligation]
id = fresh-state-vote-correspondence
statement = The real RPC persistence driver obtains create/write/sync actions from checked code and releases the pending reply only through the completed operation, with an explicit filesystem-effect premise.
goal = execution_link
kind = caller
functions = madraft/src/raft/storage.rs::new, madraft/src/raft/storage.rs::action, madraft/src/raft/storage.rs::complete, madraft/src/raft/storage.rs::release, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote
entry = madraft/proof/storage_correspondence.rs
parent = madraft/proof/durable_vote_reply.rs
dependencies = madraft/proof/initialization.rs, madraft/proof/vote_operation_history.rs
premises = Async constructor and filesystem completion composition, actual network provenance, deployment group identity, crash histories and cluster induction remain open. A matching operation token alone is not durable evidence. Local operation histories cover only the implemented vote segment.
tests = raft::raft::vote_tests, raft::tests::initial_election_2a
```
<!-- /obligation -->

<!-- connections -->
| Behavior | Actual functions | Proof entry | Open premises / next step |
| --- | --- | --- | --- |
| initialization | madraft/src/raft/protocol.rs::new, madraft/src/raft/raft.rs::new | madraft/proof/vote_operation.rs | Checked core constructor; async initialization, restore and stable group identity remain open. |
| transitions | madraft/src/raft/protocol.rs::begin_vote, madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_operation_history.rs | Exact request/submission transition checked; actual delivery and async wrapper preconditions remain open. |
| outputs | madraft/src/raft/protocol.rs::complete_vote, madraft/src/raft/raft.rs::persist, madraft/src/raft/raft.rs::request_vote | madraft/proof/vote_operation_history.rs | Exact pending reply/image matching checked; real write completion and RPC send provenance remain open. |
| recovery | madraft/src/raft/protocol.rs::recover, madraft/src/raft/raft.rs::restore | madraft/proof/vote_operation.rs | Core validates saved version/identity/range; durable-history correspondence and async decoder caller remain open. |
| snapshots | madraft/src/raft/raft.rs::snapshot, madraft/src/raft/raft.rs::cond_install_snapshot | madraft/proof/snapshots.rs | All snapshot paths, compaction and application decoding remain open. |
<!-- /connections -->

Checks: round 1 independent replay retained passing local initialization, vote/recovery proofs and RPC storage test; the full Raft and KV families failed because features are unfinished. Round 2 first attempt built and passed the RPC test but encountered a proof visibility compilation error and a missing parent file. Moving the matching-operation lemma into its owning module repaired visibility without exposing private state. The second obligation check passed the initialization dependency, phase-machine entry, transition parent, build and RPC test (runs/checks/1793345530295356757/results.json); initial_election_2a still fails with no leader. The final local attempt passed build, both vote_tests (real RPC persistence/recovery and pending/mismatched completion), the initialization dependency, the phase-machine entry and the finite operation-history parent (runs/checks/1793345656252436479/results.json). initial_election_2a continues to fail. Combined progress.rs passed separately (runs/checks/1793345703777213809/results.json). These are local/parent results, not global theorem acceptance.

Failed approaches retained: the inherited truncate/write storage skeleton cannot provide available recovery across its gap; the current vote-only path stops on undecodable/truncated images instead of forgetting votes. No recoverable generation design is claimed. The first round-2 proof attempted to reveal a closed specification from another module; a public theorem inside the owner replaced that invalid proof interface. No rollback, imported candidate, assumption, skipped consensus body or fabricated completion proof was used.

Next dependency: connect each actual delivered request and submitted operation through the async filesystem result to the released RPC response, preserving prior sends and durable history across crashes. The local completion relation admits matching completions without claiming they are durable; it is not a full runtime/environment relation. Then implement checked election initiation and vote collection. No global implementation.rs adapter or nonempty cluster witness exists yet. All five fixed safety targets, full labs, snapshot fidelity, conditional liveness and application correctness remain OPEN.

Round 3 in progress: the real persist loop now dispatches checked Create/Write/Sync/Release actions. A private WrittenVote receipt can only be obtained after the checked sequence, and the RPC path passes its same operation to Machine::complete_vote. The proof models actual submitted filesystem actions and their disclosed effects, including incomplete images and arbitrary error effects, rather than treating matching tokens as evidence of disk contents. Async dispatch/host-effect adequacy remains OPEN.

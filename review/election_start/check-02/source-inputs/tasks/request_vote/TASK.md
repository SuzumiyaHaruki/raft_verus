# RequestVote receiver — DRAFT

The eventual research objective is to generate complete MadRaft and prove its five safety properties. This task generates only the real receiver and local proofs, not candidate tallying, majorities or leader promotion. A node not changing its vote within a term is not cluster Election Safety.

## Fixed behavior

The rules follow [Raft Figure 2 and section 5.4.1](https://raft.github.io/raft.pdf). Reject lower-term requests and return the current term. For a higher term, update the term, clear the old vote and become Follower; persist this change even when rejecting an outdated log. Within the same term, grant only when no vote exists or the candidate matches the existing vote; repeated grants to that candidate are allowed. Compare last log term before last log index. A valid idle request with an available operation ID must produce the prescribed decision, and matching successful persistence must produce its reply. Always rejecting or never returning is not acceptable.

`api.rs` fixes observations and pure specifications, not the executable algorithm. `Core::receive` decides state, the record to save and the pending reply; `Core::complete` checks operation identity and the reported result before authorizing output. The actual MadRaft handler connects these functions. You choose the representation and proof; the independent caller reasserts the fixed relations instead of accepting self-reported coverage.

Every accepted valid request produces Save, including unchanged denials. This increases I/O but makes each state-dependent reply wait for its corresponding successful save. Save is not a reply. Busy preserves Pending; mismatched or duplicate completions cannot release replies. Matching failure halts the receiver without a protocol reply. Operation IDs increase without reuse in `[0,u64::MAX)`; recovering MAX-1 sets next=MAX and rejects further requests. Raft term and log index may equal MAX.

Membership is fixed to `[0,members)` and must include this node in a nonempty configuration. Requests with term=0, an invalid candidate, exactly one zero log term/index, or lastLogTerm greater than request term are Invalid. Persistent state also validates this log shape, log term not exceeding current term, and any vote naming a member in a nonzero term. Rejection priority is Halted, Busy, Invalid, Exhausted; rejection preserves state. Byzantine forgery and member authentication are outside this task: requester identity matching candidate_id is a disclosed non-Byzantine host premise, and replies return to the original RPC caller. Invalid file length/tags are I/O errors and must not silently initialize a fresh node.

Legal nonempty log summaries may come from explicit initialization/retained-record fixtures. Do not assume logs are always empty. Actual replicated-log-to-summary correspondence remains OPEN; AppendEntries is not implemented here.

## Four independent proof groups

1. `rules`: actual new/recover/receive/complete/inspect states and outputs satisfy fixed relations, with no request-validity precondition. The nonempty caller constructs a real core and obtains an affirmative reply.
2. `history`: the fixed caller starts with actual construction and executes arbitrary finite Receive/Complete sequences, observing every actual reply. Affirmative replies with the same node and term must name the same candidate. The fixed caller checks the replies observation; you supply the inductive invariant.
3. `confirmed`: actual receive emits the record, successful complete emits an affirmative reply delivered by the host, and no later operation is accepted. Recover the same record and execute any finite suffix; combined affirmative outputs remain unique.
4. `pending`: receive emits the record, the host finishes writing, but complete/reply have not occurred and no later operation is accepted. Recover that record and execute any finite suffix without inventing an old completion or reply. Recovery establishes an empty new observation, Follower, retained term/vote/log, next=op+1 and no Pending. Same-term B cannot receive a grant; A may retry.

Readback equality to the actual Write is a public faithful-I/O/file-retention/decoding premise. Verus checks actual recovery and later calls under that premise; a success boolean does not prove external I/O. The prefix state belongs to the same actual Core and its invariant; history separately connects construction to arbitrary finite prefixes.

## Host and acceptance

Preparation adds unfinished core/lemma entries to the original RaftHandle and registered RequestVote RPC. Other student TODOs, original tests and full targets remain. `host.rs` and thin handler wiring are unverified: they serialize operations, perform actual MadSim file I/O, report actual results, encode records and forward authorized replies unchanged. Test barriers control timing/observation only; failure injection writes through an actual read-only file. Fixed tests use real RPC, independent file-byte assertions, same-node kill/restart and fresh Core/endpoints.

`safety` runs only local Verus. `feedback`/`obligation` reserve time separately for functional and proof checks. `final` checks all local tests, all configured seeds and four proof entries against one stopped candidate. Only all required checks passing yields REQUEST_VOTE_CHECKS_PASSED; all five global safety properties remain OPEN, full MadRaft INCOMPLETE and conditional liveness OPEN. Tests, a local verified count or agent exit=0 alone cannot finish the task successfully.

Interrupted create/write, failed-write recovery, arbitrary crashes, other term/vote-changing paths, complete log correspondence and the async host remain unproved. These are local scope limits, not changes to the main environment. Do not expand indefinitely to eliminate disclosed host premises.

## Inputs

Start from the original root student scaffold and fixed requirements, with no old runs. The four calibration labs informed the same-source core, Pending/completion and bounded confirmed/unconfirmed recovery method; their evidence, sessions and any Raft solution are not inputs. Generate implementation and proofs within this experiment.

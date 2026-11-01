# Receive RequestVote and start a campaign — DRAFT

Regenerate the receiver and add active campaign start on the student MadRaft scaffold. One Core owns term, role, votedFor, Pending, operation IDs and recovery; the original State is not a second authority. The real `RaftHandle::start_election` can later be called by a timer. Tests trigger it without supplying the algorithm. Full targets and fault requirements remain unchanged. This task does not complete 2A, count votes, promote a leader, send heartbeats or replicate logs.

## Fixed receiver behavior

Preserve rejection priority: Halted, Busy, Invalid request, Exhausted operation IDs. Invalid means term=0, candidate outside membership, exactly one zero log index/term, or lastLogTerm greater than request term. Compare lastLogTerm before lastLogIndex. Reject older terms and another same-term candidate; allow the previously chosen candidate to retry. A higher term clears the old vote and becomes Follower even if its stale log prevents a grant. Every accepted decision is saved, including unchanged denials; only matching successful completion returns a reply. Keep the original eight test cases: rpc_tests.rs mechanically adapts Action types and additionally observes old-task cancellation before restart.

## Fixed campaign and authorization behavior

Campaign rejection priority is Halted, Busy, Leader, term=MAX, next=MAX. Rejection changes neither state nor outputs. Otherwise a valid Follower or Candidate must enter exactly old term+1, become Candidate, choose this node, preserve its actual log summary and allocate the shared next save ID. No caller supplies the next term. IDs are in `[0,MAX)` without reuse; term/index may equal MAX. The last valid increment and ID must still work. A one-member configuration also remains Candidate.

`start_election` returns Campaign::Save(Write). Its Action::Send is prepared context, not network permission. Only an actual `complete` return of Some(Action::Send) authorizes sending. Send contains the member count, this node's identity and this round's immutable Request term/candidate/log summary. Destinations are all other members. The host maps IDs to configured addresses and forwards the payload without reading a later term or log. Delayed delivery preserves the request's round; a network error does not imply no message was sent.

Receiver and campaign operations share Pending and the save sequence. Matching failure clears Pending and halts without reply/send. A wrong ID preserves Pending; a duplicate completion produces no action; Reply and Send cannot be confused. Candidate Core owns all protocol decisions. The host cannot choose identities, terms, vote eligibility or authorization. RPC results are observations only: reply provenance, higher reply terms, tally deduplication, majority reasoning and leader promotion remain OPEN.

## Fixed history and recovery targets

The independent caller directly imports the same vote_core.rs as Cargo. Starting from real construction, its loop executes arbitrary finite Start/Receive/Complete sequences. It observes the actual inspect state after an accepted campaign as Choice(node,term,votedFor), and every actual affirmative reply as Choice(node,term,candidate). Any two choices for the same node and term must name the same candidate. Do not omit self-choices, choose an empty observation, restrict to already-correct histories or replace actual functions with model projections. Self-choice is observable before I/O completion: it is neither I/O evidence nor a vote eligible for majority counting. You must establish and preserve your own invariant and implementation lemmas.

Terms never decrease; accepted campaigns increase them by exactly one. Actual complete returns depend on prior Pending and the matching successful notice. There is no outbound return from campaign start. Fixed nonempty callers require both an eligible affirmative reply and an eligible campaign followed by the correct Send; always refusing or never sending is not a solution.

Keep both receiver recovery windows and extend them to self-votes. The record was actually written successfully, then either complete authorized the corresponding reply/send, or complete has not occurred; no later operation was accepted. Actual recover must read the retained Record and establish Follower, retained term/vote/log, next=op+1, empty Pending and empty new-segment observations. It must not resend an old request/reply. Old completion notices cannot consume new Pending. The fixed combination executes a real mixed prefix, the window operation, recover and arbitrary mixed suffix, then checks uniqueness of combined choices. The uncompleted window does not invent an old complete.

Faithful readback equal to the actual emitted Record is a public host premise, expressed by the fixed caller's record comparison. The comparison is not a filter on voting correctness. Recovery correctness and uniqueness are conclusions, not host premises. Arbitrary interrupted writes, recovery after failure, induction across arbitrarily many crashes, real-disk power-loss durability and actual replicated-log-to-summary correspondence remain OPEN.

## Actual path and checks

The path is RaftHandle construction/registered RequestVote → the same Core.receive or Core.start_election → host.save → Core.complete → reply to the requester or real RequestVote RPCs to other members. term, is_leader and the existing start role check also read the Core. Preparation changes only a new copy. The host serializes each save/completion and releases its lock before sending an immutable authorized payload.

Fixed tests cover the eight receiver cases, mixed interactions, boundaries/completion, at least one actual candidate and two actual receivers with a nonempty network exchange, a pre-write barrier, real write failure, both self-vote restart windows and no single-member promotion. Restart waits for cancellation and a Weak observation that the old Core was released, then constructs a fresh endpoint/core on the same node. Legal nonempty log fixtures do not establish replication. A finite interval without an RPC is test evidence, not a general async proof.

Checks separately report rules, campaign, authorization, history, receive_confirmed, receive_pending, self_confirmed and self_pending. Final acceptance runs every required local test, all configured seeds and all proofs on the same stopped source. Formal acceptance uses --no-cheating and verifies actual candidate bodies, all lemmas and the fixed caller. --verify-root and --no-verify are allowed only in explicitly separate maintenance conditional/type checks. A helper count, tests or agent exit=0 alone do not complete the assignment.

## Disclosed unverified boundary

Host serialization, actual writes/retention, encoding/decoding, truthful completion, membership/address mapping, original-requester/recipient forwarding, cancellation and async wiring are not verified by Verus; fixed integration tests constrain them. Completed MadSim writes surviving restart do not establish real-disk durability. A Success flag cannot prove I/O. The toolchain and ghost erasure remain trusted.

ELECTION_START_CHECKS_PASSED denotes local machine checks pending semantic review. Election Safety, Leader Append-Only, Log Matching, Leader Completeness, State Machine Safety, complete student functionality and conditional liveness remain OPEN.

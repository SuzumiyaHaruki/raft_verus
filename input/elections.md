# Leader election

Status: DRAFT. Module: INCOMPLETE. RequestVote and durable replies are generated through MADRAFT-VOTE; MADRAFT-ELECTION extends that run's code to the complete election lifecycle. No election behavior is supplied.

## Responsibility and connections

Implement the complete follower/candidate/leader election lifecycle: election timeouts, term increments, durable self-voting, RequestVote requests and responses, unique-voter quorum counting, leader transition and step-down on higher terms. Handlers operate repeatedly on arbitrary inputs satisfying their stated contract, including duplicates and stale traffic across many terms. A request handler is one part of this module, not a complete election implementation.

Use shared [state](state.md), the [runtime coordinator](runtime.md) and existing Message/Event/Effect types. Leader transition initializes [replication](replication.md) state and follows the existing model's Noop/durability behavior. Read `election_timeout`, `send_vote`, `higher_term`, `receive_vote_request`, `receive_vote_response` and `become_leader` in `crates/raft-spec/src/lib.rs`. Membership, timer conditions and persistence follow [requirements.md](requirements.md), [assumptions.md](assumptions.md) and [io.md](io.md); unresolved choices remain DRAFT.

## Module completion

All election paths are integrated into the node, including one-node membership, competing candidates, duplicate votes, stale RPCs, higher terms and crashes around vote persistence. Unit tests and multi-term cluster scenarios use that node. Prove local handler correspondence, term/vote invariants and quorum intersection, then the historical per-term election-safety argument with cluster composition. PO-AUX-TERM, PO-AUX-VOTE and PO-S-ELECTION remain open. Safety may not depend on liveness assumptions. Generate election progress arguments together with these paths as specified in [progress](progress.md); eventual election is a required project result.

## Step: Start and retry elections

Stage: PLANNED. Requires node state and the persistence coordinator.

Implement election timer handling, checked term advancement, candidate transition, self-vote persistence and RequestVote generation to fixed peers. Tie timers and RPCs to incarnation/generation/term so obsolete callbacks cannot restart the wrong attempt. A self-vote must be durable before sending the dependent messages. Reset/rearm timers according to the existing draft contract; propose unresolved timing behavior for review rather than assuming eventual election.

Deliver timeout/request-generation code, tests for retries, stale timers, one-node membership and resource limits, plus local correspondence and term/vote preservation proofs. Configure real independent build/tests/proofs before enabling this step. Those profiles and the prerequisite state/coordinator are absent.

## Step: Process vote requests

Stage: PLANNED. Requires node state and the persistence coordinator.

Implement a reusable RequestVote handler. Reject older terms, process higher-term step-down and durable term/vote changes, compare last-log term/index, and grant at most one eligible candidate per term. A repeated request for the recorded candidate must still satisfy the eligibility rule. Bind the response to the request and release a positive vote only after the required durable completion. Reject unknown peers and handle uncertain persistence without false success.

Deliver source, tests for old/new terms, stale/fresh logs, duplicate requests and crashes around replies, and a local correspondence proof. This alternative planned step is not active; the default MADRAFT-VOTE assignment generates this behavior against the existing independent vote profiles. This step does not claim election-module completion or the global vote invariant.

## Step: Count votes and transition roles

Stage: PLANNED. Requires timeout and vote-request paths.

Handle VoteResponse using the current term, outstanding RPC and fixed membership. Count each eligible voter once, become leader only at a strict majority, and initialize the model's leader replication/Noop state. Apply higher-term step-down consistently and invalidate obsolete pending election/leader work. Exercise several successive elections rather than one hard-coded successful exchange.

Deliver response/role-change code, quorum and stale-message tests, multi-term election traces and local proofs feeding historical election safety. Independent profiles must include all preceding election regressions. The actual cluster driver is supplied, but the complete handlers remain missing. Use the active MADRAFT-ELECTION step below. Finish the module only after its integration, invariants and stated module checks are established.

## Step: Generate elections on the supplied foundation

Stage: READY as MADRAFT-ELECTION after MADRAFT-VOTE in the same candidate. Extend handle_event, shared node state and generated local proof code. Implement current-generation election timers, checked term increases, durable self-votes, RequestVote emission, current-term/RPC vote response matching, unique-member counting, leader transition and higher-term step-down. Retain this run's generated vote path and the independent durable-reply constraints.

Use actual MadSim nodes as candidates, followers and leaders. The supplied cluster-election profile checks one/three/five members, competing timeouts and leader replacement after a partition. Public boundary tests reject duplicate/stale votes and unsafe counter exhaustion. Timers may affect observed progress, but safety cannot assume fair delivery, synchrony or eventual election.

Generate the contracts and auxiliary term/vote/quorum invariants needed for this implementation. Explain which are established by initialization, preserved by each event and sufficient for historical election uniqueness. Do not call a duplicate-vote test or local barrier proof Election Safety. Historical vote provenance, quorum intersection and cluster composition must still be discharged before PO-S-ELECTION can close.

Run interface-check, core-vote-tests, core-election-tests, cluster-election and core-proof; repair earlier completed work when necessary. Keep generated proof gaps explicit and preserve attempt history. This step's checks establish only its stated local completion, not the five-property safety aggregate.

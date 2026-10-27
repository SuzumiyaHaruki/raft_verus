# Environmental and resource assumptions

Status: DRAFT. Source: original requirements sections 5-7. These premises do not replace protocol invariants or proofs.

## Applicability

Safety does not rely on A-L-* liveness assumptions. The mathematical natural-number/sequence layer needs no capacity premise; executable conclusions additionally need A-P-MACHINE and representation refinement. Structural `well_formed`, term monotonicity, deduplicated votes, persistence before voting, log freshness, truthful replication progress, current-term commitment and index/token bounds are proof targets, not environmental axioms.

Liveness claims are non-probabilistic conditional theorems. The complete environment is A-L-STABLE, A-L-NETWORK, A-L-OPPORTUNITY, A-L-TERM, A-L-HOST and A-L-IO below. Request progress additionally needs A-L-REQUEST; executable conclusions need A-P-CAPACITY/CLOCK/MACHINE. Premises concern faults, messages/processing, timing, the host and inputs; they do not assume eventual election or commitment.

| Targets | Environmental premises |
|---|---|
| All S-* | A-S-ID, A-S-STORAGE, A-S-COMPLETION |
| S-APPLY additionally | A-S-APP |
| All L-* | A-S-ID, A-S-STORAGE, A-S-COMPLETION and all six liveness premises listed above |
| L-COMMIT additionally | A-L-REQUEST |
| Executable conclusions | A-P-MACHINE; executable liveness also A-P-CAPACITY and A-P-CLOCK |

For L-ELECTION, Q eventually stops crashing, persistence and round trips are bounded, exclusive election windows occur, and external higher-term interference disappears. The last two conditions exceed weak fairness and do not follow automatically from random timeouts. For L-COMMIT, a local Accepted reply alone does not establish eligibility: retries preserve identity/content and receive fair service; instances accepted by old leaders may be lost. For L-DELIVER, replication, storage and ordered delivery receive service without permanent blocking of earlier positions. Permanently partitioned or crashed nodes are excluded. Application completion additionally needs A-S-APP and A-L-IO; library delivery safety remains meaningful with an unresponsive application.

Old-message replay is explicitly restricted for liveness: each actual Send has finitely many copies, each eventually delivered or dropped, so messages from a finite prefix eventually drain. Not every message must arrive promptly. Without this condition, a crashed leader's same-term heartbeat could be replayed forever and continually reset election timers even after higher-term interference stops. New messages from outside Q also cannot permanently prevent A-L-OPPORTUNITY windows. This condition is stronger than internal majority stability and still needs human review and formal encoding. Safety does not use these liveness restrictions.

Fixed-width counters are not inexhaustible. A-P-CAPACITY must specify a numerical budget for the execution prefix needed by a target request. Deriving B and checking executable entry preconditions remain PO-REPRESENTATION/PO-LIVE-REFINEMENT obligations. Resource exhaustion may permit safe stopping without an unconditional liveness conclusion.

## Safety environment

### A-S-ID

Fixed members have unique, non-Byzantine identities, with at most one active incarnation per identity. Every in-flight protocol message originates from a legal Send by its claimed sender; fields, votes and completions are not forged. Messages may be lost, duplicated, delayed or reordered.

Responsibility: Host network and deployment. Test limit: Negative-input, authentication and codec tests; global authenticity relies on the external contract.

### A-S-STORAGE

A successful persistence completion means the entire term/vote/log state is durable, atomic and ordered. Recovery reads the latest complete durable image for that identity, without rolling back acknowledged writes or inventing entries. A failed operation may have written data; recovery uses the actual durable state.

Responsibility: Host storage. Test limit: Torn-write and crash injection can test adapters; device semantics remain trusted.

### A-S-COMPLETION

The host reports truthful outcomes only for issued operations. A full token binds node, incarnation and sequence. Old notifications may be duplicated; the library must ignore mismatched notifications.

Responsibility: Host I/O. Test limit: Test wrong, duplicate, late and cross-recovery tokens; audit adapters for truthful outcomes.

### A-S-APP

The host applies only commands and positions actually delivered by the library. Durable application state and the applied prefix are updated atomically, or replay is idempotent by position. Acknowledgments never run ahead of application; recovery does not invent applied positions. Repeated commands are allowed. Business semantics and exactly-once external effects are outside the guarantee.

Responsibility: Host application. Test limit: Test duplicate delivery and crashes; application transaction semantics require separate proof or trust.

## Liveness environment

### A-L-STABLE

There exist a finite K and a fixed strict majority Q of the member set M such that, after K, every node in Q runs continuously without crashing and all relevant local events receive service. Nodes outside Q may keep failing, subject to A-L-TERM.

Responsibility: Host runtime environment. Test limit: Finite runs can expose violations but cannot prove permanent eventual stability.

### A-L-NETWORK

There exist a finite K and a positive integer Delta such that, after K, at least one copy of repeatedly sent protocol messages within Q is delivered and processed within Delta. Each actual Send creates only finitely many copies; each copy is delivered or dropped in finite time, so old messages from any finite prefix eventually drain. Loss and finite delay remain allowed; not every message must arrive promptly. The periodic heartbeat interval plus Delta and the disk bound is strictly below the follower election-timeout lower bound. This send-scheduling condition does not assume a leader exists.

Responsibility: Host network and scheduler. Test limit: Controlled-clock and packet-loss tests cover finite windows, not permanent real-world bounds.

### A-L-OPPORTUNITY

After some finite K, whenever valid heartbeats are no longer continuously sent and promptly processed within Q, every continuously participating node in Q repeatedly gets an exclusive election opportunity: its valid timeout fires first, then no other Q node starts an election for H time units. H exceeds the bound for term/vote persistence and a RequestVote round trip. Messages from outside Q cannot indefinitely reset Q timers and prevent these opportunities. Together with A-L-TERM, this is an explicit interference restriction, not a consequence of majority stability. A-L-NETWORK bounds old-message duplication and draining. Every expired event eventually receives service; new valid heartbeats may reset timers, and no extra timeout is injected unconditionally.

Responsibility: Timer environment; the library handles resets and stale-event filtering. Test limit: Deterministic simulations can exhibit windows; random timeouts do not establish the condition for every trace.

### A-L-TERM

After some finite K, nodes outside Q deliver only finitely many messages with a term above the maximum known in Q. Those messages and old in-flight messages are processed or dropped in finite time; external messages then stop raising Q terms. External shutdown, isolation or eventual timing conditions for all nodes may establish this condition; majority stability alone does not.

Responsibility: Deployment, network and timer environment. Test limit: Counterexamples can be tested; permanent disappearance of interference is an environment condition.

### A-L-HOST

Continuously enabled protocol work, valid completions, replication, delivery and timer events receive weakly fair service. Each service has a finite processing bound and is not starved by unlimited new input. Termination of each library call must be proved by PO-LIVE-REFINEMENT, not assumed.

Responsibility: Host scheduler. Test limit: Test queue fairness and finite budgets; infinite fairness requires a model.

### A-L-IO

Persistence, transport-resource release and delivery backpressure in Q complete within finite bounds. Permanent disk failure and indefinite blocking are excluded from liveness conditions. Where application progress is required, the host eventually processes preceding delivered positions.

Responsibility: Host I/O and application. Test limit: Error and backpressure tests can reveal violations, not prove all future device responses.

### A-L-REQUEST

A valid request has a stable unique request_id and an unchanged command. Until a definite commit or delivery outcome is known, the client keeps retrying at participating Q nodes. The host serves the request fairly without starvation by unlimited new requests. Eligibility does not assume eventual acceptance or commitment.

Responsibility: Client and host scheduler. Test limit: Test fair queues and retries with the same identity; persistent retrying is an environment premise.

## Platform and resources

### A-P-MACHINE

The implementation executes under the recorded Rust/Verus compiler and target-architecture semantics, without arbitrary memory or machine-state corruption. Integer overflow must not silently wrap; checked conversions and safe errors remain implementation proof obligations.

Responsibility: Compiler, OS and hardware. Test limit: Test numeric boundaries and target builds; compiler and hardware correctness remain in the TCB.

### A-P-CAPACITY

Each concrete liveness target requires declared term/index/token headroom, log and queue capacity, and a finite input budget B. The implementation checks the budget and must prove it covers the execution prefix needed for that target. B has not yet been derived, so executable liveness remains OPEN. Finite counters are not assumed inexhaustible under infinite input or execution.

Responsibility: Configuration supplies capacity; the library establishes entry preconditions. Test limit: Test capacity and overflow; sufficient bounds require proof.

### A-P-CLOCK

Timing units used for liveness are monotonic, with declared finite drift and scheduling-delay bounds. Safety uses event order and does not depend on wall-clock accuracy. The library filters clock rollback and notifications from stale timer generations.

Responsibility: Host timer adapter. Test limit: Inject rollback and drift; physical timing bounds remain externally trusted.

# Pending decisions

Status: DRAFT. All ten recommendations await human review; none is approved. Approver and approval time: none.

The requested maintenance revision activates STATE-INIT as a draft generation experiment. Review its candidate API, independent targets and results after delivery; activation approves no Raft semantics. RUNTIME-PERSIST has supplied types, contracts, independent checks and failing seeds; executable behavior is for the experiment to generate. Real adapters and temporal proofs remain open.

| ID | Topic | Status |
|---|---|---|
| ADR-01 | Events/effects and coordinator boundary | DRAFT |
| ADR-02 | Membership and cluster size | DRAFT |
| ADR-03 | Numbers, tokens and resources | DRAFT |
| ADR-04 | Non-circular conditional liveness | DRAFT |
| ADR-05 | Proposals, request identity and retries | DRAFT |
| ADR-06 | Delivery, application and recovery | DRAFT |
| ADR-07 | Storage, network and TCB | DRAFT |
| ADR-08 | Verus, temporal library and entrypoints | DRAFT |
| ADR-09 | Isolation and local mode | DRAFT |
| ADR-10 | Evidence binding and review after delivery | DRAFT |

## ADR-01: Events/effects and coordinator boundary

Recommendation: Use a single-threaded step(Event) -> Effects API. Keep persistence dependencies, token lifetimes, error feedback and delivery ordering inside the library to be verified. I/O requests and completions are separate observable model steps.

Rationale: Crashes may occur between writes, sends and callbacks; the library must enforce their protocol ordering, beyond a pure handler proof.

Alternatives: Put coordination in the host, expanding external assumptions; or use blocking I/O, which still needs intermediate-crash semantics.

Impact: Determines PO-IO/PO-REFINE-STEP and host duties. Active source contains shared types and failing operation seeds; old maintenance implementations are retained only as excluded offline fixtures.

Definition: [related material](../input/io.md). This candidate may be revised; it is not an approved baseline.

## ADR-02: Membership and cluster size

Recommendation: Use parameterized nonempty finite membership and strict majorities. Support one node; start routine simulations with 1/3/5 nodes.

Rationale: Quorum intersection can be proved for arbitrary finite size; one node is a meaningful boundary that should not be silently excluded.

Alternatives: Explicitly restrict v1 to n>=3, or check one fixed n. Fixed-size checks cannot claim a parameterized-size proof.

Impact: Determines the set domain and quorum lemmas for PO-INIT/PO-AUX-VOTE; test counts do not change theorem quantifiers.

Definition: [related material](../input/requirements.md). This candidate may be revised; it is not an approved baseline.

## ADR-03: Numbers, tokens and resources

Recommendation: Use nat/Seq in the model and checked u64/finite containers in executable code. Report resource exhaustion and stop affected actions safely. Node-scoped tokens contain incarnation+sequence; incarnation increases durably across recovery.

Rationale: Prevent wraparound and cross-crash reuse of late completions. Safe stopping and liveness conditions must be separate.

Alternatives: Use arbitrary-precision integers, still subject to allocation limits; or explicitly bound the contract execution/input budget.

Impact: PO-REPRESENTATION/PO-AUX-TOKEN/PO-LIVE-REFINEMENT. Finite liveness budget B is not derived; unlimited resources are not promised.

Definition: [related material](../input/assumptions.md). This candidate may be revised; it is not an approved baseline.

## ADR-04: Non-circular conditional liveness

Recommendation: Use a non-probabilistic conditional theorem: an eventually stable majority Q, bounded communication/processing/persistence, sufficiently long exclusive election windows, valid heartbeat timer resets and fair service. External higher-term interference becomes finite and disappears. Each Send has finitely many copies that eventually arrive or drop; old messages from finite prefixes drain. Outside-Q messages cannot permanently prevent election opportunities. Not every message must arrive promptly; a stable leader is not assumed.

Rationale: Weak fairness or a stable majority alone cannot break election competition or stop repeated minority term increases. Even after higher-term traffic stops, endless replay of a crashed leader's same-term heartbeat can reset timers forever. Disclose both old-message draining and external timer-interference restrictions as environmental conditions.

Alternatives: Require eventual synchrony of all nodes, or develop an almost-sure probabilistic election proof with an explicit randomness/probability model. The latter is not a deterministic all-traces guarantee.

Impact: PO-L-ELECTION/COMMIT/DELIVER and fairness refinement. Environmental premises are not fully encoded as Verus trace predicates; D3 remains PARTIAL.

Definition: [related material](../input/assumptions.md). This candidate may be revised; it is not an approved baseline.

## ADR-05: Proposals, request identity and retries

Recommendation: Accepted means local receipt only. request_id=(client,sequence) stably binds command. Unconfirmed clients retry; eventually at least one corresponding entry commits. Separate Client entries from internal NoOp.

Rationale: An old leader's accepted proposal may be overwritten; retries may occupy several positions. Exactly-once is not implicit.

Alternatives: Add library request deduplication/result caching, expanding code/proof scope; or promise instance progress only under an already stable leader, weakening the top-level request claim.

Impact: PO-REQUEST/PO-L-COMMIT and host retry contract. Eligibility must not assume eventual acceptance or success.

Definition: [related material](../input/io.md). This candidate may be revised; it is not an approved baseline.

## ADR-06: Delivery, application and recovery

Recommendation: Guarantee ordered delivery of committed prefixes and permit crash replay. Persist application state/cursor in one host transaction, or make replay idempotent by position. Application and acknowledgment are separate steps.

Rationale: Delivery is not successful application; recovery must handle a completed transaction with a lost acknowledgment. Repeated requests at different positions do not provide exactly-once effects.

Alternatives: Add a library deduplication store/application engine, expanding scope; or claim delivery consistency without claiming completed S-APPLY.

Impact: Separate PO-DELIVERY/PO-S-INTERFACE from PO-S-APPLY. A-S-APP affects the latter only.

Definition: [related material](../input/io.md). This candidate may be revised; it is not an approved baseline.

## ADR-07: Storage, network and TCB

Recommendation: The library owns protocol state and I/O dependencies. Abstract storage provides atomic/ordered writes, truthful acknowledgments and recovery. Host contracts constrain network identity/codecs, devices and timers; register each external trust boundary.

Rationale: Actual adapters are absent. external_body must not disguise system assumptions as implementation proofs.

Alternatives: Verify file/network adapters too, or remove crash recovery from scope; the latter conflicts with the current eventual v1 target.

Impact: PO-RECOVERY/PO-IO/PO-COMPOSITION and A-S-ID/STORAGE/COMPLETION. New trusted locations need review.

Definition: [related material](../input/trust.md). This candidate may be revised; it is not an approved baseline.

## ADR-08: Verus, temporal library and entrypoints

Recommendation: Record actual local versions/APIs and prefer a compatible verus-tla integration. Until compatibility is established, keep it NOT_PINNED/BLOCKED. Separate positive/negative smoke, spec, interface-check and full profiles.

Rationale: Explicit quantified temporal targets are reviewable without a fabricated axiom library; syntax success does not prove Raft.

Alternatives: Start with transparent trace definitions and integrate the temporal library later; compatibility and proofs still need verification.

Impact: PO-L-*/PO-COVERAGE. User-authorized official Verus 0.2026.01.23.1650a05, bundled vstd, isolated Rust 1.92.0 and Z3 4.12.5 were actually exercised. Smoke results matched expectations and spec was TYPECHECKED only. This combination remains unapproved; verus-tla compatibility is BLOCKED. Version observations are not approved pins. The temporary installation was absent on 2026-10-27; the same release was restored to a persistent user directory. See [tool setup and actual records](../docs/tools.md).

Definition: [related material](../docs/sources.md). This candidate may be revised; it is not an approved baseline.

## ADR-09: Isolation and local mode

Recommendation: Bootstrap evidence is local-development evidence. Formal execution needs separate permissions, a container or host sandbox so tasks cannot modify approvals, checker, toolchain or controller evidence, or read secrets.

Rationale: Worktrees/directory copies share OS permissions; build scripts/tests execute host code. Digests cannot prevent same-UID checker modification.

Alternatives: Continue supervised candidate experiments on a trusted local host, explicitly without formal or hostile-agent-resistant acceptance claims.

Impact: TCB-HOST/CHECKER; formal full acceptance remains blocked by isolation gaps. Offline fixtures consume no real generation calls.

Definition: [related material](../docs/workflow.md). This candidate may be revised; it is not an approved baseline.

## ADR-10: Evidence binding and review after delivery

User clarification: Review delivered code with the assistant; do not require an approval file inside start/resume. This approves no Raft semantics or correctness.

Recommendation: Retain existing evidence bindings and separate candidates. Review results after delivery; do not regenerate signatures or add runtime checkpoints.

Rationale: A historical PASS can be reused after uncommitted source/checker changes. Git omits worktree/external tools; versions/keys do not identify content; database transactions/uniqueness do not cover files/processes; types/tests do not record exact inputs of past checks. Existing bindings associate evidence with those inputs.

Alternative: Rerun every check instead of reusing evidence. Same-user isolation and complete correctness remain open. See [the workflow](../docs/workflow.md).

## Review items introduced by this revision

All remain recommendations, not approvals:

| Existing decision | Concrete item to review | Recommendation and current limit |
|---|---|---|
| ADR-01 / ADR-03 | Reconciliation API and numeric/resource boundary | Use the shared `reconcile_log` after term/predecessor validation. Invalid indices and representable-length overflow return errors without mutation. Future callers must check `u64` to `usize` conversion and committed-prefix compatibility. `Vec` allocation can still abort; do not claim recoverable allocation failure or resource liveness. |
| ADR-07 / ADR-08 | Shared Cargo/Verus build and local proof target | Use the installed macro erasure with matching child Rust 1.92. Keep `merged` unchanged. A protected proof caller demands the original relation even if an editable implementation contract is weakened. Derived non-Copy Clone contracts are unavailable; the verified operation does not invoke Clone. |
| ADR-09 / ADR-10 | Revised local acceptance and resumable candidates | Review delivered task/check definitions separately from semantic claims. Typecheck/test/measurement completion need not close a theorem; real whole-crate function-attributed proof evidence can close only its local target. No hostile-process isolation or whole-project acceptance is claimed. |
| ADR-01 / ADR-02 / ADR-03 | Initial Node API | Recommend `Node::new(id, members)` for genuinely fresh storage only. Accept arbitrary ordering, reject duplicates/unknown identity/empty membership; preserve IDs and order. Incarnation zero is not a recovery allocator. Pending I/O fields are supplied; peer state remains a future extension to the same Node. No behavior is approved by this task. |
| ADR-08 / ADR-10 | Initialization correspondence and experiment | Recommend the protected `init_result` consumer of the shared constructor. It checks initial represented fields and demands success on all valid memberships; it is not full node/cluster refinement. The source seed must fail, offline scripted repairs are SIMULATED generation, and only their actual checking is REAL_LOCAL. The initialization profile runs its eight unchanged independent cases; reconciliation runs its 13 operation/replay cases. Previously completed tasks are regressed during continuation. The unused legacy token helper and its two tests are preserved in an excluded fixture. Review the exact added ghost-only import allowance, which imports a specification and does not alter executable code. |

The fixture repair demonstration used no live agent session. The configured experiment requires an explicit launch; the ten technical decisions remain DRAFT.

## Continuation revision candidates (still DRAFT)

| Existing decision | Recommendation delivered for review | Remaining choice |
|---|---|---|
| ADR-01 / ADR-07 | One pending write; Busy preserves it; full token/kind/incarnation matching; NotPerformed and Unknown both fail-stop, with distinct errors and no released reply | Approve or revise direct known-failure retry semantics only together with storage/recovery model and tests |
| ADR-01 / ADR-05 | `submit_persist` returns a token and holds the proposed image in Node.hard; host snapshots it before callback; partial step returns an optional dependent message | Review the eventual Effect/Send operation boundary and legal handler caller proofs; arbitrary images are not authorized Raft transitions |
| ADR-08 | Use installed Verus/vstd spec_fn/nat traces for the early coordinator progress theorem | External temporal-library compatibility, interference, resource adequacy and host fairness transfer remain open |
| ADR-08 / ADR-10 | Protected persistence/progress consumers and independent tests; persistent budgets, scoped checkpoints, conservative affected-task reruns and optional explicit performance rule | Review checker changes and scope; no whole-project acceptance or new runtime approval checkpoint |
| ADR-10 | Keep real microbenchmarks measurement-only until a comparable reference and explicit rule are configured | No approved performance target or cluster measurement claim exists |

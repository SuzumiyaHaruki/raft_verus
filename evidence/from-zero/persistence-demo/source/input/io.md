# Events, effects and host contract

Status: DRAFT. ADR-01/03/05/06/07 await review. Types are in `../crates/raft-verified/src/io.rs`, reexported by lib.rs. The persistence slice implements part of this DRAFT API; other rows remain obligations for the future service.

Recommend `step(event) -> Result<effects, error>` with serial processing in a single-threaded node core. Each external operation receives a node-scoped `(incarnation, sequence)` token; host routing also binds the node. The model calls the sequence `serial`. Incarnation increases durably across recovery; sequence is never reused within an incarnation. Overflow/capacity exhaustion must fail explicitly and stop affected protocol actions, never wrap.

| Input | Library checks and result | Host responsibility |
|---|---|---|
| Message | Validate membership, destination, indices and structure; stale terms do not advance state; correlate late responses with issued requests/tokens and terms; reject unknown senders | Authentic node identity; decoding preserves fields and never invents success |
| Timeout | Accept only the current timer generation; old/duplicate notifications have no effect; valid timeouts trigger election or replication | Authentic timer source; liveness separately constrains timeliness and competition |
| Proposal `(client,id,command)` | Register valid command provenance; reject or redirect at non-leaders; acceptance is local, not commitment; reject changed content for the same identity | Do not reuse client identifiers; retry until commit/delivery is known; backoff cannot starve the request forever |
| Persistence completion | Consume only a matching pending token; success acknowledges the host's already durable whole-image write and releases the retained dependent reply; failure is not success | Success means atomic, ordered, recoverable persistence of the entire write; failure may have written data, so recovery reads actual durable state |
| Send completion | Release local transport resources only; successful send does not mean reception or durable storage | Loss/duplication/reordering are allowed; remote protocol messages acknowledge replicated entries |
| Application acknowledgment | Match a previously delivered position/command; duplicate acknowledgment is idempotent; reject unknown or cross-incarnation callbacks | Acknowledgments reflect actual application and the recovery contract; arbitrary positions cannot be declared applied |
| Recovery | Restore the identity's latest complete durable image; reconstruct volatile indices/pending state; invalidate old operations | Do not treat arbitrary corruption as a fresh node; at most one running instance per identity |

| Effect | Success, failure, retry and cancellation |
|---|---|
| Persist | Send term/vote/log-dependent responses only after success. Failure blocks dependent effects and returns an error. Retry uses a new token. Crash cancels waiting without implying the disk write did not happen. |
| Send | Serialize an actual model-produced message. Transport failure may trigger retry but not invented acknowledgment. Old messages may arrive after a crash. |
| Deliver | Emit only a contiguous committed prefix, with increasing positions within an incarnation. Delivery is not successful application; unacknowledged durable application may require replay after recovery. |
| Timer | Set/cancel carries a generation; late notifications after cancellation cannot start an election. |
| Error | Distinguish NotLeader, InvalidInput, StaleCompletion, StorageFailure and ResourceExhausted; never report false commitment. |

Recommend persisting application state and applied position in one host transaction, or idempotent deduplication by log position. Resume from the durable applied cursor; the volatile delivery cursor may roll back to it and replay. Nontransactional external side effects do not receive an exactly-once guarantee. Reapplying the same command at the same position is permitted; `S-APPLY` prohibits different commands at one position, not application business-logic errors. A durable cursor cannot exceed the actual delivered/applied prefix, and the recovered log must cover it.

Entries distinguish `Client(request_id, command)` from `NoOp`. Only actual valid proposals create Client entries; NoOp cannot impersonate client success. Retries may create multiple positions for one request, always with the same command. Exactly-once request deduplication is a separate future feature.

`A-S-ID/STORAGE/COMPLETION/APP` are external premises. Unique vote counting, log freshness, current-term commitment and stale-token filtering are library proof obligations, not host assumptions. `PO-IO`, `PO-RECOVERY` and `PO-REFINE-STEP` cover errors, crashes and I/O interleavings.

The current coordinator rejects new writes while one is pending (Busy); submission updates volatile hard state while the disk remains host-owned. Both NotPerformed and Unknown currently take the model's conservative fail-stop path, with distinct errors and no dependent reply. A direct known-failure retry would require a reviewed extension; a new token must be allocated after implemented recovery. No recovery path is fabricated in the driver. Returning `Option<Message>` from the partial dispatcher is an interim DRAFT boundary; integrating general Effect values and independently identified sends remains RUNTIME-DISPATCH work.

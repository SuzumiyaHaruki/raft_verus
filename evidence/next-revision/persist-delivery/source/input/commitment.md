# Commitment and ordered delivery

Status: DRAFT. Module: INCOMPLETE; commit advancement and delivery are absent.

## Responsibility and connections

Turn validated durable replication progress into committed prefixes, then deliver those prefixes in order to the host. Keep proposal acceptance, commitment, delivery, application and application acknowledgment distinct. Use [replication](replication.md) for confirmed peer progress, [runtime](runtime.md) for effect/completion ordering and [recovery](recovery.md) for application cursors.

Read `advance_commit`, `deliver`, `application_performed`, `application_ack`, `leader_completeness`, `interface_safety` and `application_safety` in the model, plus [io.md](io.md) and [properties.md](properties.md). Replayed deliveries and repeated request IDs do not imply exactly-once business effects.

## Module completion

Leaders apply the current-term durable-majority rule; followers adopt bounded commit positions from validated AppendEntries. Delivery is ordered, committed and tracked through completion/recovery. Tests cover majority boundaries, old-term entries, delayed responses, failed/delayed application and lost acknowledgments. Prove commit/prefix invariants, delivery correspondence and the relevant global completeness/interface/application claims under their explicit host assumptions. PO-AUX-COMMIT, PO-DELIVERY, PO-S-COMPLETE, PO-S-INTERFACE and PO-S-APPLY stay open; [progress](progress.md) must additionally establish eventual commitment/delivery.

## Step: Advance committed prefixes

Stage: PLANNED. Requires validated leader replication progress.

Count unique fixed members using a strict majority. Advance directly only through a current-term entry durably confirmed by that majority, thereby committing its prefix; an old-term entry on a majority is insufficient by itself. Ignore stale responses, keep commit monotonic and within durable log bounds. Integrate follower commit bounds with its validated request path.

Deliver the decision code, tests for one-node/majority boundaries, old-term and stale-response cases, and a local `advance_commit` proof. Retain a deliberate old-term-commit regression. Actual state, source scope and local test/proof profiles are absent. Supply them before execution; passing a decision function's checks does not automatically prove global leader completeness or finish this module.

## Step: Deliver and acknowledge committed entries

Stage: PLANNED. Requires commit advancement and I/O coordination.

Emit ordered Deliver effects for committed entries and track their operation/index. Handle application success, nonperformance and unknown outcomes according to the existing host contract. Keep an application's actual transaction distinct from its acknowledgment; preserve the cursor relationship used by recovery. Do not emit an uncommitted entry or infer exactly-once effects from request identity.

Deliver delivery/completion code, tests for ordering, duplicate/stale completions and crash replay, plus `deliver`/`application_ack` correspondence and invariant proofs. Supply real delivery tests, crash scenarios and Verus profiles before activation. The application coordinator and those checks are missing. Semantic ambiguities about host transactions or cursors remain review items under ADR-06.

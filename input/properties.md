# Safety and liveness targets

Status: DRAFT. Every target is OPEN. Definitions and typechecking are not proofs.

These targets apply to valid executions over a nonempty finite fixed membership. Their formal definitions live in `crates/raft-spec/src/lib.rs`; [proofs.md](proofs.md#requirement-to-source-map) maps the explanations to source and intended proofs. Ghost histories retain logical events across crashes. Protocol positions start at 1; 0 is a prefix sentinel. Commit transitions record actual durable-majority commitment of a current-term entry. Commitment is not defined by future permanence.

## Safety

| Property / obligation | Target |
|---|---|
| S-ELECTION / PO-S-ELECTION | Every term has at most one historically elected leader, including across crashes. |
| S-APPEND / PO-S-APPEND | Every step starting with a live leader and leaving that node alive preserves its old log prefix, including a step that changes its role. |
| S-LOG / PO-S-LOG | For any two nodes and position i, equal terms at i imply equal prefixes of length i. |
| S-COMPLETE / PO-S-COMPLETE | Every entry committed in term t occurs at the same position in every leader elected in a later term. |
| S-INTERFACE / PO-S-INTERFACE | Deliver only committed prefixes, in order within each incarnation; deliveries at the same position agree across nodes and crashes, and errors never report false success. |
| S-APPLY / PO-S-APPLY | All historical applications at the same position use the same command, including recovery replay. |

## Conditional liveness

| Property / obligation | Target |
|---|---|
| L-ELECTION / PO-L-ELECTION | From every permitted reachable start and subsequent admissible infinite trace, eventually a node l∈Q remains leader in one term, with continued replication opportunities in Q. |
| L-COMMIT / PO-L-COMMIT | Every eligible client request r proposed at finite time k eventually has a corresponding Client entry committed at some position i and time j≥k. At least one instance must commit; an internal NoOp is not a witness. |
| L-DELIVER / PO-L-DELIVER | Every committed entry is eventually delivered in order to the host of each continuously participating Q node. The observation ends at Deliver. |

Environmental premises and applicability belong in [assumptions.md](assumptions.md); proof dependencies belong in [proofs.md](proofs.md).

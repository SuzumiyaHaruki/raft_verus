# vote-collection scoped checks

INCOMPLETE

Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| source_boundary | PASS: same_candidate_imports |
| collection_rules | PASS: checked |
| collection_campaign | PASS: checked |
| collection_authorization | PASS: checked |
| collection_history | PASS: checked |
| collection_reply_rules | PASS: checked |
| collection_self_vote | PASS: checked |
| collection_rounds | PASS: checked |
| collection_higher_reply | PASS: checked |
| collection_provenance | OPEN: verification_failure |
| collection_nonempty | OPEN: verification_failure |
| collection_receive_confirmed | PASS: checked |
| collection_receive_pending | PASS: checked |
| collection_self_confirmed | PASS: checked |
| collection_self_pending | PASS: checked |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

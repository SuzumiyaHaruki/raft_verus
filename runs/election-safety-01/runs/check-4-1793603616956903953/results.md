# election-safety scoped checks

PASS

Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, finite executions from Followers, without node restart. Four local recovery windows are separate. RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | PASS: checked |
| source_boundary | PASS: same_candidate_imports |
| election_rules | PASS: checked |
| election_campaign | PASS: checked |
| election_authorization | PASS: checked |
| election_history | PASS: checked |
| election_reply_rules | PASS: checked |
| election_self_vote | PASS: checked |
| election_rounds | PASS: checked |
| election_higher_reply | PASS: checked |
| election_provenance | PASS: checked |
| election_nonempty | PASS: checked |
| election_promotion | PASS: checked |
| election_election_basis | PASS: checked |
| election_election_history | PASS: checked |
| election_election_nonempty | PASS: checked |
| election_receive_confirmed | PASS: checked |
| election_receive_pending | PASS: checked |
| election_self_confirmed | PASS: checked |
| election_self_pending | PASS: checked |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

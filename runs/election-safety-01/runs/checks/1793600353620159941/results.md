# election-safety scoped checks

INCOMPLETE

Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, finite executions from Followers, without node restart. Four local recovery windows are separate. RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | PASS: checked |
| source_boundary | PASS: same_candidate_imports |
| election_rules | OPEN: compilation_error |
| election_campaign | OPEN: compilation_error |
| election_authorization | OPEN: compilation_error |
| election_history | OPEN: compilation_error |
| election_reply_rules | OPEN: compilation_error |
| election_self_vote | OPEN: compilation_error |
| election_rounds | OPEN: compilation_error |
| election_higher_reply | OPEN: compilation_error |
| election_provenance | OPEN: compilation_error |
| election_nonempty | OPEN: compilation_error |
| election_promotion | OPEN: compilation_error |
| election_election_basis | OPEN: compilation_error |
| election_election_history | OPEN: compilation_error |
| election_election_nonempty | OPEN: compilation_error |
| election_receive_confirmed | OPEN: compilation_error |
| election_receive_pending | OPEN: compilation_error |
| election_self_confirmed | OPEN: compilation_error |
| election_self_pending | OPEN: compilation_error |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

# election-safety scoped checks

INCOMPLETE

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
| election_provenance | OPEN: verification_timeout |
| election_nonempty | OPEN: verification_timeout |
| election_promotion | PASS: checked |
| election_election_basis | OPEN: verification_timeout |
| election_election_history | OPEN: verification_timeout |
| election_election_nonempty | OPEN: verification_timeout |
| election_receive_confirmed | OPEN: check_budget_exhausted |
| election_receive_pending | OPEN: check_budget_exhausted |
| election_self_confirmed | OPEN: check_budget_exhausted |
| election_self_pending | OPEN: check_budget_exhausted |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

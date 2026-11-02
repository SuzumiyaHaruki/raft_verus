# election-safety scoped checks

INCOMPLETE

Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, finite executions from Followers, without node restart. Four local recovery windows are separate. RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | OPEN: test_timeout |
| inventory | OPEN: build_dependency_failed |
| tests_seed_1_raft_raft_vote_tests | OPEN: build_dependency_failed |
| source_boundary | PASS: same_candidate_imports |
| election_rules | OPEN: verification_timeout |
| election_campaign | OPEN: check_budget_exhausted |
| election_authorization | OPEN: check_budget_exhausted |
| election_history | OPEN: check_budget_exhausted |
| election_reply_rules | OPEN: check_budget_exhausted |
| election_self_vote | OPEN: check_budget_exhausted |
| election_rounds | OPEN: check_budget_exhausted |
| election_higher_reply | OPEN: check_budget_exhausted |
| election_provenance | OPEN: check_budget_exhausted |
| election_nonempty | OPEN: check_budget_exhausted |
| election_promotion | OPEN: check_budget_exhausted |
| election_election_basis | OPEN: check_budget_exhausted |
| election_election_history | OPEN: check_budget_exhausted |
| election_election_nonempty | OPEN: check_budget_exhausted |
| election_receive_confirmed | OPEN: check_budget_exhausted |
| election_receive_pending | OPEN: check_budget_exhausted |
| election_self_confirmed | OPEN: check_budget_exhausted |
| election_self_pending | OPEN: check_budget_exhausted |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

# election-safety scoped checks

INCOMPLETE

Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, finite executions from Followers, without node restart. Four local recovery windows are separate. RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.

| Check | Result |
| --- | --- |
| build | PASS: checked |
| inventory | PASS: checked |
| tests_seed_1_raft_raft_vote_tests | OPEN: test_or_build_failure |
| source_boundary | PASS: same_candidate_imports |
| election_rules | OPEN: verification_failure |
| election_campaign | OPEN: verification_failure |
| election_authorization | OPEN: verification_failure |
| election_history | OPEN: verification_failure |
| election_reply_rules | OPEN: verification_failure |
| election_self_vote | OPEN: verification_failure |
| election_rounds | OPEN: verification_failure |
| election_higher_reply | OPEN: verification_failure |
| election_provenance | OPEN: verification_failure |
| election_nonempty | OPEN: verification_failure |
| election_promotion | OPEN: verification_failure |
| election_election_basis | OPEN: verification_failure |
| election_election_history | OPEN: verification_failure |
| election_election_nonempty | OPEN: verification_failure |
| election_receive_confirmed | OPEN: verification_failure |
| election_receive_pending | OPEN: verification_failure |
| election_self_confirmed | OPEN: verification_failure |
| election_self_pending | OPEN: verification_failure |

All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.

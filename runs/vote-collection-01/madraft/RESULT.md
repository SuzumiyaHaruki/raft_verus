The bounded receiver/campaign/real-reply collection implementation and source proof are complete at the generated-code level; all 22 required scoped checks passed in round2-final.log. The aggregate remained INCOMPLETE because PROGRESS.md and auxiliary diagnostic logs changed concurrently; no test or theorem failed. The clean repeat returned VOTE_COLLECTION_CHECKS_PASSED: runs/checks/1793593391348242067/results.json. All 22 required checks passed with no concurrent changes; semantic review remains NOT_PERFORMED and implementation acceptance NOT_ESTABLISHED. The code uses one authoritative Core and the original protected RPC/file wiring. Every consensus method has an actual executable body verified with --no-cheating; no assume, admit, external consensus body or changed fixed acceptance is used.

| Fixed scoped target | Evidence |
| --- | --- |
| Receiver rules and nonempty log decisions | Scoped safety PASS. |
| Campaign and shared state | Scoped safety PASS. |
| Matching completion authorization | Scoped safety PASS. |
| Mixed finite histories and vote uniqueness | Scoped safety PASS, including RPC events. |
| Reply eligibility and actual voter queries | Scoped safety PASS. |
| Self-save eligibility and no automatic leader promotion | Scoped safety PASS. |
| Deduplication and round isolation | Scoped safety PASS. |
| Higher reply save and exhaustion | Scoped safety PASS. |
| Actual cross-node finite-message provenance | Fixed caller PASS: 63 verified, zero errors; final run also PASS. |
| Nonempty three-receiver and duplicate witnesses | Final PASS: 62 verified, zero errors. |
| Four local recovery callers | Final PASS for all confirmed and written-pending receiver/self windows. |
| Actual constructor/step composition | vote_source_entry.rs PASS: 58 verified, zero errors; independently checked via check.py proof. |

Execution tests: all 29 vote tests passed seed 1 after the invariant refactor. The final run passed all five seeds 1 through 5, including original RPC, failed/delayed save, old/new round, duplicate, higher-term and restart regressions. Current logs are round2-tests.log and round2-final.log. Formal local checks are recorded in round2-safety.log; the complete source entry is in round2-source-entry.log and round2-entry-check.log.

Connection: tasks/vote_collection/network.rs::new_network calls the actual Core::new; execute calls actual start_election, receive, complete and collect and records their exact returned sends/replies. vote_sources.rs proves boot initialization, every enabled fixed step, and the fixed sourced conclusion. The invariant retains send/reply/delivery records without a term or grant filter. New self evidence names a successful actual Send completion; new remote evidence names a delivered actual receiver Reply with the original request association. Unchanged votes retain their earlier witnesses. The fixed caller separately checks actual finite-set membership/cardinality and arbitrary finite event iteration. No node restart is included in that network theorem.

The four recovery callers cover exactly the retained confirmed or written-but-uncompleted receive/self windows, using fresh recovered Core state and arbitrary mixed suffixes. Faithful readback is a premise; arbitrary interrupted/failed-write recovery and repeated cluster crashes remain OPEN. Snapshots are unimplemented and unverified. MadSim file writes and sync operations do not establish real-disk durability or power-loss consistency.

Host premises remain UNVERIFIED: faithful fixed membership/address mapping; original immutable RPC result association; no fabricated/cross-wired delivery; faithful actual I/O, completion and retained-image readback; retention in the two specified windows; serial operation wiring; serialization; task cancellation; and forwarding. Concrete host paths are RaftHandle::new_receiver, start_election, send_vote_request, deliver_vote_result, registered request_vote handling, vote_host::load and vote_host::save. These premises establish no remote Raft eligibility; that source conclusion is proved from actual Core behavior in the fixed finite-message history.

Full labs 2A, 2B, 2C, 2D, KV 3A/3B, shard controller 4A and sharded KV 4B remain OPEN. All five global safety targets (Election Safety, Leader Append-Only, Log Matching, Leader Completeness and State Machine Safety), majority/election authorization, leader promotion, replication, snapshot/application correctness and conditional liveness remain OPEN. Scoped machine checks require separate semantic review of actual host coverage and do not establish full MadRaft correctness.

Reproduce with the existing tools.ini: python3 scripts/check.py final --task vote-collection --timeout 300; python3 scripts/check.py safety --task vote-collection --timeout 150; python3 scripts/check.py proof madraft/proof/vote_source_entry.rs --task vote-collection --timeout 45; python3 scripts/check.py test raft::raft::vote_tests --task vote-collection --seed 1 --timeout 120. No fixed-input objection remains.

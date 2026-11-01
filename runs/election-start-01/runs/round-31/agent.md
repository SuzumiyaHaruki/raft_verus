Saved check-30 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass; the checked source matches the candidate. Seeds 1–5 each pass 15/16 tests. The remaining failure is the protected closed-channel assertion, not a tool timeout. No permitted candidate repair resolves it; acceptance remains incomplete pending corrected test input.

Failures and cumulative spending are preserved. Serialization, truthful I/O, faithful readback/retention and unchanged forwarding remain host premises. All five global safety goals remain OPEN.
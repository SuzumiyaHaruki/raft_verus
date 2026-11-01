Saved check-62 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass, including receiver, campaign, Choice history, authorization and four recovery entries. All five seeds still fail only the protected assertion at `rpc_tests.rs:213`: a closed channel returns immediately rather than timing out. No compilation/proof failure or tool timeout occurred.

Acceptance remains **INCOMPLETE**, requiring corrected protected input. Candidate sources are unchanged; failures and cumulative spending remain preserved. Serialization, truthful I/O/readback, retained files and unchanged forwarding remain host premises. All five global safety properties remain OPEN.
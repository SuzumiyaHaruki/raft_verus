Saved check-22 findings in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md) and PROGRESS.md.

Build and all eight proof entries pass. Each of five seeds passes 15/16 tests. The remaining functional failure is a protected-test contradiction: after aborting its sole sender, the test expects a timeout, but receives an immediate closed-channel error. No compilation/proof failure or tool timeout occurred.

Acceptance remains incomplete. Failures and cumulative spending are retained. Serialization, truthful I/O/readback, file retention and unchanged forwarding remain host premises; all five global safety goals remain OPEN.
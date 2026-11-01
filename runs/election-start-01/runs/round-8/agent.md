Check-7 confirms the unchanged blocker. Build and all eight proof entries pass; seeds 1–5 each pass 15/16 tests. The only failure is the protected observer assertion, with no tool timeout.

Saved the evidence in [RESULT.md](/home/nitro/Desktop/raft_verus/runs/election-start-01/madraft/RESULT.md). **Acceptance remains INCOMPLETE; corrected protected input is required.** No candidate repair can keep the test’s channel open after its sole sender is dropped.

Failures and cumulative spending are preserved. Serialization, truthful I/O/readback, retention, forwarding and cancellation remain host premises. All five global safety targets, full labs and liveness remain OPEN.
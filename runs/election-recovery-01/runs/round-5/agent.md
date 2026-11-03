Redesigned the resource-exhausting preservation proofs into event-specific lemmas while retaining actual Core execution and historical evidence.

- Executable finite-restart proof: **149 verified, 0 errors**.
- Build and all 43 election tests passed at seeds 1–5.
- Final scoped check remained incomplete: promotion timed out; ten later groups were not run because the check budget expired.

Saved code, exact diagnostics and next obligation in [PROGRESS.md](madraft/PROGRESS.md), with outcomes in [RESULT.md](madraft/RESULT.md).

Host I/O, file retention, serialization, RPC forwarding, callback cancellation and scheduling remain unverified. Full Raft correctness remains open.
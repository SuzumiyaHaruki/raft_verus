Repaired `uniqueness` and `restart_composition` using the actual Core invariant.

- All 43 election tests passed at seeds 1–5.
- Nine scoped proof groups passed.
- The executable repeated-restart write-origin proof passed: 90 verified, zero errors.

Full Election Safety remains OPEN at the network/history invariant. Exact proof failures, timeouts, unrun checks, next obstacle, and unverified host premises are saved in `madraft/PROGRESS.md` and `madraft/RESULT.md`.
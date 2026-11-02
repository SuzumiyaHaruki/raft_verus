The fixed CLI reports ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED but exits with status 1 after all required checks pass. This is an exit-code mismatch only; the proof assignment is satisfiable and no acceptance exception is requested.

Reproduce unchanged: `python3 scripts/check.py final --task election-safety --timeout 380`; inspect the command exit status and generated results.json. Observed run: runs/checks/1793603270321300948/results.json has the scoped success status, all 26 check results PASS and no concurrent candidate changes; the completed command exited 1.

Cause: scripts/check.py main() recognizes PASS, MACHINE_CHECKS_PASSED, REQUEST_VOTE_CHECKS_PASSED and ELECTION_START_CHECKS_PASSED as zero-exit results, omitting ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED. Fixed checker inputs remain untouched. This objection does not weaken the historical theorem, alter tests or replace any proof obligation.

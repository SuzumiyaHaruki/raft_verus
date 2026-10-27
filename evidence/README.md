# Reading execution evidence

Start with [the review packet](../review/REVIEW_PACKET.md). This directory contains
actual command output and generated controller records; it is not another set of
requirements to edit or send wholesale to an LLM.

- `experiment-setup/RESULTS.md`: current Sol/High defaults, launcher and review-after-delivery checks.
- `module-workflow/RESULTS.md`: earlier module/step revision commands and results.
- `sol-high/RESULTS.md`: coupled-module instructions and explicit model/effort forwarding checks; no live model call.
- `revision/commands.json`: earlier real reconciliation/controller revision.
- `runs/<run-id>/`: a controller run, its candidate, attempts and independent checks.
- `bootstrap/`, `simplification/`, `layout-and-tools/` and other named directories:
  earlier development/checking history.
- Root `model-*` files: earlier tool observations, model checks, verifier calibration
  and licenses. Some remain referenced by calibration configuration.

Inside a run, read `conclusion.json` (or `plan-state.json`) first. `*.check.json`
contains an independent result, `*.stdout.log`/`*.stderr.log` the actual output,
and `*.process.json` the command, exit status and process observations. Attempts
also retain the selected prompt, agent claim, changed paths/patch and repair
feedback. Manifests/bindings associate existing checks and approvals with inputs.
`candidate/` and build directories are generated working artifacts.

Old results are historical: changing source or checker policy can make them
inapplicable to current code. SIMULATED generation/checking must be distinguished
from REAL_LOCAL Rust/Verus execution. A local success is not whole-Raft acceptance.
Raw historical evidence is preserved; no cleanup or retention policy was approved.

# Current evidence

This directory is output, never experiment input. Obsolete maintenance archives, old candidate copies and build products were deleted at the user's request. They were not moved to another archive.

- calibration/: current real positive and negative Verus smoke results used to calibrate the checker.
- checks/protocol-scaffold/: current validation, tests and real Rust/Verus results after removing the maintenance vote implementation. Earlier records under checks/ predate this change; their vote-slice successes do not apply to the current scaffold. Temporary build products are removed after checking.
- runs/RUN_ID/: each explicitly launched experiment or direct check gets a new directory. A plan keeps its candidate and cumulative history there; its task/check record IDs refer to their own unique run directories. Only explicit resume continues an existing plan.

A new experiment starts from the supplied source foundation, never from this directory. Keep required diagnostics while an experiment is being investigated. Completed disposable runs may be deleted deliberately; deleted runs cannot be resumed. Logs are actual observations, not assumptions or automatic global proof claims. Scripted generation is SIMULATED; real tool execution is recorded separately.

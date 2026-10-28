# Supplied host and generated protocol

Status: DRAFT. Python-launched Codex generates and verifies all Raft protocol behavior on a MadRaft-inspired MadSim host. Maintenance supplies infrastructure, requirements and independent checks, not a protocol solution.

## Supplied inputs

The supplied foundation contains the real MadSim host, transport/timer/storage drivers, event/effect vocabulary, observable state, behavioral targets, tests and independent Verus callers. MadSim is pinned to `519950efb4711464f300ed7edf2967ed62d5f502`. The host design refers to MadRaft; no upstream MadRaft protocol implementation is imported. Host provenance and the atomic whole-image storage limitation are described in docs/experiments/madraft-poc.md.

`experiments/madraft-poc/src/cluster.rs` constructs nodes through `Node::new` and drives `Node::handle_event`. It executes effects, supplies truthful completion events and controls faults; it makes no vote, quorum, log-merge or commit decision. Its disk/application models are supplied environment mechanisms, not generated protocol persistence coordination. Protected tests preserve election, commit and application histories across crashes. Finite traces do not prove universal safety or conditional liveness.

## Generated work

The former maintenance-written vote/durable-reply implementation and separate feasibility constructor have been removed. `Node::new`, the persistence coordinator, `handle_request_vote`, `finish_vote`, `handle_event` and reconciliation are unimplemented scaffolds. Generate their executable bodies, internal representations, local contracts, invariants and proofs during this recorded run. Do not retrieve a previous slice or offline solution. A fresh candidate must initially fail the behavioral and independent proof targets; a compiling scaffold is not a protocol implementation.

The default plan reuses STATE-INIT and RUNTIME-PERSIST, then generates voting through MADRAFT-VOTE, elections, replication/commit/delivery and recovery. All steps share one Node and one candidate. The old `step -> Option<Message>` and dedicated vote APIs remain independent boundary-test interfaces; the full cluster uses `handle_event -> Vec<Effect>`. Reuse generated helpers across those entrypoints rather than building separate protocol engines. `src/protocol/` may hold generated submodules imported by both Cargo and the core proof caller.

`Observation` is the inspection vocabulary, not a prescribed internal layout. `from_observation` is a boundary-test adapter, not a recovery algorithm or evidence of reachability. Generate additional semantic invariants and establish their initialization/preservation. Supplied independent specifications state the required boundary behavior; they do not supply implementation-layer proof strategies.

## Independent checking

Constructor, persistence, vote and reconciliation consumers retain their behavioral targets. Their callers import the executable source used by Cargo/MadSim. Generated weak local contracts cannot replace those targets, and rejecting all events cannot satisfy required positive behavior. Original vote scenarios now check generated voting and durable replies. Later scenarios require real elections, commands, catch-up, application and recovery through the same node.

`core-proof` checks a local durable-reply barrier. The five global properties, conditional election/commitment/delivery liveness and complete-execution refinement remain mandatory and OPEN. Their complete independent consumers are still missing, so `raft-safety` and `full` remain BLOCKED. Neither this local plan nor model typechecking closes those gaps.

## Fresh experiments

Each new run creates a unique evidence/runs/RUN_ID/candidate from this scaffold. It excludes evidence, target, offline fixtures, local agent/configuration directories, Agent.md and the maintenance review packet. Budgets and history are fresh. Only explicit resume continues that experiment, and generated candidates are never automatically copied into the source scaffold. Exclusion prevents accidental answer inclusion; it is not hostile-process OS isolation.

# MadRaft-inspired MadSim host scaffold

Status: DRAFT. The supplied foundation is a host, interfaces and independent checks. Initialization, voting, persistence coordination and every other Raft protocol behavior are generated with matching Verus proofs by Python-launched Codex. No completed RequestVote or durable-reply solution remains in the starting source. All global safety, conditional-liveness and executable-refinement obligations remain OPEN.

## Source and tool selection

The host design refers to [MadRaft at 8ec4565ecd3844f54f52865f2d7b3da4044f657f](https://github.com/madsim-rs/MadRaft/tree/8ec4565ecd3844f54f52865f2d7b3da4044f657f). Runtime Cargo.toml/Cargo.lock pin [MadSim at 519950efb4711464f300ed7edf2967ed62d5f502](https://github.com/madsim-rs/madsim/tree/519950efb4711464f300ed7edf2967ed62d5f502), package 0.2.34. The host uses installed Rust 1.92 and Verus 0.2026.01.23.1650a05 on the same raft-verified source with cfg(madsim). Attribution and the retained Apache license are in experiments/madraft-poc/NOTICE.md and LICENSE-APACHE. No upstream MadRaft protocol implementation is imported.

## Boundaries

| Path | Role |
|---|---|
| crates/raft-verified/src/node.rs, persistence.rs, madraft.rs, protocol.rs | Unimplemented constructor, persistence, voting and event APIs; generated during each recorded experiment |
| crates/raft-proofs/madraft_vote.rs, persistence.rs, core.rs | Independent consumers of actual executable methods; local targets only |
| experiments/madraft-poc/src/lib.rs, tests/scenarios.rs | Supplied vote test driver and eight independent durability/fault regressions |
| experiments/madraft-poc/src/cluster.rs, tests/cluster.rs | Actual MadSim nodes, transport, timers, storage, restart and finite history checks |
| experiments/madraft-poc/src/bin/replay.rs | Fixed-seed repeatability check through the actual generated core |

Both hosts construct nodes using the generated Node::new. The cluster makes no quorum, vote, merge or commit decision. Its storage driver models atomic whole-image writes and truthful completion callbacks; the generated core chooses what to persist, retains pending work and decides when a reply is authorized. Host application plus cursor updates form a disclosed transaction. These DRAFT host premises are not verified production devices. Finite MadSim histories do not prove universal safety or temporal liveness.

## Checking

```sh
python3 -m orchestrator verify --profile madraft-poc-build
python3 -m orchestrator verify --profile madraft-poc-runtime
python3 -m orchestrator verify --profile madraft-poc-verus
python3 -m orchestrator verify --profile core-vote-tests
python3 -m orchestrator verify --profile core-proof
python3 -m orchestrator verify --profile cluster-tests
```

The scaffold builds but fails protocol behavior and independent proof checks until generation. The negative-control command `python3 experiments/madraft-poc/negative.py` requires an already passing generated vote implementation; it reports BLOCKED if that prerequisite fails. It then applies a SIMULATED early-reply defect in a disposable copy, requires actual Rust/Verus rejection and restores that same candidate's code for real rechecking. It never supplies a maintenance solution or contacts Codex. Current results are recorded in the review packet; previous successful slice logs do not establish results for this scaffold.

# Design references

Status: DRAFT. References guide the supplied materials and serial repair workflow; they are not proof evidence for this implementation. No external agent framework is imported. Current tool configuration is in tools.md and runtime provenance in experiments/madraft-poc.md. Historical maintenance archives have been deleted.

| Reference | Use and limit |
|---|---|
| [IDS](https://arxiv.org/html/2605.23109v1), sections 2–4, 6 and Appendix F | Joint incremental code/proof synthesis, failed-obligation feedback and local redesign; not evidence that Rust/Verus Raft or liveness is solved. |
| [SkyDiscover synthesis guide](https://github.com/skydiscover-ai/skydiscover/blob/main/skydiscover/synthesize/README.md) | Separate candidate generation and independent evaluation within this existing serial controller; no multi-agent framework dependency. |
| [Ponytail AGENTS.md](https://github.com/DietrichGebert/ponytail/blob/main/AGENTS.md) | Minimal complete changes, reuse, caller inspection and actual checks. |
| [Codex non-interactive mode](https://developers.openai.com/codex/non-interactive-mode), [CLI reference](https://developers.openai.com/codex/cli/reference) | Structured subprocess input/output; installed CLI help/version determines supported flags. No live account/model availability claim. |
| [Verus modes](https://verus-lang.github.io/verus/guide/modes.html), [TCB](https://verus-lang.github.io/verus/guide/tcb.html) | Same-source exec/spec/proof separation and explicit trusted boundaries. |
| [Ongaro raft.tla](https://github.com/ongardie/raft.tla/blob/master/raft.tla) | Quorums, historical state and safety target context; not an imported implementation proof. |
| [etcd/raft doc.go](https://github.com/etcd-io/raft/blob/main/doc.go) | Protocol/I/O separation and durable output ordering; no protocol backend imported. |
| [verus-tla](https://github.com/anvil-verifier/verus-tla/tree/84487a0ef8c01e79fa6aaa035476d0970775bac1) | Temporal API reference only; not installed or compatibility-checked. Existing transparent trace definitions introduce no replacement axioms. |

## Specific repair and interface mechanisms

These exact files were previously inspected; retaining their references does not retain an old experiment or imply current upstream compatibility.

| Repository revision | Files read | Mechanism used here |
|---|---|---|
| SkyDiscover `0d932b690670a7e544388ad362e9876c8bd256a0` | [task.md](https://github.com/skydiscover-ai/skydiscover/blob/0d932b690670a7e544388ad362e9876c8bd256a0/skydiscover/synthesize/examples/proved-dist-kvstore/task.md) | Name the executable interface, independent theorem/consumer, example and real command; prevent reject-all or weakened-target success |
| Same SkyDiscover revision | [dsa.md](https://github.com/skydiscover-ai/skydiscover/blob/0d932b690670a7e544388ad362e9876c8bd256a0/skydiscover/synthesize/workflow/agents/2-synthesis-loop/dsa.md), [isa.md](https://github.com/skydiscover-ai/skydiscover/blob/0d932b690670a7e544388ad362e9876c8bd256a0/skydiscover/synthesize/workflow/agents/2-synthesis-loop/isa.md) | Joint code/proof work, persistent failure/design history, scoped useful checkpoints and bounded redesign seeded by checker observations; one existing adapter, no extra agents |
| ConsensusArena `d93237aac869d897fb4370d7b4852e2b79535d26` | [templates/README.md](https://github.com/hongzicong/ConsensusArena/blob/d93237aac869d897fb4370d7b4852e2b79535d26/templates/README.md), [protocol/README.md](https://github.com/hongzicong/ConsensusArena/blob/d93237aac869d897fb4370d7b4852e2b79535d26/protocol/README.md) | One state owner and the same executable event boundary for runtime and external driver; distinct protocol, recovery, execution and host responsibilities; unsupported paths explicitly reject |
| ConsensusWiki `4a7ac46f731ca8dc606b39ef10e192c07a1099dd` | [AGENTS.md](https://github.com/hongzicong/ConsensusWiki/blob/4a7ac46f731ca8dc606b39ef10e192c07a1099dd/AGENTS.md) | Ground quorum/commit/recovery/timing statements in named sources, preserve formulas and mark uncertainty; no importing another protocol's proof assumptions |

Use bounded executable assignments, independent consumers, persistent attempt explanations and real diagnostics. Internal layout, local contracts, auxiliary invariants and proof strategy may evolve together; independent targets and environmental assumptions cannot be weakened to pass. Safety and conditional liveness must be connected to actual executions. Parent Laputa skills and a complete ConsensusVerifier implementation were inaccessible in the supporting context; their contents are not claimed. No assume/admit or deferred proof hole substitutes for an unproved goal.

# Attribution — DRAFT experiment

This adapter is newly written for this repository. Its three-node test topology, explicit network fault controls and separation of host tasks from protocol state are conceptually adapted from MadRaft's test host. No MadRaft, raft-rs or etcd Raft protocol implementation is imported. MadSim is a Cargo dependency, not a replacement protocol engine.

MadRaft: https://github.com/madsim-rs/MadRaft/tree/8ec4565ecd3844f54f52865f2d7b3da4044f657f. MadSim: https://github.com/madsim-rs/madsim/tree/519950efb4711464f300ed7edf2967ed62d5f502. Both checked repositories provide the Apache License, Version 2.0; the checked license text is retained in [LICENSE-APACHE](LICENSE-APACHE). Copyright notices in the dependency sources remain intact. The MadRaft README credits the MIT 6.824 labs and PingCAP Talent Plan; see that upstream for its original attribution.

These sources are compatibility and host-design references. This maintenance-authored feasibility slice is not evidence that an experiment LLM generated a Raft implementation.

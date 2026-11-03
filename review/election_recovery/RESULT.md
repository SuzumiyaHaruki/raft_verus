# election-recovery 维护交付

本轮新增的是“完整写入成功后，节点在完成通知前或完成通知后停止并重启”的集群执行与测试任务，其他节点和网络继续运行。新增证明目标是保留真实写入来源、跨实例选择与历史当选依据，并对声明范围内的一般有限重复重启历史证明 Election Safety。它补的是已有无重启集群证明与四个局部恢复证明之间的组合缺口，具体不变量和证明仍由后续实验 agent 生成。写中断、失败后恢复、真实掉电、完整 MadRaft、其他四项安全性和条件性活性仍 OPEN，异步宿主仍 UNVERIFIED。

实际 HEAD 为 `7f0a2709f233c09d67b50cc96ff31b6b50974dac`，开始时工作区干净；通过原有外置 Git 元数据核对，未回退。根 `.git` 未暴露给普通 Git 命令，因此维护工具自己的 `git_revision` 如实记录 UNAVAILABLE，外置核对见 [preservation.log](preservation.log)。原学生骨架、requirements、四个 lab、旧任务、旧 runs、tools.ini、根 AGENTS.md 和 .gitignore 均未改动。没有真实 Codex CLI 调用、付费生成、其他 agent、提交或推送。

## 实际修改

| 文件 | 作用 |
| --- | --- |
| `tasks/election_recovery/TASK.md`、AGENTS 与三份提示词 | 英文任务范围、旧/新语义对照、宿主前提、首个依赖与验收命令 |
| `recovery_network.rs` | 实际调用同一个 Core；拆分 Write/Notify；记录本节点成功镜像、运行实例、机械追加的 Choice/Frame；丢弃旧 Core 后从镜像 recover；旧请求保留，死亡调用者的回调不可复活 |
| `caller.rs`、`recovery_lemmas.rs` | 保留旧 18 组入口，新增来源、一般有限重启历史与非空性三组；固定 caller 每个前缀检查目标；跨实例不变量与保持证明为空缺 |
| `host.rs`、`prepare.py` | 仅新增启动恢复状态的被动观察接线；原文件协议与真实 RPC 路径不变；准备三个未完成的实现/证明入口 |
| `recovery_tests.rs` | 四个多节点集成场景和一个文件/屏障校准；原 38 项测试语义保留，总共 43 项 |
| `scripts/election_recovery.py`、`inputs.py` | 接入原任务注册与 checker，分别报告公式结果、阶段验收和全局 OPEN |
| `scripts/check.py`、`run.py` | 共用注册任务的精确成功标签；CLI 正常成功为 0、未完成/失败为 1、BLOCKED/NEEDS_REVIEW 为 2；不按字符串后缀放行 |
| `scripts/tests/test_election_recovery.py`、`maintain.py`、`fixtures.rs` | 离线分派与真实 CLI 子进程退出码回归、实际新工作区文件交付、真实工具和隔离负例校准；维护 fixture 不进入实验输入 |

真实调用链是 `RaftHandle → 同一 vote_core.rs → Write → vote_host::save → complete → 真实发送/收票/晋升`；恢复通过同节点保留文件进入 `vote_host::load → Core::recover`。固定多节点调用者使用相同 Core，成功写入与完成通知分别调度。镜像没有外部 Record 参数；来源只能是该节点实际方法返回的 Write。停止不清空全局消息、Choice 或 Elect 历史，恢复不合成旧 complete、回复或自投票计数。模型的回调实例号对应宿主中被杀任务和旧 Handle 的生命周期，此接线是公开的未验证前提。

## 真实检查与证据

主要记录是 [check-03/results.json](check-03/results.json)，含每条命令、耗时、进程状态、退出码和原始日志；[check-03/inputs.json](check-03/inputs.json) 记录该次输入身份。以下维护结果不构成新候选生成成功。

| 检查 | 实际结果与含义 |
| --- | --- |
| 工具 | 使用现有 Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05 和锁定的 MadSim；没有安装或升级 |
| 未完成输入 | Cargo 构建与清单通过；43 项测试中 10 通过、33 失败；全部 21 组无过滤 Verus 检查均为 `verification_failure`，最终 INCOMPLETE，非缺工具或编译失败；见 [unfinished/results.json](check-03/unfinished/results.json) |
| 固定接口 | 21 组类型检查和条件 caller 检查通过；明确是 TYPECHECK/CONDITIONAL，不是候选证明 |
| 固定调用/观察对应 | 新 `recovery_network` 条件检查 10 verified、0 errors；旧网络条件检查保留；候选合同仍未证明，不能给候选安全性记功 |
| 活观察器与真实文件 | 9 个校准案例 × seeds 1–5 共 45 次通过；包括关闭通道、取消、传输错误、提前输出和实际写后屏障 |
| 核心合同/机械片段错误 | 清除 votedFor、降低 term、丢历史、用错误来源记录四个 fixture 都可编译；Rust 各退出 101，Verus 各以预期后置条件失败退出 1；正常控制通过 |
| 实际执行框架变异 | 丢弃全局观察账本、恢复时改用任意 Record 两种隔离变异均类型检查通过，实际框架对应证明以预期断言/后置条件失败拒绝；补丁及原始日志均在 check-03 |
| 已有核心的隔离 Rust 校准 | 仅把旧实验未修改的 Core 放入 `target/` 诊断副本；新旧 43 项测试 × 5 seeds 共 215 次通过。没有复制旧不变量或集群证明，不是新实验输入或新的生成成功；见 [existing-core-diagnostic.json](check-03/existing-core-diagnostic.json) |
| 错误宿主 | “忽略文件并 new”在真实恢复断言失败：实际 `(0, None)`，要求 `(1, Some(0))`；“屏障移到写前”在独立文件断言失败。两者先成功构建，再运行失败；没有复用变异二进制；最后正确版本 43 项复查通过 |
| Python 回归 | [system-all.log](system-all.log)：135 项通过；维护命令另跑其中 122 项。控制器/报告 mock 明确是 SIMULATED；退出码用真实 Python 子进程测试，不当作 Rust/Verus 证据 |
| 预览 | 成功列出新任务、sol/high、7200 秒和调用命令；没有创建付费实验或会话 |

宿主负例的 Core 文件未修改；文件读取和异步接线不在核心 Verus 证明的覆盖内，所以核心证明不能排除此类错误。本轮没有把旧核心的 Rust 校准或旧无重启证明算作跨重启证明。

开发中保留了实际失败：初始化合同最初缺少“返回状态对应传入初态”的关系，补齐后又补了对应证明；把长场景全部展开成完整 Trace 曾触发求解超时，随后拆为真实 Core 多节点调用见证与环境窗口见证，一般有限历史入口及其全部断言没有删除，固定验收时限没有放宽。check-01、check-02 和 dev 下日志保留这些失败，不作为最终通过依据。

完整检查后核对真实 launcher 时，发现 preparer 虽创建了 `recovery_lemmas.rs`，却未把它放入返回文件清单。已补齐并通过实际临时工作区准备回归，见 [launcher-followup.log](launcher-followup.log) 与 [launcher-followup.patch](launcher-followup.patch)。同时校正维护摘要字段拼写；未改实验结果或原始检查报告。最终准备出的 45 个 Rust 源文件与 check-03 已检查源逐字相同，最终输入身份、三个交付入口和此后续修改范围见 [final-inputs.json](final-inputs.json)。

## 待审核边界与仍开放的义务

新输入的 Core 仍为 `unreached()`，两组候选归纳入口仍为空缺；接收、起选、收票、晋升、恢复的实现与跨实例不变量均须由新 agent 完成。维护通过不等于新任务的无重启或跨重启公式已经通过，更不是完整 Election Safety 已完成。

窗口按当前实例解释：A 为当前实例成功写入并通知完成，B 为当前 Pending 的完整写入成功而尚未通知。恢复本身不伪造一次 complete；新实例须实际写入一次才能再次进入这两个窗口。这样允许同一节点和不同节点反复重启，但不额外包含“恢复后没有新写入又立刻重启”的第三类窗口；这项范围解释应随任务一起审核。

模型使用不回绕的 u64 运行实例号与 usize 有限事件容器；网络允许旧消息延迟、重复、乱序和不投递，旧调用者死亡后回调仅保留为历史证据。实际 RPC 关联、取消、串行化、真实 I/O/读回、编码和文件保留仍 UNVERIFIED；部分写、失败恢复、损坏或陈旧镜像、真实掉电持久性不在本轮结论内。真实日志与摘要的对应以及其余 Raft 功能仍需后续工作。

当前未完成输入的三组新正式检查分别约 34、27、26 秒，仍在既有单项 45 秒内；这不是完成候选的耗时预测。若后续真实候选超时，必须区分求解/调度问题与证明失败，保留原始日志；本轮未提高单项、final 或累计预算。

## 使用命令

维护复现；`--diagnose-existing` 仅启用旧核心的隔离 Rust/宿主诊断，旧候选保持只读，结果不会复制到实验输入：

```bash
cd /home/nitro/Desktop/raft_verus
python3 tasks/election_recovery/maintain.py --output review/election_recovery/recheck-01 --diagnose-existing runs/election-safety-01
python3 -m unittest discover -s scripts/tests -q
```

人工审核后启动一次全新实验；默认模型和累计预算保持 sol/high、7200 秒，命令行覆盖方式不变：

```bash
python3 scripts/run.py new runs/election-recovery-01 --task election-recovery --model gpt-6.1-sol --effort high
python3 scripts/run.py new runs/election-recovery-01 --task election-recovery --model gpt-6.1-sol --effort high --execute
```

只有完整固定验收通过才产生 `ELECTION_SAFETY_COMPLETED_WRITE_RESTART_CHECKS_PASSED`，仍需语义审核；完整故障恢复与全量 Raft 不随此状态关闭。本轮在维护交付处停止，没有新增运行中审批文件或交互审批关卡。

# 接收端与主动竞选生成任务维护交付 — DRAFT

本轮交付 `election-start` 生成任务的输入、接线和独立检查，没有替实验 agent 完成接收／竞选算法、不变量或证明，没有运行 Codex CLI、其他 agent、提交或推送。任务维护检查通过不等于候选通过；当前八组候选证明均 OPEN，完整 MadRaft、完整 2A 和五项全局安全性仍未完成。

## 实际修改

| 文件 | 作用 |
| --- | --- |
| [任务与提示词](../tasks/election_start/TASK.md)、[AGENTS](../tasks/election_start/AGENTS.md)、start/continue/redesign.md | 英文任务、固定规则、实际路径、修复要求和未验证前提；旧 request-vote 的五份当前提示文档也改为英文，历史实验提示记录不改 |
| [api.rs](../tasks/election_start/api.rs)、[caller.rs](../tasks/election_start/caller.rs) | 纯规格和独立实际调用：接收、起选、完成授权、混合有限历史、接收／自投票各两种恢复窗口；不提供实现专属不变量 |
| [core.rs](../tasks/election_start/core.rs)、[lemmas.rs](../tasks/election_start/lemmas.rs) | 未完成入口；执行体仍为 `unreached()`，关键证明仍为 `assert(false)`；实验 agent 自行构造表示、不变量和证明 |
| [prepare.py](../tasks/election_start/prepare.py)、[driver.rs](../tasks/election_start/driver.rs)、[host.rs](../tasks/election_start/host.rs) | 仅改新副本：删除旧 State 实例，所有本轮路径和 term/role 查询共用 Core；真实保存、结果回传、原 RPC handler 和实际向成员发送的薄接线 |
| [rpc_tests.rs](../tasks/election_start/rpc_tests.rs)、[election_tests.rs](../tasks/election_start/election_tests.rs) | 保留原八个接收测试及断言，机械适配 Action 并加强取消观察；新增八个混合调用、边界、三节点 RPC、延迟／失败、自投票恢复和单成员不晋升测试 |
| [election_start.py](../scripts/election_start.py)、[request_vote.py](../scripts/request_vote.py)、run.py / check.py / inputs.py | 一个新任务描述，复用原检查循环、会话、累计预算、失败记录和输入保护；分项反馈及局部完成标签，不新增调度平台 |
| [maintain.py](../tasks/election_start/maintain.py)、[系统回归](../scripts/tests/test_election_start.py) | 真实工具维护、明确错误的 fixture、隔离构建、固定检查不可篡改、输入隔离及英文提示检查；维护 fixture 不进入实验输入 |

准备后的实际路径为 `RaftHandle::start_election / 注册的 RequestVote handler → 同一个 Core → host.save → Core.complete → 实际 RPC / 原请求者回复`。准备好的 Write.action 只是数据，只有实际 complete 返回 Send 才授权发送；宿主使用该轮不可变 payload，不从另一份状态拼接任期、身份或日志。起选会记录实际自投票选择，肯定回复记录实际返回选择；两者共同受唯一性约束。自投票观察本身不是成功 I/O，也不是可以计入多数派的票。

所有当前提供给实验 LLM 的自编材料和动态反馈均为英文；三种模式的输入清单及续接文本已有回归检查，另见 [语言检查](election_start/language-check.json)。中文维护报告和旧结果不是新实验输入。原 `full` 和 `request-vote` 模式及其目标保留，tools.ini 未改：`gpt-6-astra / high`，累计 7200 秒，包含 agent 和工具时间，种子 1–5。

## 真实检查及证据

最新实际命令为 `python3 tasks/election_start/maintain.py --output review/election_start/check-02`；首轮 check-01 及开发中真实诊断也保留。逐条命令、环境、退出码和原始日志见 [results.json](election_start/check-02/results.json)；输入副本见 [source-inputs](election_start/check-02/source-inputs)，接线及宿主负例见 [preparation.patch](election_start/check-02/preparation.patch) 和 [host-send-before-save.patch](election_start/check-02/host-send-before-save.patch)。

| 检查 | 结果和含义 |
| --- | --- |
| 固定规格及最后复查 | 真实 Verus：19 verified，0 errors；只是固定定义及其派生义务，不是协议实现证明 |
| 八组独立 caller | 类型检查通过；`--verify-root` 条件检查通过。候选合同及引理仍是未证明前提，明确不计为候选函数体证明 |
| 新 election-start 初始输入 | 真实 Cargo 构建／测试清单成功；未完成核心的测试失败，八组实际函数体 Verus 检查均因 verification_failure 拒绝；总体 INCOMPLETE |
| 原 request-vote 初始输入重检 | 构建／八项测试清单成功，四项候选证明仍正确拒绝未实现输入；未修改或重跑旧 request-vote-01 答案来展示通过 |
| 八类核心错误 fixture | 均可 Rust 编译；实际功能断言与 Verus 语义义务均拒绝：原任期自投、分裂投票状态、错误候选者、错误请求任期／日志、无完成授权发送、恢复丢投票／复用编号 |
| 实际宿主保存前发送 | 正常接线在三节点延迟保存测试通过；仅修改宿主提前发送后，在 `no campaign RPC before save` 断言失败；恢复接线、换独立构建目录后通过 |
| 两种自投票窗口的真实错误恢复 | 实际 kill、等待旧 Core 释放、同节点重启并检查保留文件后，均在 `recover self vote and next from file` 拒绝 next=0（应为 1）；这是恢复负例，未宣称完整正确恢复实现已通过 |
| 系统回归 | 105 项通过：92 项离线系统回归（含标明 SIMULATED 的 agent/checker），13 项已有真实 Rust/Verus 检查；模拟控制器结果不计为协议证明 |
| 新实验预览 | 已运行，不创建实验目录，不调用 agent；实际执行命令留给操作者 |

宿主测试使用明确错误的单轮 fixture 核心，它在变异前后均被 Verus 拒绝，不是正确竞选答案或“已证明基线”。宿主提前发送由真实接收断言发现，没有宣称 Verus 证明了异步宿主。正式任务验收不使用 `--verify-root` 或 `--no-verify`，每组都以 `--no-cheating` 检查实际导入的候选函数体、引理及固定调用者。维护运行使用种子 1；最终验收要求同一份候选上的全部 16 个任务测试、全部五个种子和全部八组证明。

工具沿用 Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05、Cargo.lock 中的 MadSim 0.2.34（519950ef 对应修订）。已查看本地锁定 MadSim 的 JoinHandle 取消、kill/restart 和文件实现：Drop 会分离任务，is_exit 不证明旧对象已释放，sync_all／掉电模型也不提供真实磁盘持久性证明。新恢复测试等待取消并用 Weak 观察旧 Core 释放，再在同节点创建新端点和 Core；真实文件读取不传入旧对象。没有升级工具。

## 保留范围与未完成项

根 `.git` 元数据不可用，`git status` 实际退出 128；没有声称读取到根 HEAD。修改前捕获的 453 个文件与可读发布副本 `623219a314cd59b85d3a81ae97079a8b665228f3` 逐字节一致。根 madraft 的 31 个文件、requirements 的 10 个文件、四个 lab 的 267 个文件均与修改前逐字节一致；35,931 个预存 runs 文件的大小和修改时间未变。原接收任务非 Markdown 文件未改。详见 [preservation.json](election_start/preservation.json) 和 [已有文件差异](election_start/system-changes.patch)。未重置、复制旧解法或改写历史结论。

`.gitignore` 恢复忽略 target；本轮可读日志、输入和补丁保存在 review，构建缓存／二进制只留本地。已有 target 文件若早已进入 Git 索引，仅改 ignore 不会取消追踪；本环境不能访问根索引，因此这一步没有假称完成，也没有删除缓存或历史证据。

以下语义和边界仍需审核：共享单 Pending 串行保存；每次接收决定都保存，包括不变的拒绝；匹配失败后 Halted；起选错误优先级 Halted → Busy → Leader → term=MAX → next=MAX；单成员也不晋升 Leader。合法非空日志摘要作为输入，并未证明它对应实际复制日志。固定 readback 比较只表达真实保留记录等于实际保存内容的宿主前提，不把唯一性或恢复正确性当作前提。

| 目标 | 当前状态 |
| --- | --- |
| 接收规则、起选、自投票及匹配完成后的发送授权 | OPEN，候选实现未生成 |
| 任意有限混合历史的本节点同任期选择唯一性 | OPEN，实现不变量和证明待 agent 生成 |
| 接收／自投票的已完成及已写未完成恢复组合 | OPEN，未建立完整候选的组合证明 |
| 全部候选 RPC／文件／重启测试 | 未通过；维护负例和接线测试不能代替候选验收 |
| Election Safety、Leader Append-Only、Log Matching、Leader Completeness、State Machine Safety | 各自 OPEN |
| 完整 2A、回复来源／高任期回复处理、计票／多数派／晋升、复制、快照、KV／分片、条件性活性 | OPEN，原目标未删除 |

宿主串行化、实际写入与保留、编解码、如实通知、成员地址映射、原样发送、取消和异步接线仍未获 Verus 证明。仅覆盖最后接受的操作已成功保存后的两种指定窗口，不覆盖 create/write 中断、写失败后恢复、任意多次崩溃的归纳或真实掉电。局部标签 `ELECTION_START_CHECKS_PASSED` 仍要求后续语义审查，绝不代表集群 Election Safety。

## 复现与下一次生成

```bash
cd /home/nitro/Desktop/raft_verus
# Maintenance only: new evidence directory, real local tools, no agent.
python3 tasks/election_start/maintain.py
# Preview only.
python3 scripts/run.py new runs/election-start-01 --task election-start
# Operator starts one fresh experiment after reviewing this delivery.
python3 scripts/run.py new runs/election-start-01 --task election-start --execute
```

最后一条命令本轮没有执行。它将从未完成根骨架准备新工作区、新会话，让实验 agent 共同生成接收端、起选实现及证明；旧候选、维护 fixture、会话和缓存源码不进入输入。本轮交付到此停止，等待人工审核。

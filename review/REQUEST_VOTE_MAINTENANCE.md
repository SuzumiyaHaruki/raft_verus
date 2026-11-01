# RequestVote 生成任务维护交付 — DRAFT

**任务已准备，生成未运行／未成功。** 本轮没有启动 Codex CLI 或实验 agent，没有完成投票算法或投票历史证明，没有提交、推送、升级工具。交付的是从原始 MadRaft 创建未完成候选、约束其真实执行路径并独立验收的局部任务。

## 修改范围

| 文件 | 作用 |
| --- | --- |
| [TASK.md](../tasks/request_vote/TASK.md)、[任务 AGENTS.md](../tasks/request_vote/AGENTS.md)、start/continue/redesign.md | 中文范围、规则、宿主前提、启动与修复提示；不要求深入 RPC/执行器证明 |
| [api.rs](../tasks/request_vote/api.rs) | 固定类型与纯 Verus 规格：投票资格、term/vote/log、Pending、完成授权、恢复及实际回复唯一性 |
| [caller.rs](../tasks/request_vote/caller.rs) | 独立调用实际 Core；分别检查规则/非空回复、一般有限历史、已确认恢复、已写未 complete 恢复 |
| [core.rs](../tasks/request_vote/core.rs)、[lemmas.rs](../tasks/request_vote/lemmas.rs) | 未实现的学生入口；执行体为 unreached，证明为开放义务，没有正确投票答案 |
| [prepare.py](../tasks/request_vote/prepare.py)、[host.rs](../tasks/request_vote/host.rs) | 仅在新候选加入 Core 和证明文件，接入原 MadRaft 注册 handler；宿主执行串行化、文件 I/O、结果回传和转发 |
| [rpc_tests.rs](../tasks/request_vote/rpc_tests.rs) | 8 个固定测试，包括两个真实客户端、非空日志、延迟/失败、错误完成、边界、两种同节点 kill/restart |
| [run.py](../scripts/run.py)、[inputs.py](../scripts/inputs.py)、[check.py](../scripts/check.py)、[request_vote.py](../scripts/request_vote.py) | 增加窄的任务选择、输入版本绑定、保护文件、分项反馈和独立局部成功标签；复用原会话、预算、失败记录和检查机制 |
| [系统回归](../scripts/tests/test_request_vote.py)、[maintain.py](../tasks/request_vote/maintain.py) | 输入隔离、续跑范围不可切换、固定测试不可篡改、局部结果不冒充全量成功，以及真实工具负例 |

新候选实际路径是 `RaftHandle::new → 共用构造接线 → start_rpc_server → RaftHandle::request_vote → Raft::request_vote → Core::receive/complete`。恢复使用同一构造接线和注册 handler，并要求文件存在；不传入旧 Core 或期望投票状态。Core 可以重选私有表示、ghost 历史、不变量和引理，固定 caller 检查其实际输出和状态，不以导入文件或覆盖表代替证明。其他学生功能仍为 TODO。

根目录 Git 元数据不可用，`git status` 实际退出 128，故没有假称确认当前 HEAD。修改前保存的 327 个文件与可读发布副本 `54569d0d7a562220af168abb90cd746d86230c9b` 逐字节一致。根 `madraft/`、`requirements/`、四个 `labs/` 及其证据、`docs/`、AGENTS、tools.ini、旧 RESULT 和 .gitignore 均未改；旧 runs 未用作输入或修改。详见 [保留检查](request_vote/preservation.json)。

## 实际检查

主命令为 `python3 tasks/request_vote/maintain.py --output review/request_vote/final`；随后追加运行了“已写未回复重启后错误改投”的真实 fixture，并将它纳入默认维护命令。每条实际命令、环境、退出码和日志在 [results.json](request_vote/final/results.json)。源码输入在 [source-inputs](request_vote/final/source-inputs)，候选接线差异在 [preparation.patch](request_vote/final/preparation.patch)。

| 检查 | 实际结果及含义 |
| --- | --- |
| 固定规格 | [15 verified，0 errors](request_vote/final/fixed-definitions.log)；只是规格和派生义务检查，不是投票实现证明 |
| 三个历史/恢复独立入口类型检查 | TYPECHECKED，0 verified；不记作证明通过 |
| 固定 caller 的条件检查 | history 6、confirmed/pending 各 7 verified，0 errors；使用 `--verify-root`，候选合同和引理仍未证明，因此只说明调用者结构可在这些合同成立时闭合，绝不用于任务验收 |
| 原始未完成候选 | [Cargo 构建、完整测试清单成功；局部测试失败，四项 Verus 目标 OPEN，总状态 INCOMPLETE](request_vote/final/unfinished-results.json)；没有把未实现失败计为语义反例 |
| 总拒绝 fixture | Rust 编译成功；真实功能断言失败；Verus 在非空实际授票断言拒绝 |
| 忽略 votedFor fixture | Rust 编译成功；真实功能断言失败；Verus 拒绝两次实际函数输出组成的同任期改投历史 |
| 恢复清空投票 fixture | Rust 编译成功；真实功能断言失败；Verus 拒绝恢复值等于保留记录的后置条件 |
| 宿主假报保存成功 | 在实际 MadRaft RPC 上运行；独立文件读取因文件不存在失败，诊断为“authorized grant must be in real file” |
| 宿主忽略文件初始化 | 实际保存、同节点 kill/restart 后，在“restore vote from file”断言失败 |
| 已写未 complete 重启后的错误改投 | 正常宿主加故意总授票核心：通过写后屏障、真实文件、无提前回复、同节点重启和无补发检查，最终在同任期 B 不应获票的断言失败；没有构造正确候选 |
| 系统回归 | [96 项通过](request_vote/final/system-regression.log)：83 项离线系统回归（含明确标注的 SIMULATED agent/checker），另有 13 项已有真实 Rust/Verus 检查；两者不混称 |
| 正式检查后的规格复查 | [15 verified，0 errors](request_vote/final/definitions-recheck.log) |

每个宿主 fixture 使用独立构建目录。三个核心 fixture 是明显错误的小函数；宿主 fixture 使用同一个实际 MadRaft handler 中的故意总授票核心，均不是实验 agent 产出的候选变异。没有声称这些错误核心的证明通过；宿主 I/O 错误可能不改变核心定理，正是需要独立文件/RPC 测试的原因。开发中曾出现 fixture 语法错误，[原始日志](request_vote/fixture-development.log) 单独保留，不计入语义错误检出。

工具沿用 Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05 和锁定 MadSim 0.2.34；没有安装或替换全局工具。实际检查使用 tools.ini 配置的环境，Cargo 与 Verus 同用候选 Core，Verus 也设置 `madsim` cfg；正式独立验收使用 `--no-cheating`，不使用 `--verify-root` 或 `--no-verify`。

## 当前状态与待审约定

| 目标 | 状态 |
| --- | --- |
| 接收规则及非空合法回复 | OPEN：实现未生成 |
| 一般有限接收/完成历史同任期唯一性 | OPEN：实现专属不变量和证明待生成 |
| 已确认保存后的恢复组合 | OPEN |
| 已写成功但尚未 complete 的恢复组合 | OPEN |
| 实际候选 RPC/文件测试全部通过 | 未成立；目前只有未完成候选和错误 fixture |
| Election Safety / Leader Append-Only / Log Matching / Leader Completeness / State Machine Safety | 全部 OPEN |
| 完整 MadRaft / 条件性活性 | INCOMPLETE / OPEN |

以下是本次 DRAFT 的明确约定，尚待人工审核，不是对主 Raft 环境的修改：所有被接受的决定均保存，包括状态未变的拒绝；匹配 I/O 失败后停止接收端；固定编号成员、term=0 请求和非法日志组合显式拒绝；MAX 是操作身份耗尽标记，不限制 Raft term/index 为 MAX；合法非空日志摘要由初始化 fixture 提供。真正日志到摘要的对应、其他修改 term/vote 的入口、计票/多数派、任意崩溃、写失败恢复和 create/write 中断仍 OPEN。

公开的未验证边界是操作串行化、外部 I/O 真实发生、文件保留和解码、完成结果如实上报、原 RPC 请求者接收授权回复。核心的成功通知不是 I/O 证明。Verus 有限历史和两种恢复结论只能在这些宿主前提下成立；测试只覆盖实际运行的有限场景，不能证明宿主的所有执行。独立 caller 已连接真实函数，但工具链、ghost 擦除、导入依赖及最终实现的语义对应仍需审核。

## 下一次实验

模型与预算未改：`gpt-6-astra / high`，累计 7200 秒，包含 agent 和工具检查。默认全量模式保留；仅显式 `--task request-vote` 使用局部输入/验收，resume 不能切换任务。全新局部实验不会携带旧 runs、维护 fixture、实验会话或四个 lab 的证据。

```bash
cd /home/nitro/Desktop/raft_verus
# 预览，不创建候选，不调用任何 agent。
python3 scripts/run.py new runs/request-vote-01 --task request-vote
# 人工审核本交付后，由操作者启动一次新实验。
python3 scripts/run.py new runs/request-vote-01 --task request-vote --execute
# 需要时继续同一个实验，保留会话和累计花费。
python3 scripts/run.py resume runs/request-vote-01 --execute
# 单命令复现维护检查，自动选择新的证据目录；不调用 agent。
python3 tasks/request_vote/maintain.py
```

预览已经实际执行；没有执行上面的 `--execute`。局部最终通过时只能报告 `task_scope=request-vote-receiver`、`REQUEST_VOTE_CHECKS_PASSED`，全量目标继续 OPEN。本轮到此停止，等待人工审核。

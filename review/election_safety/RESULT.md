# Election Safety 生成任务维护结果

本轮准备的真实行为是：同一 MadRaft 核心凭本轮已保存的严格多数票晋升 Leader，driver 在多数形成时立即调用，不等待全部 RPC。后续实验要证明真实投票、Choice 历史、当选依据和角色变化共同满足无节点重启范围的历史 Election Safety。它补的是从“票有来源”到“真实当选且同任期历史唯一”的缺口。完整协议、任意崩溃恢复、其他四项全局安全性、日志/快照/应用及条件性活性仍未解决。

本轮结论为 **MAINTENANCE_CHECKS_PASSED**，新候选 **尚未生成**。这不否定既有 vote-collection-01 的局部通过，也不将其答案复制成新输入。实际参考 HEAD 为 `81f5f7c96728ad184641eadf48f02c487137a09e`；根 `.git` 未暴露，使用现存临时 Git 元数据核对，开始时工作区干净。[保留检查](preservation.log)确认已追踪的原学生骨架、requirements、旧任务、runs、四个 lab、AGENTS 和 tools.ini 未变。本轮没有调用 Codex CLI、其他 agent、提交或推送。

## 实际改动

新任务版本为 `madraft-election-safety-1`，位于 [tasks/election_safety](../../tasks/election_safety/TASK.md)。`api.rs` 固定晋升语义；`core.rs` 所有协议入口仍未完成；`lemmas.rs` 的候选不变量与选举证明仍 OPEN。Leader 保留当选票集、停止计入等任期迟到票；更高任期仍先退选、清票并按原策略保存。旧阶段冲突及替换理由已列入任务表，旧任务本身不变。

`driver.rs` 和准备补丁把晋升调用接到自投保存后、每次收票及接收端保存完成后。判断全部留在核心。新 `vote_call` 只执行实际调用和被动记录前后视图，原 `State` 不成为第二份权威状态。已删除新准备副本中多余的未使用接收转发入口；与原准备脚本的差异见 [prepare.diff](prepare.diff)。

`network.rs` 扩展现有实际 Core 调用框架，新增仅检查节点边界的 Promote 步骤；每一步独立追加实际前后状态和当时票集。固定结论要求票连接各投票者真实 Choice，历史当选依据在清票和退选后仍成立。`observation.rs` 不信任候选返回的胜选标记，非法 Leader 任期变化也会被记录；其词汇桥接使用原 `targets::Event::Elect`，不构造虚假日志或快照。多数派交集、协议归纳不变量和完整候选证明均留给实验 agent。语义依据与[原论文 Figure 3、§5.2](https://raft.github.io/raft.pdf)的历史当选唯一性一致，本阶段不引入心跳与计时器。

Python 仅增加阶段描述与报告字段、注册任务，以及晋升目标的入口分类；`run.py` 和 `check.py` 未改，也没有新控制循环。报告分列 `scoped_election_safety_no_restart`、四个局部恢复窗口及完整五项目标；固定输入异议、有界诊断、功能失败阻塞和无进展停止继续复用。

## 实际检查与原始依据

最终证据为 [check-02/results.json](check-02/results.json)，命令、退出码、耗时及环境均在其中；[inputs.json](check-02/inputs.json)记录本轮输入身份，结束时重新比对一致。沿用 Rust 1.92.0、Verus 0.2026.01.23.1650a05 和已有离线 MadSim 依赖，没有升级工具。

| 检查 | 实际结果与含义 |
| --- | --- |
| 固定纯定义 | Verus exit 0，22 verified / 0 errors；不是候选算法证明 |
| 18 组接口 | 全部真实类型检查 exit 0；`--no-verify` 不计证明 |
| 18 组固定调用者 | 全部条件检查 exit 0；`--verify-root` 信任未证明接口，不计候选验收 |
| 多节点调用/观察接线 | 条件模块检查 exit 0，11 verified / 0 errors；候选合同仍未证明 |
| 观察器、词汇桥接及数学控制 | 完整 `--no-cheating` exit 0，28 verified / 0 errors；只证明这些通用支持 |
| 未完成候选 | Cargo 构建/清单通过；38 项测试中 9 通过、29 因未实现失败；18 组无过滤正式证明均为 `verification_failure`，聚合 INCOMPLETE |
| 观察与调度校准 | 8 项 × seeds 1–5，共 40 次真实 Rust/MadSim 检查 exit 0 |
| 四类错误 fixture | 全部可编译；Rust 各 exit 101，Verus 各 exit 1，均为预期断言/后置条件诊断，非编译错误 |
| 正常 fixture | Rust、Verus exit 0；四种错误的目标未随负例更改 |
| 离线系统回归 | 116 项通过，exit 0；其中 agent/checker mock 明确为 SIMULATED，不替代上面的真实工具检查 |
| 新实验预览 | exit 0，未创建候选或会话，未运行 agent |

四类负例分别为半数冒充严格多数、未保存自投就授权晋升、漏掉真实当选、删除退选后的旧当选记录，见 `bad_*-rust.log` 和 `bad_*-verus.log`。观察校准另检出“先 A 当选、A 退选、B 又在旧任期当选”的历史冲突，以及错误的 Leader-to-Leader 换任期。通道关闭、异常和传输错误仍与有效静默分开；新等待顺序校准有正常屏障控制及提前输出负例。

固定功能测试覆盖单成员保存前后、三节点第三个 RPC 尚阻塞时晋升、四成员半数拒绝、五成员重复票不足、多候选分票无人当选、旧轮与重复完成、更高任期退选，以及四个恢复窗口。五节点历史测试在 A 真正当选并退选后才向 B 释放 E 的真实肯定旧票；B 仍不能组成多数，A 的旧当选必须保留。这些协议测试已编译，但**尚未在完整生成候选上正向跑通**；未完成输入失败与观察 fixture 通过都不等于这一步已完成。

## 剩余边界与运行命令

新候选的晋升实现、投票到 Choice 的跨节点归纳、历史多数依据与 Election Safety 均 OPEN。集群定理只从合法 Follower 配置开始，涵盖一般有限非空成员和合法非空日志摘要；摘要来自真实复制日志的关系未证明。四个本地恢复窗口单独 OPEN，不能与无重启集群定理相加为任意故障安全性。完整 Election Safety、其他四项目标、完整 MadRaft/2A 和条件性活性继续 OPEN。

宿主的 RPC 关联、成员映射、串行接线、真实 I/O/完成/读回、保留文件、编码、取消与转发仍 UNVERIFIED；没有把合法多数或不双投塞进宿主前提。Leader 保留票集、迟到等任期票不再计入，以及有效 Follower 初始日志边界已明确为新任务 DRAFT 语义，仍需人工审阅。后续候选的语义验收未执行。本轮没有付费模型调用，费用不作估算。

本轮实际执行的总复现命令为下列第一条；再次执行须使用新的输出目录名。默认配置仍为 `gpt-6.1-sol / high`、累计 7200 秒、seeds 1–5；本轮实际预览未传模型覆盖参数，也未启动生成。最后一条是操作者审核后的一次全新实验执行命令，显式指定同一模型和强度。

```bash
cd /home/nitro/Desktop/raft_verus
python3 tasks/election_safety/maintain.py --output review/election_safety/check-02
# Reproduce maintenance into a fresh evidence directory:
python3 tasks/election_safety/maintain.py --output review/election_safety/check-next
# Preview only:
python3 scripts/run.py new runs/election-safety-01 --task election-safety
# Operator-started generation; not executed during this maintenance:
python3 scripts/run.py new runs/election-safety-01 --task election-safety --model gpt-6.1-sol --effort high --execute
```

维护到此停止，等待人工审核。没有新增审批文件或交互审批节点。

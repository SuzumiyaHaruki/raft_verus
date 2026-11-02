本轮准备的实际协议行为是：候选者把真实 RequestVote RPC 回复交回同一个 Core，维护当前有效投票者集合。**这是 DRAFT 生成任务准备完成，不是收票实现已生成。**

后续实验需要新增的证明是：实际选票的来源、原请求轮次归属、去重、自投保存资格，以及新回复事件保持已有接收、起选和受限恢复保证。

它为 Election Safety 补上真实选票依据；本轮没有多数派或真实当选动作，不能据此宣布 Election Safety 已证明。

尚未完成的是：多数派授权当选、完整集群故障恢复、其他四项全局安全性及完整 MadRaft；条件性活性仍 OPEN。

交付位于 [tasks/vote_collection](../../tasks/vote_collection/)。[TASK.md](../../tasks/vote_collection/TASK.md) 和提示词全部英文；[api.rs](../../tasks/vote_collection/api.rs) 固定行为和观察；[core.rs](../../tasks/vote_collection/core.rs)、[lemmas.rs](../../tasks/vote_collection/lemmas.rs) 仍为未完成学生入口，未提供正确收票算法或 Raft 归纳不变量。[caller.rs](../../tasks/vote_collection/caller.rs) 调用真实候选；[network.rs](../../tasks/vote_collection/network.rs) 从实际 Send/Reply 输出记录消息，执行任意有限无重启调用，保留旧消息并允许重复、延迟、乱序和丢失；来源结论不能成为输入或过滤条件。独立非空调用要求三个实际 Core 完成往返后得到三票，重复 B 的回复仍为两票。

新任务的 prepare/driver/host 在新副本中接入原 RaftHandle 和注册 RPC，每个返回携带原发送上下文并即时交给核心；网络等待在串行操作锁之外，存储操作仍串行。宿主只传递数据、执行 I/O 和转发动作。固定测试保留原 20 项及修好的存活观察器，增加 9 项计票、来源关联观察、队列、真实 RPC、延迟旧回复、高任期保存及失败场景；新增高任期/拒绝场景先调用真实核心竞选、保存和 Send 授权，再按测试安排投递请求，高任期场景隔离第三节点以确保观察的是 B 的迟到回复；四个恢复测试增加清空票集和拒绝复活旧轮的检查。全部 29 项、五个 seed、14 个正式证明组必须在同一候选上满足，才能得到局部 `VOTE_COLLECTION_CHECKS_PASSED`。

Python 仅增加一个 24 行任务声明并登记到既有分派；run.py 仍为 1066 行，未增加调度循环。原输入异议、一次有界诊断、具体失败反馈、无实质进展停止、同次续跑和累计预算机制保留。模型保持 `gpt-6-astra / high`，累计预算保持 7200 秒。没有调用 Codex CLI、实验 agent 或其他 agent，没有提交、推送或修改全局工具。

最终原始记录见 [check-04/results.json](check-04/results.json)，含实际命令、环境、退出码和日志；输入身份见 [inputs.json](check-04/inputs.json)。使用已有 Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05 和锁定的 MadSim 0.2.34 / 519950efb4711464f300ed7edf2967ed62d5f502。当前 .git 为空，无法独立读取本地 HEAD；fa5f6bb 是用户提供的参考提交，未回退到它。[preservation.json](preservation.json) 核对原学生骨架、requirements、labs、全部旧 runs、旧任务、已有审核记录及 tools.ini 的内容保持不变。

| 实际检查 | 结果与含义 |
| --- | --- |
| 固定纯规格及最终复查 | 22 verified，0 errors；不证明候选实现 |
| 14 个入口类型检查 | 全部通过，TYPECHECKED ONLY |
| 14 个固定 caller 条件检查 | 全部通过；使用 --verify-root，候选合同和引理未证明，不计正式验收 |
| 实际调用与记录框架条件检查 | 11 verified，0 errors；仅检查 vote_network 模块，依赖尚未证明的候选合同，来源不变量仍 OPEN |
| 真实 Cargo 构建与固定测试清单 | PASS；运行的是原学生输入加未完成新任务骨架 |
| 未完成输入真实功能检查，seed 1 | 29 项中 7 通过、22 失败，整体 INCOMPLETE；不是正确候选的正向验收 |
| 14 项正式 Verus 主体检查 | 全部以 verification_failure 拒绝未完成输入；没有用类型检查或函数过滤替代候选验证 |
| 新旧观察器及队列校准，seed 1–5 | 6 项 × 5，共 30 次真实 Rust/MadSim 检查通过；不执行正确 Raft 收票算法 |
| 重复增加票数、把旧请求换成当前轮次 | 两个独立 fixture 可编译，Rust 指定断言失败、Verus 指定后置条件失败；不是编译错误 |
| 无远端实际输出却注入回复 | 真实 Rust 来源记录检查和固定 Verus enabled 断言均拒绝；此 Verus 校准单独使用 root 过滤，不是候选证明。正向控制允许已记录的错误输出存在，不以 Raft 正确性筛掉它 |
| 离线系统回归 | 109 项通过；涉及 agent/checker 的替身均为 SIMULATED，不作形式化证明证据 |

首轮 [check-01](check-01/) 另运行了已有 13 项真实通用 Rust/Verus 回归；其新生成的临时输出已从旧维护 runs 路径移到 target，本轮可读依据见 [support-tools](check-01/support-tools/LOCATION.md)。最终维护入口把离线回归与本任务真实工具检查分开，不再向旧 runs 写维护产物。

待语义审查的明确选择包括：保留固定成员编号范围与每个接收决定都保存的策略；Write.request 改为 Option，为更高回复任期增加真正的 Persisted 完成结果，不虚构请求；计票是易失集合，普通同任期接收保存可保留集合但阻止新票处理；合法更高请求或回复遇到 ID 耗尽时，更新易失任期并转 Follower、清票、停机，不假称已保存。新字段和规则只作用于新任务，旧 election-start/request-vote 输入未改。这些 DRAFT 选择等待人工审核，未标成已批准。

宿主的原 RPC 关联、地址映射、串行接线、真实 I/O、如实完成和读回、文件保留、转发与取消仍 **UNVERIFIED**。新无重启跨节点来源证明与旧两类窗口的本地恢复证明是不同目标，不能相加成集群任意崩溃安全性。正确候选尚未生成；实际收票正例、来源归纳、恢复组合及全部候选证明均待实验 agent 完成。全局五项安全性、完整 2A/MadRaft 和条件性活性保持 OPEN。

在仓库根目录复现维护检查，输出目录必须未使用：

```bash
python3 tasks/vote_collection/maintain.py --output review/vote_collection/check-next
```

全新实验的预览命令已经实际运行，未创建候选或会话。审核后可用第二条命令启动一次新实验；新输入不含旧候选、复核副本、fixture 或维护证据：

```bash
python3 scripts/run.py new runs/vote-collection-01 --task vote-collection
python3 scripts/run.py new runs/vote-collection-01 --task vote-collection --execute
```

本轮到此停止，等待人工审核。

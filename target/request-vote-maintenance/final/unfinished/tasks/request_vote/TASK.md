# RequestVote 接收端 — DRAFT

最终研究目标仍是生成完整 MadRaft 并证明五项安全性。本次只生成真实接收端代码及局部证明，不完成候选者计票、多数派或 Leader 晋升。单节点同任期不改投不是整个集群 Election Safety。

## 固定行为

规则依据 [Raft Figure 2 与 §5.4.1](https://raft.github.io/raft.pdf)。低任期请求拒绝并返回当前任期。高任期先更新任期、清除旧投票、转 Follower，即使日志过旧而拒绝也必须保存新任期。同任期只可投给尚未投票的候选者或已经投过的同一候选者；后者可重复获票。日志新旧先比最后任期，再比最后索引。合法、空闲、操作 ID 可用时必须形成规定决定；匹配的成功保存必须返回规定回复，不能总拒绝或不返回。

`api.rs` 是固定观察接口和纯规格，不是执行算法。`Core::receive` 决定状态、保存记录及待授权回复，`Core::complete` 检查操作身份和真实完成通知后授权输出；真实 MadRaft handler 只转接这两个函数。候选自由选择内部表示和证明，但独立 caller 会重新断言固定规格，不以候选自报的覆盖表验收。

本次采用简单的保存策略：每个被接受的合法请求都发出一次 Save，包括状态未变的拒绝；这会增加 I/O，但保证所有依赖状态的回复在相应保存成功后释放。Save 不等于已经返回回复。Busy 保留 Pending；不匹配或重复完成不释放回复；匹配失败停止接收端，不发协议回复。操作 ID 单调分配，合法范围 `0..u64::MAX` 不含 MAX；恢复 MAX−1 后 next=MAX，拒绝后续请求。Raft term 与操作 ID 无关，任期与索引允许 MAX。

成员固定为 `[0,members)`；配置须非空且包含本节点。请求 term=0、非法成员、末条 term/index 中仅一个为零、末条任期大于请求任期均显式 Invalid。持久状态同样检查日志组合、日志任期不超过当前任期、投票成员合法且投票任期非零。优先次序 Halted、Busy、Invalid、Exhausted，错误不改状态。本任务不处理拜占庭伪造或证明成员认证；RPC 请求者身份与 candidate_id 一致属于非拜占庭宿主约定，重复投票回复返回原 RPC 请求者。文件编码长度/标签错误是 I/O 错误，绝不能隐式初始化。

非空日志由明确的合法初始化/保留记录 fixture 提供，接收端不得永久假设空日志。真实复制日志到该摘要的对应仍 OPEN；没有完成 AppendEntries。

## 四项独立证明

1. `rules`：实际构造、恢复、receive、complete、inspect 的前后状态和输出符合固定关系；无需调用者预先保证请求合法。`nonempty` 必须实际构造并获得一个有效赞成回复。
2. `history`：固定 caller 从实际构造函数开始遍历任意有限 Receive/Complete 序列，按每次实际返回记录全部回复。对其中任何两条同节点、同任期的赞成回复，candidate 必须相同。`replies` 是不可漏记的观察接口，其连接由固定 caller 检查；具体归纳不变量由你实现。
3. `confirmed`：实际 receive 产生记录，成功 complete 产生赞成回复且宿主已送达，之后无新接受操作；从同一记录恢复，运行一般有限后续调用，前后赞成历史合并仍唯一。
4. `pending`：实际 receive 产生记录，宿主已写成功，尚未 complete、尚未回复且无新接受操作；从同一记录恢复，继续一般有限调用。不得虚构旧 complete 或旧回复。恢复本身建立空本次输出历史、Follower、保留 term/vote/log、next=op+1、无 Pending；同任期 B 不能获票，A 可合法重试。

独立恢复入口的 readback 与实际 Write 相同，是公开的真实 I/O/文件保留/解码前提。Verus 验证的是在该前提下实际 recover 及后续调用，不是把成功标志变成外部 I/O 的证明。前缀状态由同一个实际 Core 及其不变量约束，`history` 另外证明它可由构造及任意有限前缀调用产生。

## 宿主与验收

候选准备只新增未实现 Core/lemma 入口，接到原 `RaftHandle` 和注册的 RequestVote RPC；协议之外的学生 TODO、原始测试、全量性质均保留。`host.rs` 与少量 handler 接线未验证：负责操作串行化、真实 MadSim 文件操作、真实结果回传、编码及原样转发授权回复。测试屏障只控制时序/观察；失败注入使用真实只读文件写错误。固定测试使用实际 RPC、独立文件字节断言、同节点 kill/restart 和全新 Core/端点。

`safety` 只运行局部 Verus；`feedback`/`obligation` 为局部功能和证明保留各自时间；`final` 对同一停止候选运行所有局部测试、全部配置种子和四个独立入口。仅全部通过才能标记 `REQUEST_VOTE_CHECKS_PASSED`，报告仍保留五项全局安全性 OPEN、完整 MadRaft INCOMPLETE、条件性活性 OPEN。测试通过、局部 verified 或 agent exit=0 均不单独结束成功。

本次不证明 create/write 中断、写失败后重启、任意崩溃历史、其他修改 term/vote 的路径、完整日志对应或异步宿主。这些限制只属于本任务，不修改主环境要求。不要消除公开宿主边界来无限扩展任务。

## 输入来源

本任务从根 MadRaft 学生骨架和固定 requirements 新建，不读取旧 runs。四个 `labs` 的方法只用于维护设计：同源核心、Pending/完成、已确认和已写未确认的有限恢复；本次不复制其证据、会话或 Raft 解法，也不要求阅读论文寻找实现方案。代码与证明留给本次实验自行生成。

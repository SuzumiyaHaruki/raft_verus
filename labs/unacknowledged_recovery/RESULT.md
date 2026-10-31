本例只新增一个重启窗口：最后接受的 Put 已完成真实文件写入、调用返回 Ok，但尚未调用 `Core::complete`、尚未生成这次 Ack 时 kill；没有后续接受操作、在途文件调用或并行写入者。**不支持写调用中断、create/write 中间崩溃或失败后恢复，也不声称通用崩溃一致性。** 前三步的范围和主 Raft 环境要求没有改变。

本轮验收完成，无工具阻塞或未闭合的本轮核心义务。仅新增本目录。复现命令在仓库根目录执行，证据默认保存到新的时间戳目录：

```bash
python3 labs/unacknowledged_recovery/check.py
```

[src/core.rs](src/core.rs) 复用第三步核心，除来源注释外，执行代码、合同和历史表示逐字节相同；没有重写 recover。Cargo 与 Verus 检查本目录的同一份代码，快照与变异包含完整核心。`recover` 从合法记录建立 value、next=id+1、空 Pending 和类型不变量；MAX 非法，MAX−1 可恢复但随后耗尽，不回绕。

新增的 [proof.rs](proof.rs) 不调用第三步的已确认恢复定理。`begin_then_recover` 真实调用 begin，证明新 Pending 与实际 Write 对应、finished 历史未变；`recover_written_pending` 在“读回记录对应这次 Pending 写入”的公开 I/O 前提下调用真实 recover，推出恢复值等于请求／记录值，next 大于旧运行全部已分配 ID，包括尚未确认的 ID。`written_restart_with_suffix` 接上任意有限后续调用；`old_notice_is_rejected` 排除旧编号成功或失败通知对值、Pending 和 Ack 的影响。`witness_seven_then_nine` 真实调用 new、op0 的 begin/complete、op1 的 begin 与 recover，机器检查旧值 7、Pending 值 9、新值 9，旧 finished 中只有 op0 的 Ack，没有为 op1 虚构 complete。

恢复基值与本次运行历史仍然分开：恢复后的 accepted/finished 为空，不制造旧请求或回复。字段名 `completed` 在恢复起点表示保留记录的值，**不是客户端已收到确认的证据**。所以旧内存为 7、成功写入文件为 9 时，恢复 9 是本例选定的正确语义；没有 Ack 不意味着写入没有发生。这是单节点存储语义，不是 Raft 多数派提交、应用或客户端确认规则。

[适配器](src/adapter.rs) 的实际顺序为写入返回 Ok → 写后屏障 → complete → 发送 Ack。[集成测试](tests/service.rs) 先让 A 收到 op0/7 的 Ack，再让 B 提交 op1/9；屏障报告旧核心为 7/next2/Pending=op1，测试另行读取真实文件并断言 op1/9。B 始终有接收任务；测试不释放屏障，直接 kill，等待旧服务取消及析构、确认屏障接收端销毁，再用同一节点 ID 的 restart、新句柄及新网络端点启动 Recover。新服务仅接收路径和模式，没有旧 Core、旧值或测试答案；读回文件后恢复为 9/next2/空 Pending，不补发 Ack9。之后 A 的 Put11 正常使用 op2，收到 Ack11，并独立读到文件 op2/11。

未生成 Ack 的核心结论来自 begin 保持完成历史不变及 Pending 不变量；异步程序没有越过 complete 的位置由屏障接线审查和测试支持。B 的有界无回复观察仅说明该运行窗口内未接收 Ack，不能单凭它区分网络尚未投递与核心没有生成动作，也不能替代一般历史证明。测试持续接收，避免“客户端没读取”掩盖回复。

| 实际检查 | 结果 |
| --- | --- |
| 正确核心与新组合证明，`--no-cheating` | `25 verified, 0 errors`；最终复查相同 |
| 正向测试，seeds 1、7 | 每个种子 4 个测试通过，包含真实中断／恢复、旧通知、Busy、失败、重复完成、耗尽和恢复输入边界；最终分别重跑通过 |
| 核心把恢复 next 改成 record.id | Rust 编译成功；Verus `24 verified, 1 errors`，在恢复构造的类型不变量处拒绝，退出 1 |
| 宿主 Recover 忽略文件、直接 new | 核心证明仍通过；真实恢复断言失败，实际值 0、期望 9，退出 101 |
| 屏障错误移到写入前 | 核心证明仍通过；独立文件断言失败，实际 op0/7、期望 op1/9，退出 101；没有把超时当检出 |

[原始验收记录](evidence/acceptance/results.json) 保存全部 15 次真实调用的命令、版本、种子、耗时、退出码及完整 stdout/stderr 路径；没有未运行项。`evidence/acceptance/inputs/` 为最终源码、证明、测试和检查器输入，三个 `.patch` 为仅改执行语句的负例，各副本独立 Cargo target。正确源码未被变异修改，最终再次逐字节核对。检查器按指定测试和错误诊断验收，不固定 verified 数；缺工具、超时、缺项、非预期失败或源码改变均非零退出。`evidence/development/` 保留首轮真实证明和测试输出，没有模拟检查。

公开可信边界仍是：网络来源与编解码正确、宿主确实写入指定 ID/value 并如实报告结果、文件在所选 MadSim kill/restart 后保留且如实读回、适配器在正确位置调用并原样发送。操作 ID、屏障信号或 Ok 标志本身不是外部 I/O 的证明。已重新核对锁定 MadSim 0.2.34 / `519950efb4711464f300ed7edf2967ed62d5f502` 的实际源码，与第三步的 174 行相关摘录一致：kill 取消任务和端点，文件系统 power_fail 仍是 TODO，create 会截断文件，不能推导物理掉电持久性。详见[检查依据](evidence/inspection.log)及[既有源码摘录](../quiescent_recovery/evidence/madsim-api.txt)。工具仍是 Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05。

[保留记录](evidence/preservation.log) 比较了修改前的 5079 个文件，前三个 lab、主项目所列目录及已追踪旧实验均未改变；依赖版本、主控制器、输入清单、模型和 7200 秒预算不变，新 lab 不进入实验输入。工作区 `.git` 仍不可用，未假称核对了其 HEAD；只读参考检出为 `6afbd1f0104fa8e9926034dbb5f2843c55d1b13c`，没有用于回退或覆盖。

更一般的崩溃／失败恢复、物理断电、WAL、多文件、并行写、完整异步验证、客户端去重、活性及 Raft 均未建立。本例不计为实验 LLM 自主生成成功，未调用 agent、升级工具、提交或推送。已在本轮验收范围内停止，等待人工审核。

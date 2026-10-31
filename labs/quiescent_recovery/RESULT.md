本例仅支持：最后一个被接受的 Put 已真实写入成功、客户端已经收到 Ack，之后没有新被接受操作、在途写入或其他文件写入者时终止服务器。**这不是通用崩溃一致性证明。** 失败操作也消耗 ID，因此 Pending 为空不足以允许恢复；确认 7 后，在覆盖为 9 的 create/write 之间崩溃，旧文件可能已经被截断，这种执行不在结论内。

本轮达到附件的完成标准 A。只新增本目录，未修改已有两个 lab、学生骨架、主要求、控制器、实验输入清单、模型或 7200 秒预算，未编辑旧实验，未调用 agent、提交或推送。

[src/core.rs](src/core.rs) 是 Cargo 实际运行且 Verus 检查的同一份核心。`Core::recover` 拒绝 ID=MAX；合法记录建立 completed=value、next=id+1、空 Pending 和类型不变量。恢复 MAX−1 后 next=MAX，后续请求明确耗尽，不回绕。历史分为恢复依据和本次真实调用；恢复时 accepted/finished 均为空，没有虚构旧请求、客户端地址或 Ack。`begin/complete` 保留正常接受、Busy、失败、错配、重复通知及匹配成功必须回复的规则。

[proof.rs](proof.rs) 独立调用实际入口：`finite_calls` 检查任意有限 Put/Complete 序列；`recover_retained_record` 从最后接受操作的真实历史关系和记录 ID/value 对应，推出恢复值等于旧完成值、next 大于旧运行全部已分配 ID；`one_confirmed_restart` 组合实际 new、begin、complete、recover 及任意有限后续调用。`old_notice_is_rejected` 证明低于恢复起始 next 的旧成功或失败通知不能回复、改值或清除新 Pending。`failure_is_not_a_restart_point` 明确证明“成功后又有失败”的空 Pending 不满足本轮恢复条件。这些调用者证明不执行文件 I/O；记录确实来自最后成功写入、客户端已收到 Ack，仍是公开宿主前提。

[src/adapter.rs](src/adapter.rs) 的 Recover 模式只从真实文件读取 16 字节记录，再调用已验证恢复入口；缺失、读取错误、非法长度和非法 ID 返回错误，不回退到 new。Fresh 模式显式区分并拒绝已有文件。适配器、编解码、网络来源及转发未获机器证明：如实执行写入、报告结果、读回保留内容、调用正确入口和原样发送，仍是可信接线职责。

已先核对锁定 MadSim 0.2.34 / `519950efb4711464f300ed7edf2967ed62d5f502` 的[实际源码摘录](evidence/madsim-api.txt)：kill 取消任务并重置网络端点；文件系统 reset 调用的 power_fail 仍是 TODO，因此本版本保留已写内容；create 会截断文件。测试等待旧服务取消和析构，使用同一节点 ID 的 restart、新句柄与新端点，真实读回原文件，没有复制到新节点或把旧 Core 带入恢复。Rust/Cargo 1.92.0、Verus 0.2026.01.23.1650a05 均复用现有工具。

| 真实检查 | 结果 |
| --- | --- |
| 正确核心与独立历史／组合证明 | `27 verified, 0 errors`；最终复查相同 |
| 正确版本 seeds 1、7 | 每个种子 4 个测试通过；最终分别再跑，仍全部通过 |
| 恢复值错误置 0 | Rust 编译成功；Verus `26 verified, 1 errors`，拒绝构造值的类型不变量 |
| 恢复 next 错误置 0 | Rust 编译成功；Verus `26 verified, 1 errors`，拒绝构造值的类型不变量 |
| 宿主忽略文件、直接 new | 核心证明仍 `27 verified, 0 errors`；真实重启测试在恢复值断言失败：实际 0，期望 9，退出 101 |

[测试](tests/service.rs) 实际执行 A/7→文件 op0/7→Ack、B/9→文件 op1/9→Ack→停收请求→kill/旧任务销毁→同节点 restart→读回 op1/9→恢复 9/next2→受控等待中的 op2→写入/Ack11→独立读取 op2/11。通知与屏障确定重启点及写入放行；客户端有限无回复观察不替代一般历史证明，也不引入协议超时假设。另测旧／错配／重复完成、失败消耗 ID、Busy、MAX 边界及恢复模式输入错误；大 ID 使用单独 fixture，没有宣称实际执行海量请求。

[验收记录](evidence/acceptance/results.json) 包含 15 次真实工具调用的命令、种子、版本、退出码、耗时和 stdout/stderr 路径，全部符合各自预期，没有跳过或阻塞，总工具耗时约 52.46 秒。`evidence/acceptance/inputs/` 保存最终输入，三个 `.patch` 保存仅修改执行语句的反例；各变异使用独立临时工作区和 Cargo target，规格、调用者与测试保持不变，正确源码字节也未被变异修改。verified 数与命令数不是性质覆盖率。`evidence/development/` 保留首次组合证明的失败及修复后输出。

在仓库根目录复现，默认将本次证据写入新的时间戳目录；缺工具、超时、未执行或错误类型不符合预期均返回非零：

```bash
python3 labs/quiescent_recovery/check.py
```

[保留检查](evidence/preservation.log) 对修改前副本逐字节比较，前两个 lab（含证据）和主项目所列文件均未变，依赖锁定仅更名新 lab 包。当前环境将工作区 `.git` 暴露为空目录，故无法取得其 HEAD 或未提交状态；只读核对的既有临时发布检出为参考提交 `517e3f4eede9d77d2090a2d2734f64efcca385ad`，未以它覆盖当前文件。

尚未建立任意写入中断／失败后的恢复、真实掉电持久性、完整异步服务证明、客户端去重或活性；没有修改主实验更广的故障范围，也不将此校准记为 Raft 证明或实验 LLM 自主生成成功。到此停止，等待人工审核。

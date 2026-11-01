# election-start 验收修复与原候选复核

本轮维护完成，没有启动模型生成、Codex CLI 或其他 agent，没有提交、推送或修改模型、工具链、7200 秒预算。原 `election-start-01` 仍是旧输入下的 **INCOMPLETE：64 轮、6336.720842677 秒**。本次仅为更正固定验收后的复核，不计作另一次生成成功。

工作区 `.git` 为空，无法直接读取当前 HEAD；维护前实际文件与可读的参考 checkout `490f1a0971851da53ca044e2a0a0fc6b3938d9ab` 的 Git 跟踪文件一致。见 [before.json](before.json)。未回退版本。

**固定测试。** 原测试在 abort/join 唯一发送任务后，对空关闭通道执行 recv，真实 MadSim 返回 `Ok(Err(RecvError))`，不是超时。现在客户端观察任务跨越服务器 kill/restart 窗口存活，旧服务器仍必须释放全部 Core 引用；收到回复、收到 RPC 错误、通道关闭都会使观察断言失败。只在观察结束后取消客户端任务，并单独检查取消与空通道关闭。恢复额外检查 Follower，保留真实文件、next、空 Pending、竞争者拒绝和原候选者重试要求。

测试还在存活客户端注册 MadSim 自带的回复钩子，记录实际到达的 RequestVote 回复并始终返回 true，原样放行数据包。它不引用 Core、不改变协议、不新增协议路径或等待点。重启后的合法重试必须实际触发该钩子，避免使用失效观察器。正常宿主 `host.rs`、`driver.rs`、`prepare.py` 与候选运行源码均未改动。

**真实检查。** 使用锁定的 Rust 1.92.0、MadSim 0.2.34 / `519950efb4711464f300ed7edf2967ed62d5f502` 和 Verus 0.2026.01.23.1650a05。完整命令、环境、退出状态及 stdout/stderr 保存在 [recheck-01](recheck-01/)。

| 检查 | 实际结果 |
| --- | --- |
| 构建、测试清单 | PASS |
| 原有 16 项阶段测试 × seed 1–5 | 80/80 PASS |
| 新增 4 项观察器自检 × seed 1–5 | 20/20 PASS |
| rules / campaign / authorization / history | 分别 36 / 38 / 38 / 39 verified，均 0 errors |
| receive_confirmed / receive_pending / self_confirmed / self_pending | 每项 40 verified，均 0 errors |
| 隔离宿主在恢复时发送未请求的旧回复 | 两个恢复窗口都以指定的实际回复断言拒绝，exit 101；编译成功 |
| 相同错误宿主副本的核心 history 入口 | 仍为 39 verified、0 errors，说明核心证明不能替代宿主测试 |
| 最后正确版本复查，独立构建目录 | 再次通过 100 项种子测试和全部 8 项证明 |

全部正式证明保留 `--no-cheating`，没有使用 `--verify-root` 或 `--no-verify`。各入口验证数量有重叠，不能相加当成新的独立性质。四个小自检覆盖存活无输出、关闭空通道、回复排队后关闭、真实传输错误及异常任务终止；其原始输出见 [observer-cases.log](recheck-01/observer-cases.log)。这些自检自身不是完整 RPC 恢复测试。

宿主负例在独立副本从恢复状态构造旧回复，通过真正的 MadSim Endpoint 发送给原请求节点；正常与错误路径使用同一个测试观察器。负例为构造 MadSim 的 wire payload，临时显式引用锁文件中已有的 bytes 依赖；仅该副本的宿主与 Cargo 声明改变，核心和证明未改。见 [负例补丁](recheck-01/host-unsolicited-reply.patch)、[已确认窗口日志](recheck-01/bad-host-restart_confirmed.log)和[写成功未完成窗口日志](recheck-01/bad-host-restart_written_pending.log)。

**控制器。** `scripts/check.py` 记录精确失败测试；`scripts/request_vote.py` 从构建、清单、具体测试、源边界和证明中选择真实阻塞，保留所有其他结果。已运行的必要功能检查失败时不启动 final。`scripts/run.py` 从正确位置读取 scoped PROGRESS，按检查依赖及配置区分实质修改与报告/未引用复现文件；稳定失败签名保留 seed 与关键断言，去除临时路径等噪声。同一输入与失败下，最多两个无实质变更交接后停止 NEEDS_REVIEW，并明确标注复用既有检查；续跑也不重跑同一失败 final。实现、证明依赖、固定材料、工具版本或参数改变后需要重检。依赖分析是保守的调度辅助，不是程序对应证明；遇到无法解析的模块或动态 include 时扩大到全部候选文件，可能保守地多查。

固定输入异议通过现有 PROGRESS 中一个 JSON 区块提交，包含固定文件、失败项、冲突、证据/复现、未应用建议、候选版本和未解决事项。控制器不执行 agent 提供的 shell，只独立重跑已知固定失败项；每次实验最多一次、最多 60 秒且计入累计预算，次数在执行前持久记录。复现失败后停止 `NEEDS_REVIEW / suspected_fixed_input_defect`，这仅表示需要维护审查，不表示测试已被裁定错误。证据不足会反馈一次具体缺口；仍无实质变化则停止。诊断超时或预算不足记录 inconclusive，不能免除验收。

[system-regression.log](system-regression.log) 中 **100 项离线回归通过**，覆盖 full、request-vote、election-start、保护边界、实际代码变更后的重检、报告/未引用复现文件不刷新停滞、诊断限额及累计成本。这里的 agent 与检查结果明确是 SIMULATED；真实子进程传输测试不调用模型。这些回归不能作为 Rust/Verus 成功证据，真实结果是上表的单独执行。

**来源与边界。** 新固定材料版本为 `madraft-election-start-2`；[provenance.json](recheck-01/provenance.json) 分别保留旧输入身份和新 authority 身份，`generation_performed=false`。复核副本的全部 43 个候选文件与原候选逐字节一致，原实验历史未改变；见 [preservation.json](preservation.json)、[固定材料补丁](recheck-01/fixed-input.patch)和[维护源码补丁](maintenance.patch)。根学生骨架、主 requirements、四个 labs、原候选、固定 api/caller 语义均保持不变。构建缓存只在忽略的 target 下，临时开发副本已清理。

异步宿主的串行接线、如实 I/O、编码、文件保留、成员地址映射、原样转发，以及 MadSim 观察钩子的实现仍是未验证前提。有限窗口无回复只是该执行的测试证据；它不证明任意异步执行、真实掉电持久性或任意崩溃恢复。本次未改变协议语义；观察边界和控制器停止策略等待人工审核。五项全局安全性、完整 MadRaft/2A、计票/晋升及条件性活性继续 OPEN。

复现命令如下。复核输出目录必须未使用；不需要启动实验 agent。

```bash
cd /home/nitro/Desktop/raft_verus
PYTHONPATH=scripts/tests python3 -m unittest test_checks test_helpers test_obligations test_request_vote test_election_start test_review_stop -v
python3 tasks/election_start/maintain.py --review-candidate runs/election-start-01 --output review/election_start_test_fix/recheck-02
```

已停止，等待人工审核，不预先实现下一阶段。

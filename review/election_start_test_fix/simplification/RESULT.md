# 控制器精简

本次只精简系统代码。`scripts/run.py` 从 1093 行减至 1066 行；`check.py` 从 725 行变为 738 行；`request_vote.py` 从 88 行减至 84 行。三个生产文件合计净减 18 行，没有新增生产模块。另为关键行为补充 3 项回归测试，测试文件增加 36 行。

依赖扫描统一复用 `module_paths`，未知模块或动态导入仍保守使证据失效，二进制数据依赖无需按文本解析。正式检查和固定输入异议诊断统一使用 `Checks.verus` 构造命令，目标仍由固定任务选择，不增加函数过滤。共用反馈前缀、集中最终检查和停滞退出，删除重复的审查状态身份；保留累计预算、具体失败、两次无进展停止及相同候选的审查停止状态。

实际运行结果：

- [离线系统回归](system-regression.log)：103 项通过，18.291 秒。涉及 agent 的控制器测试使用 SIMULATED 替身，不是生成实验。
- 真实 Verus 命令复查：固定 `rules` 分支 36 verified、0 errors；`campaign` 分支 38 verified、0 errors。使用既有审查副本的逐字节相同临时副本，没有修改候选代码或证明。原始日志为 [rules](rules-verus.log)、[campaign](campaign-verus.log)，命令及环境见 [verus-smoke.json](verus-smoke.json)。这两项只检查共用命令入口，不替代原验收或建立全局 Raft 正确性。
- 对照本轮修改前内容确认：`madraft/`、`requirements/`、`tasks/`、`labs/`、`runs/election-start-01/`（含历史记录）和 `tools.ini` 未改动。没有调用实验 agent、提交或推送。

离线回归复现命令（仓库根目录）：

```bash
PYTHONPATH=scripts/tests python3 -m unittest test_checks test_helpers test_obligations test_request_vote test_election_start test_review_stop -v
```

原维护报告和原验收证据保留；本目录记录其后的代码精简检查。精简幅度有限，没有通过压缩格式、删除安全检查或搬移到新模块来减少 `run.py` 行数。

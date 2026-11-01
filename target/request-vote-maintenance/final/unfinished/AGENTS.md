# MadRaft RequestVote 接收端生成任务 — DRAFT

阅读 `tasks/request_vote/TASK.md`、固定 `api.rs`、`caller.rs` 与 `rpc_tests.rs`。补全候选 `madraft/src/raft/vote_core.rs` 和 `madraft/proof/vote_lemmas.rs`，可以增加自己的内部表示、辅助规格、不变量和引理。实现与证明一起生成，根据真实检查修复；不要导入其他实验的代码、补丁或会话。

实际调用路径为 `RaftHandle::new → start_rpc_server → RaftHandle::request_vote → Raft::request_vote → Core`。不要另建服务，不把协议判断移进未验证宿主，不改固定目标、测试或检查器。不得用 assume、admit、external_body、跳过验证或不终止实现代替证明。不要证明 RPC 内部或异步执行器；公开的宿主前提保留。

先实现一个实际行为及其前置条件、输出和证明，再逐步闭合固定的接收规则、一般有限历史和两个恢复入口。局部通过不等于 Election Safety；五项全局性质、全量功能及条件性活性仍 OPEN。其他学生 TODO 保留。

用简短 `PROGRESS.md` 记录当前失败、尝试、检查和下一步；用 `RESULT.md` 分开写四项局部证明、RPC/文件测试、宿主边界和全量 OPEN。遵守本次提示的软交接期限，正常返回后控制器会独立检查同一候选并反馈。不要改全局工具或认证配置，不提交或发布。Markdown 段落不要段内换行。

```bash
python3 scripts/check.py obligation --task request-vote --timeout 120
python3 scripts/check.py safety --task request-vote --timeout 240
python3 scripts/check.py feedback --task request-vote --timeout 300
python3 scripts/check.py final --task request-vote --timeout 600
```

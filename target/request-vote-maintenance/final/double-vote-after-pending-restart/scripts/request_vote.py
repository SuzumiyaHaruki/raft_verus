"""One bounded receiver task, using the existing snapshot/check/controller machinery."""
from pathlib import Path
import importlib.util
import json
import re
import time

INPUTS = ('AGENTS.md', 'TASK.md', 'start.md', 'continue.md', 'redesign.md',
          'api.rs', 'caller.rs', 'core.rs', 'lemmas.rs', 'host.rs', 'rpc_tests.rs', 'prepare.py')
GOALS = ('rules', 'history', 'confirmed', 'pending')
LABEL = 'REQUEST_VOTE_CHECKS_PASSED'
BOUNDARY = ('局部接收端检查不等于全局 Election Safety。宿主串行化、实际 I/O、编码、文件保留、'
            '通知真实性及向原 RPC 请求者转发仍未验证；只覆盖两种指定恢复窗口。')
TEST_FILTER = 'raft::raft::vote_tests::'


def prepare(authority):
    spec = importlib.util.spec_from_file_location('vote_prepare', Path(authority) / 'tasks/request_vote/prepare.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.prepare(authority)


def evaluate(checks, mode, seeds):
    """All required results bind to the existing stopped-source snapshot."""
    from check import SAFETY, source_links
    from inputs import save_json
    text = (checks.authority / 'tasks/request_vote/rpc_tests.rs').read_text()
    checks.expected = {TEST_FILTER + n for n in re.findall(r'#\[(?:madsim::)?test\]\s*(?:async\s+)?fn\s+(\w+)', text)}
    if mode != 'safety':
        checks.phase_deadline = min(checks.deadline, time.monotonic() + (checks.deadline - time.monotonic()) * 0.4)
        checks.cargo('build')
        checks.cargo('inventory')
        checks.test_plan = [TEST_FILTER]
        for seed in seeds if mode == 'final' else seeds[:1]:
            checks.cargo('test',seed,TEST_FILTER)
    checks.phase_deadline = checks.deadline
    caller = checks.snapshot / 'tasks/request_vote/caller.rs'
    _, outside = source_links(caller,checks.snapshot)
    checks.record('source_boundary','NEEDS_REVIEW' if outside else 'PASS',
        'external_source_import' if outside else 'same_candidate_imports', external_paths=outside)
    for goal in GOALS:
        argv = [checks.cfg['verus'],str(caller),'--crate-type','lib','--cfg','madsim','--no-cheating','--triggers-mode','silent']
        if goal != 'rules': argv += ['--cfg','rv_' + goal]
        checks.run('vote_' + goal,argv,proof=True)
    first = next((g for g in GOALS if checks.results['vote_' + g]['status'] != 'PASS'), None)
    # This small fixed dependency chain is metadata for existing failure history,
    # not a candidate-selected proof target or another task scheduler.
    goal = first or 'rules'
    entry = 'madraft/src/raft/vote_core.rs' if goal == 'rules' else 'madraft/proof/vote_lemmas.rs'
    item = checks.results['vote_' + goal]
    checks.current = dict(plan=dict(id='request-vote-' + goal,goal='request-vote-' + goal,
        statement='Close the fixed receiver ' + goal + ' caller',kind='composition',entry=entry,
        functions=['madraft/src/raft/vote_core.rs::receive','madraft/src/raft/vote_core.rs::complete'],
        dependencies=[],premises=BOUNDARY),units={'vote_' + goal:entry},
        layer='LOCAL_UNIT_CHECKED' if not first else 'PREPARATION',
        first_blocker=dict(check='vote_' + goal,entry=entry,reason=item['reason'],log=item.get('log')) if first else
            dict(reason='run_all_local_tests_and_seeds'), imported_references=['madraft/src/raft/vote_core.rs'])
    required = list(checks.results)
    report = checks.finish(mode,required)
    report.update(task_scope='request-vote-receiver', full_madraft='INCOMPLETE',
        global_safety={p:'OPEN' for p in SAFETY},semantic_boundary=BOUNDARY,
        local_goals={g:checks.results['vote_' + g]['status'] for g in GOALS},
        host_verification='UNVERIFIED', uncovered=['arbitrary crashes','failed-write recovery',
            'other term/vote updates','real log-to-summary correspondence','candidate tally and majority'],
        task_completion='LOCAL_CHECKS_PASSED_REVIEW_REQUIRED' if report['status'] == LABEL else 'OPEN')
    save_json(checks.output / 'results.json',report)
    lines=['# RequestVote 局部检查','',report['status'],'',BOUNDARY,'',
        '| 项目 | 结果 |','| --- | --- |']
    lines += [f'| {n} | {i["status"]}: {i["reason"]} |' for n,i in checks.results.items()]
    lines += ['', '五项全局安全性全部 OPEN；完整 MadRaft INCOMPLETE；条件性活性 OPEN。', '']
    (checks.output / 'results.md').write_text('\n'.join(lines))
    return report

"""One bounded receiver task, using the existing snapshot/check/controller machinery."""
from pathlib import Path
import importlib.util
import json
import re
import time
from inputs import scoped_task

INPUTS = ('AGENTS.md', 'TASK.md', 'start.md', 'continue.md', 'redesign.md',
          'api.rs', 'caller.rs', 'core.rs', 'lemmas.rs', 'host.rs', 'rpc_tests.rs', 'prepare.py')
VERSION = 'madraft-request-vote-1'
DIRECTORY = 'tasks/request_vote'
HELPERS = ['scripts/request_vote.py']
TEST_FILES = ('rpc_tests.rs',)
PREFIX = 'vote_'
CFG = 'rv_'
SCOPE = 'request-vote-receiver'
FUNCTIONS = ['receive','complete']
UNCOVERED = ['arbitrary crashes','failed-write recovery','other term/vote updates','real log-to-summary correspondence','candidate tally and majority']
GOALS = ('rules', 'history', 'confirmed', 'pending')
LABEL = 'REQUEST_VOTE_CHECKS_PASSED'
BOUNDARY = ('Local receiver checks do not establish cluster Election Safety. Host serialization, real I/O, '
            'encoding, file retention, truthful completion and forwarding to the original requester remain unverified. '
            'Only the two specified recovery windows are covered.')
TEST_FILTER = 'raft::raft::vote_tests::'


def prepare(authority, directory=DIRECTORY):
    spec = importlib.util.spec_from_file_location('vote_prepare', Path(authority) / directory / 'prepare.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.prepare(authority)


def evaluate(checks, mode, seeds):
    """All required results bind to the existing stopped-source snapshot."""
    from check import SAFETY, source_links
    from inputs import save_json
    spec = scoped_task(checks.task)
    text = '\n'.join((checks.authority / spec.DIRECTORY / f).read_text() for f in spec.TEST_FILES)
    checks.expected = {spec.TEST_FILTER + n for n in re.findall(r'#\[(?:madsim::)?test\]\s*(?:async\s+)?fn\s+(\w+)', text)}
    if mode != 'safety':
        checks.phase_deadline = min(checks.deadline, time.monotonic() + (checks.deadline - time.monotonic()) * 0.4)
        checks.cargo('build')
        checks.cargo('inventory')
        checks.test_plan = [spec.TEST_FILTER]
        for seed in seeds if mode == 'final' else seeds[:1]:
            checks.cargo('test',seed,spec.TEST_FILTER)
    checks.phase_deadline = checks.deadline
    caller = checks.snapshot / spec.DIRECTORY / 'caller.rs'
    _, outside = source_links(caller,checks.snapshot)
    checks.record('source_boundary','NEEDS_REVIEW' if outside else 'PASS',
        'external_source_import' if outside else 'same_candidate_imports', external_paths=outside)
    for goal in spec.GOALS:
        argv = [checks.cfg['verus'],str(caller),'--crate-type','lib','--cfg','madsim','--no-cheating','--triggers-mode','silent']
        if goal != 'rules': argv += ['--cfg',spec.CFG + goal]
        checks.run(spec.PREFIX + goal,argv,proof=True)
    first = next((g for g in spec.GOALS if checks.results[spec.PREFIX + g]['status'] != 'PASS'), None)
    # Order is build/inventory/tests/boundary/proofs. Report an actual failure,
    # including its exact tests and seeds, even when every theorem passes.
    blocker = next(((n,i) for n,i in checks.results.items() if i['status'] != 'PASS'), None)
    # This small fixed dependency chain is metadata for existing failure history,
    # not a candidate-selected proof target or another task scheduler.
    goal = first or 'rules'
    entry = 'madraft/src/raft/vote_core.rs' if goal in ('rules','campaign','authorization') else 'madraft/proof/vote_lemmas.rs'
    item = checks.results[spec.PREFIX + goal]
    checks.current = dict(plan=dict(id=checks.task + '-' + goal,goal=checks.task + '-' + goal,
        statement='Close the fixed ' + checks.task + ' ' + goal + ' caller',kind='composition',entry=entry,
        functions=['madraft/src/raft/vote_core.rs::' + f for f in spec.FUNCTIONS],
        dependencies=[],premises=spec.BOUNDARY),units={spec.PREFIX + goal:entry},
        layer='LOCAL_UNIT_CHECKED' if not first else 'PREPARATION',
        first_blocker=dict(check=blocker[0],entry=entry,**{k:blocker[1].get(k) for k in
            ('reason','category','log','diagnostic','failed_tests','seed')}) if blocker else
            dict(reason='run_all_local_tests_and_seeds'), imported_references=['madraft/src/raft/vote_core.rs'])
    required = list(checks.results)
    report = checks.finish(mode,required)
    report.update(task_scope=spec.SCOPE, full_madraft='INCOMPLETE',
        global_safety={p:'OPEN' for p in SAFETY},semantic_boundary=spec.BOUNDARY,
        local_goals={g:checks.results[spec.PREFIX + g]['status'] for g in spec.GOALS},
        host_verification='UNVERIFIED', uncovered=spec.UNCOVERED,
        task_completion='LOCAL_CHECKS_PASSED_REVIEW_REQUIRED' if report['status'] == spec.LABEL else 'OPEN')
    save_json(checks.output / 'results.json',report)
    lines=['# ' + checks.task + ' scoped checks','',report['status'],'',spec.BOUNDARY,'',
        '| Check | Result |','| --- | --- |']
    lines += [f'| {n} | {i["status"]}: {i["reason"]} |' for n,i in checks.results.items()]
    lines += ['', 'All five global safety properties OPEN; full MadRaft INCOMPLETE; conditional liveness OPEN.', '']
    (checks.output / 'results.md').write_text('\n'.join(lines))
    return report

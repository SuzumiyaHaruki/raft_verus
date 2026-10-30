#!/usr/bin/env python3
"""A bounded generate/check/feedback loop in one fresh workspace and conversation."""
import argparse
import configparser
import difflib
from datetime import datetime, timezone
import fcntl
import json
import math
import os
import re
from pathlib import Path
import shlex
import shutil
import signal
import sys
import time
import uuid

from check import CONNECTIONS, FAMILIES, OBLIGATIONS, SEMANTIC_BOUNDARY, configured_seeds, current_obligation, evaluate, group_diagnostics, module_paths, Checks
from inputs import INPUT_VERSION, copy_files, differences, files_under, fingerprint, git_revision, input_files, protected_files, save_json
from process import run_process
from tools import ROOT, environment, settings

PROMPTS = ('start', 'continue', 'redesign', 'behavior')
FORMAT = 'madraft-generate-check-5'


def command(state, workspace, output):
    argv = [state['codex'], '--no-daemon', '--ask-for-approval', 'never',
            '-c', 'sandbox_mode="workspace-write"',
            '-c', 'model_reasoning_effort=' + json.dumps(state['effort']), 'exec']
    if state['thread_id']:
        argv += ['resume', state['thread_id']]
    return argv + ['--ignore-user-config', '--skip-git-repo-check', '--json', '--model', state['model'],
                   '--output-last-message', str(output), '-']


def control_path(root, workspace):
    return root / 'runs/.controller' / workspace.name


def prepare(root, workspace, state):
    control = control_path(root, workspace)
    control.mkdir(parents=True, exist_ok=False)
    workspace.mkdir(parents=True, exist_ok=False)
    authority = control / 'inputs'
    names = input_files(root)
    copy_files(root, authority, names)
    system = ['scripts/run.py'] + [f'scripts/prompts/{name}.md' for name in PROMPTS]
    copy_files(root, authority, system)
    raw_tools = (root / 'tools.ini').read_text()
    cfg = settings(root)
    resolved, _ = environment(root)
    for key in ('verus', 'rust_bin', 'rustup_home', 'cargo_home'):
        cfg['tools'][key] = resolved[key]
    with (authority / 'tools.ini').open('w') as file:
        cfg.write(file)
    copy_files(authority, workspace, names)
    # These ignore rules concern run output only; no protocol code is supplied.
    (workspace / '.gitignore').write_text('/runs/\n/madraft/target/\n__pycache__/\n*.pyc\n*.rlib\n*.rmeta\n')
    personal = Path(os.environ.get('CODEX_HOME', Path.home() / '.codex')).expanduser().resolve()
    home = workspace / 'runs/codex-home'
    home.mkdir(parents=True, mode=0o700)
    for name in ('auth.json', 'rules'):
        if (personal / name).exists():
            (home / name).symlink_to(personal / name)
    state.update(format=FORMAT, input_version=INPUT_VERSION, run_id=str(uuid.uuid4()),
                 workspace=str(workspace), source_root=str(root), git_revision=git_revision(root),
                 input_files=names, system_files=system, original_tools=raw_tools,
                 created_at=time.time(), history=[], in_flight=None, tools=None,
                 status='INCOMPLETE', reason='created', redesigns=0,
                 obligations={}, current_key=None, integration_wait=0, checkpoints=[], replayed=[],
                 guidance=state.get('guidance', 'B'), test_history={}, costs=[], interventions=[],
                 input_fingerprint=fingerprint({n: authority / n for n in names}),
                 prompt_version=fingerprint({n: authority / n for n in system}),
                 interrupted_budget_seconds=0)
    save_json(control / 'state.json', state)
    return control


def origin_changes(root, control, state):
    names = state['input_files'] + state['system_files']
    changed = differences(root, control / 'inputs', [n for n in names if n != 'tools.ini'])
    if (root / 'tools.ini').read_text() != state['original_tools']:
        changed.append('tools.ini')
    if input_files(root) != state['input_files']:
        changed.append('input file list')
    return changed


def probe_tools(root, workspace, state, output, timeout):
    cfg, env = environment(root)
    env['CODEX_HOME'] = str(workspace / 'runs/codex-home')
    commands = {
        'codex': [state['codex'], '--version'], 'codex_help': [state['codex'], '--help'],
        'exec_help': [state['codex'], 'exec', '--help'],
        'resume_help': [state['codex'], 'exec', 'resume', '--help'],
        'rustc': [str(Path(cfg['rust_bin']) / 'rustc'), '--version'],
        'cargo': [str(Path(cfg['rust_bin']) / 'cargo'), '--version'],
        'verus': [cfg['verus'], '--version'],
    }
    # Parse the actual invocation shapes with --help; neither command starts an agent.
    for name, thread in [('new_command_help', None), ('resume_command_help', '00000000-0000-0000-0000-000000000000')]:
        example = dict(state, thread_id=thread)
        commands[name] = command(example, workspace, workspace / 'runs/probe-unused.md')[:-1] + ['--help']
    deadline = time.monotonic() + timeout
    evidence = {}
    for name, argv in commands.items():
        left = deadline - time.monotonic()
        if left <= 0:
            return dict(status='BLOCKED', reason='tool_probe_timeout', evidence=evidence)
        result = run_process(argv, workspace, env=env, timeout=min(10, left))
        (output / (name + '.log')).write_text('$ ' + shlex.join(argv) + '\n' + result.stdout + result.stderr)
        evidence[name] = {'command': argv, 'output': result.stdout.strip(), 'exit': result.exit_code, 'process': result.status}
        if result.status != 'COMPLETED' or result.exit_code != 0:
            return dict(status='BLOCKED', reason='tool_probe_failed:' + name, evidence=evidence)
    required = {'codex_help': ('--no-daemon', '--ask-for-approval', '--config'),
                'exec_help': ('--ignore-user-config', '--json', '--output-last-message', '--skip-git-repo-check'),
                'resume_help': ('SESSION_ID', '--ignore-user-config', '--json', '--output-last-message')}
    for name, flags in required.items():
        missing = [flag for flag in flags if flag not in evidence[name]['output']]
        if missing:
            return dict(status='BLOCKED', reason='unsupported_cli_flags', missing=missing, evidence=evidence)
    version_file = Path(cfg['verus']).parent / 'version.json'
    if not version_file.exists():
        return dict(status='BLOCKED', reason='cannot_identify_verus_compiler', evidence=evidence)
    toolchain = json.loads(version_file.read_text())['verus']['toolchain']
    if toolchain.split('-')[0] not in evidence['rustc']['output']:
        return dict(status='BLOCKED', reason='rust_verus_compiler_mismatch', evidence=evidence)
    return dict(status='PASS', evidence=evidence)


def session_id(log, previous=None):
    if not log.exists():
        return previous
    for line in log.read_text().splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if event.get('type') == 'thread.started':
            found = event.get('thread_id')
            if previous and found != previous:
                raise ValueError('Resume unexpectedly changed the experiment session')
            previous = found
    return previous


def token_usage(log):
    """Preserve returned counters. CLI scope is not assumed to be per call."""
    events = []
    if log.exists():
        for line in log.read_text().splitlines():
            try: event = json.loads(line)
            except ValueError: continue
            usage = event.get('usage')
            if isinstance(usage, dict):
                events.append(dict(event=event.get('type'), counters={k:v for k,v in usage.items()
                    if k in ('input_tokens', 'cached_input_tokens', 'output_tokens', 'total_tokens') and isinstance(v, (int, float))}))
    return dict(raw_events=events, counter_scope='UNKNOWN', summed_tokens=None, dollar_cost=None)


def feedback(authority, state, report, mode, workspace, seconds):
    soft = max(0, seconds - state['handoff_seconds'])
    deadline = datetime.fromtimestamp(time.time() + soft, timezone.utc).isoformat(timespec='seconds')
    parts = [(authority / f'scripts/prompts/{mode}.md').read_text().strip(),
             f'Workspace: {workspace}. Soft handoff deadline: {deadline} ({soft:.1f} seconds from now). '
             f'Save code and PROGRESS.md, then return normally by this deadline. '
             f'The hard watchdog is {seconds:.1f} seconds; it is a last resort, not a planned handoff. '
             'There is no CLI pause signal. Stop starting long checks near handoff.',
             f'Cumulative budget remaining: {state["total_seconds"] - state["seconds_spent"]:.1f} seconds. '
             f'Round: {state["attempts"] + 1}/{state["max_rounds"]}.',
             'Diagnostics refer to the stopped candidate. Edit this workspace’s madraft/ files, not check snapshots.']
    if state.get('guidance', 'B') == 'B':
        parts.append((authority / 'scripts/prompts/behavior.md').read_text().strip())
    if report:
        parts.append('Independent candidate revision: ' + report['candidate_revision'])
        current = report.get('current_obligation', {})
        parts.append('Selected increment (candidate plan, not proof): ' + json.dumps(current.get('plan', {})))
        parts.append('First unfinished dependency: ' + json.dumps(current.get('first_blocker')))
        parts.append('Checked layer: ' + current.get('layer', 'PREPARATION'))
        parts.append('Next obligation: ' + next_obligation(state, report))
        groups = report.get('diagnostics', group_diagnostics(report['checks']))
        groups = sorted(groups, key=lambda g: not any(n.startswith('current_') for n in g['affected']))
        for group in groups[:6]:
            parts.append('Affected: ' + ', '.join(group['affected']) + '\n' +
                         json.dumps({k:v for k,v in group.items() if k not in ('affected', 'logs')}, ensure_ascii=False) +
                         '\nRaw logs: ' + ', '.join(group['logs']))
        parts.append('All outcomes (details in the independent results.json):\n' + '\n'.join(
            f'{name}: {item["status"]} — {item["reason"]}' for name, item in report['checks'].items()))
        for area in CONNECTIONS:
            item = report['checks'].get('connection_' + area, {})
            parts.append(f'connection_{area}: {item.get("status", "OPEN")}; local unit '
                         f'{item.get("unit_status", "NOT_RUN")}; outer paths {item.get("open_paths", [])}.')
        parts.append('Functional gaps: ' + ', '.join(report.get('placeholders', [])))
        parts.append('Available checkpoints of this run (historical checks, never current acceptance): ' +
                     json.dumps([{k:c[k] for k in ('id', 'layer', 'first_blocker')} for c in state.get('checkpoints', [])[-4:]]))
        key = state.get('current_key')
        if key:
            record = state['obligations'][key]
            parts.append('Current obligation history: ' + json.dumps({k:record.get(k) for k in
                ('aliases', 'stalled', 'redesigns', 'blocked', 'last_outcome', 'newly_checked')}))
        parts.append('Recent checked attempts and abandoned versions: runs/attempts.jsonl. Local check notes without independent replay are untrusted. Read relevant failures, not the entire experiment history.')
        parts.append('Final still requires every retained family/seed and all formal targets on one stopped candidate. ' + SEMANTIC_BOUNDARY)
    for record in state['history'][-2:]:
        parts.append(f'Previous round {record["round"]}: {record["process"]}; independent result {record["check_status"]}.')
        parts.append('Candidate-reported approach/dependencies (UNVERIFIED):\n' + record.get('progress', '')[-3000:])
    return '\n\n'.join(parts) + '\n'


def obligation_key(state, plan):
    """Keep aliases/representation revisions together; renaming an id is no reset."""
    entries = set([plan['entry'], *plan.get('dependencies', []), *([plan['parent']] if plan.get('parent') else [])])
    functions = set(plan['functions'])
    records = state.setdefault('obligations', {})
    for key, record in records.items():
        if plan['id'] in record['aliases'] or plan['entry'] in record['roots'] or (record['goal'] == plan['goal'] and functions.intersection(record['functions'])):
            break
    else:
        key = 'obligation-' + str(len(records) + 1)
        records[key] = dict(aliases=[], goal=plan['goal'], entries=[], roots=[], functions=[], versions=[],
                            units={}, stalled=0, redesigns=0, blocked=False, attempts=0)
    record = records[key]
    record['aliases'] = sorted(set(record['aliases']) | {plan['id']})
    record['entries'] = sorted(set(record['entries']) | entries)
    record['roots'] = sorted(set(record['roots']) | {plan['entry']})
    record['functions'] = sorted(set(record['functions']) | functions)
    content = {k:v for k,v in plan.items() if k not in ('id', 'rollback', 'restore')}
    if not record['versions'] or record['versions'][-1]['plan'] != content:
        record['versions'].append(dict(version=len(record['versions']) + 1, plan=content))
    return key


def record_attempt(state, report, *, advance=True):
    current = report.get('current_obligation', {})
    plan = current.get('plan', {})
    if not plan or plan.get('error'):
        state['current_key'] = None
        return dict(outcome='PREPARATION', reason=plan.get('error', 'missing_current_obligation'))
    key = obligation_key(state, plan)
    state['current_key'] = key
    record = state['obligations'][key]
    units = {entry: report['checks'][name] for name, entry in current['units'].items()}
    passed = record.setdefault('passed_units', [])
    def contents(snapshot, entry):
        if not snapshot:
            return None
        root = Path(snapshot)
        paths = module_paths(root / entry, root)
        return sorted(p.read_bytes() for p in paths if p.is_relative_to(root) and p.is_file()) or None
    newly = []
    for entry, item in units.items():
        if item['status'] != 'PASS' or any(old['entry'] == entry for old in passed):
            continue
        source = contents(report.get('snapshot'), entry)
        if source and any(source == contents(old.get('snapshot'), old['entry']) for old in passed):
            continue  # Renaming an already checked unit is not new progress.
        newly.append(entry)
        passed.append(dict(entry=entry, snapshot=report.get('snapshot')))
    reasons = {item['reason'] for item in units.values()}
    outcome = ('RELATED_UNIT_PASSED' if newly else 'PROOF_FAILURE' if 'verification_failure' in reasons else
               'COMPILATION' if 'compilation_error' in reasons else 'SCHEDULING_OR_TOOL' if reasons.intersection(
                   {'verification_timeout', 'tool_unavailable', 'dependency_or_toolchain_unavailable', 'verifier_error', 'check_budget_exhausted', 'log_limit'}) else
               'PREPARATION' if any(item['status'] != 'PASS' for item in units.values()) else 'LOCAL_ONLY_OR_REVIEW')
    if advance:
        record['attempts'] += int(any(item.get('command') for item in units.values()))
    if newly:
        record['stalled'] = 0
    elif advance and outcome == 'PROOF_FAILURE':
        record['stalled'] += 1
    # Missing files/timeouts neither consume redesigns nor erase past failures.
    record.update(last_outcome=outcome, newly_checked=newly, first_blocker=current.get('first_blocker'))
    record['units'].update({entry: item['status'] for entry, item in units.items()})
    return dict(key=key, obligation=plan['id'], version=len(record['versions']), outcome=outcome,
                newly_checked=newly, first_blocker=current.get('first_blocker'),
                layer=current['layer'], checks={entry: {k:i.get(k) for k in
                    ('status', 'reason', 'log', 'errors', 'command', 'candidate_revision')} for entry, i in units.items()})


def next_obligation(state, report):
    current = report.get('current_obligation', {})
    plan = current.get('plan', {})
    key = state.get('current_key')
    record = state.get('obligations', {}).get(key, {})
    if record.get('blocked'):
        return 'This obligation exhausted its redesign allowance. Select a different necessary dependency and explain the change, or report no legal next step. Renaming this obligation does not reset it.'
    if state.get('integration_wait', 0) >= 2:
        return 'ESTABLISH_EXECUTION_CONNECTION: select actual initialization or one real handler step and its executable-to-history proof entry now. Run it even if it fails; repair its first concrete diagnostic before unrelated features/helpers. This is missing integration, not a failed Raft invariant.'
    if plan.get('error') or not plan:
        return 'SELECT_OBLIGATION: name one actual constructor/handler or necessary parent dependency, its proof entry and test filters in the obligation block. Begin execution/observation correspondence with the first meaningful core behavior.'
    first = current.get('first_blocker') or {}
    reason = first.get('reason', '')
    if reason in ('missing_entry_or_dependency', 'missing_current_obligation'):
        return 'ESTABLISH_INTERFACE: implement/check ' + first.get('entry', plan['entry']) + '; no proof failure has yet been demonstrated.'
    if reason in ('verification_timeout', 'check_budget_exhausted', 'tool_unavailable', 'dependency_or_toolchain_unavailable', 'verifier_error'):
        return 'CHECKING_BLOCKED: narrow the selected unit or fix checking availability; preserve targets. A timeout or absent tool is not a protocol counterexample.'
    if reason == 'verification_failure':
        return 'REPAIR_CURRENT_OBLIGATION: ' + first.get('entry', plan['entry']) + '; use the real diagnostic, discharge caller/recovery premises or revise the representation. A local failed condition is not automatically reachable.'
    if reason == 'compilation_error':
        return 'REPAIR_INTERFACE: resolve the current compilation error before judging the proof design.'
    return 'CONNECT_PARENT: retain the checked local result; check its actual caller/initialization/step and parent use next. Open premises and outer paths remain obligations, even when every nominated unit passes.'


def select_mode(state, report):
    if not state['attempts']:
        return 'start'
    record = state.get('obligations', {}).get(state.get('current_key'), {})
    if record.get('last_outcome') == 'PROOF_FAILURE' and record.get('stalled', 0) >= state['stall_attempts']:
        if record['redesigns'] < state['max_obligation_redesigns'] and state['redesigns'] < state['max_redesigns']:
            record['redesigns'] += 1
            state['redesigns'] += 1
            record['stalled'] = 0
            return 'redesign'
        record['blocked'] = True
    return 'continue'


def checkpoint_scope(report):
    """Save all named code and transitive proof dependencies, not function bodies."""
    plan = report.get('current_obligation', {}).get('plan', {})
    snapshot = Path(report['snapshot'])
    paths = set()
    for entry in report.get('current_obligation', {}).get('units', {}).values():
        paths.update(module_paths(snapshot / entry, snapshot))
    paths.update(snapshot / ref.partition('::')[0] for ref in plan.get('functions', []))
    return sorted(str(p.relative_to(snapshot / 'madraft')) for p in paths
                  if p.is_relative_to(snapshot / 'madraft') and p.is_file())


def round_plan(state):
    left = state['total_seconds'] - state['seconds_spent']
    minimum = state['min_work_seconds'] + state['handoff_seconds']
    if state.get('attempts', 0) >= state.get('max_rounds', 64):
        return 0, 'final'
    normal = min(state['slice_seconds'], left - state['check_seconds'] - state['final_seconds'])
    if normal >= minimum and state.get('attempts', 0) + 1 < state.get('max_rounds', 64):
        return normal, 'feedback'
    closing = min(state['slice_seconds'], left - state['final_seconds'])
    return (closing, 'final') if closing >= minimum else (0, 'final')


class Loop:
    def __init__(self, root, workspace, control, state):
        self.root, self.workspace, self.control, self.state = root, workspace, control, state
        self.authority = control / 'inputs'
        self.statefile = control / 'state.json'
        self.started, self.spent_at_start = time.monotonic(), state['seconds_spent']

    def elapsed(self):
        self.state['seconds_spent'] = max(self.state['seconds_spent'],
            self.spent_at_start + time.monotonic() - self.started)

    def save(self):
        save_json(self.statefile, self.state)

    def charge(self, kind, limit, action):
        state = self.state
        self.elapsed()
        available = state['total_seconds'] - state['seconds_spent']
        allowance = min(limit, available)
        if allowance <= 0:
            return None
        spent = state['seconds_spent']
        state['seconds_spent'] += allowance
        state['in_flight'] = {'kind': kind, 'allowance': allowance, 'pid': None}
        self.save()  # A killed controller cannot replenish this reservation.
        started = time.monotonic()
        def started_process(pid):
            state['in_flight']['pid'] = pid
            self.save()
        try:
            return action(allowance, started_process)
        finally:
            actual = time.monotonic() - started
            state['seconds_spent'] = spent + actual
            state.setdefault('costs', []).append(dict(round=state['attempts'], kind=kind,
                reserved_seconds=allowance, actual_seconds=actual))
            state['in_flight'] = None
            self.save()

    def remember(self, report, *, advance=True, checkpoint=True):
        clean = not any(report.get(k) for k in ('changed_standards', 'concurrent_changes', 'evidence_mismatches'))
        event = record_attempt(self.state, report, advance=advance) if clean else dict(outcome='INVALID_EVIDENCE')
        event.update(revision=report['candidate_revision'], report=str(Path(report.get('snapshot', self.control)).parent / 'results.json'),
                     run_id=self.state['run_id'], input_fingerprint=self.state['input_fingerprint'],
                     evidence='INDEPENDENT_CHECK', round=self.state['attempts'])
        if advance:
            previous = self.state.get('last_attempt_report')
            if previous and Path(previous).is_file() and report.get('snapshot'):
                old = json.loads(Path(previous).read_text())
                if old.get('snapshot'):
                    a = files_under(Path(old['snapshot']) / 'madraft')
                    b = files_under(Path(report['snapshot']) / 'madraft')
                    patch = []
                    for name in sorted(a.keys() | b.keys()):
                        left = a[name].read_text(errors='replace') if name in a else ''
                        right = b[name].read_text(errors='replace') if name in b else ''
                        if left != right:
                            patch.extend(difflib.unified_diff(left.splitlines(True), right.splitlines(True), 'previous/' + name, 'attempt/' + name))
                    path = Path(event['report']).parent / 'attempt.patch'
                    path.write_text(''.join(patch))
                    event['patch'] = str(path)
            self.state['last_attempt_report'] = event['report']
            with (self.control / 'attempts.jsonl').open('a') as file:
                file.write(json.dumps(event) + '\n')
            shutil.copy2(self.control / 'attempts.jsonl', self.workspace / 'runs/attempts.jsonl')
        build_status = report['checks'].get('build', {}).get('status', 'NOT_RUN')
        local_status = report['checks'].get('current_entry', {}).get('status')
        if checkpoint and clean and report.get('snapshot') and self.state['attempts'] and (build_status == 'PASS' or local_status == 'PASS'):
            current = report.get('current_obligation', {})
            cp = dict(id='cp-' + str(len(self.state['checkpoints']) + 1),
                      run_id=self.state['run_id'], input_fingerprint=self.state['input_fingerprint'],
                      report=event['report'], scope=checkpoint_scope(report),
                      layer=current.get('layer', 'BUILDABLE'), build_status=build_status,
                      premises=current.get('plan', {}).get('premises', 'Unreviewed'),
                      first_blocker=current.get('first_blocker'))
            if cp['layer'] == 'PREPARATION': cp['layer'] = 'BUILDABLE'
            self.state['checkpoints'].append(cp)
        self.save()

    def replay_attempts(self):
        """Replay bounded candidate-local microchecks; candidate JSON is never proof."""
        pending = []
        for p in sorted((self.workspace / 'runs/checks').glob('*/results.json')):
            if str(p.parent.name) in self.state['replayed']:
                continue
            if p.is_symlink() or not p.resolve().is_relative_to(self.workspace / 'runs/checks'):
                continue
            try:
                claim = json.loads(p.read_text())
            except (ValueError, OSError):
                continue
            if claim.get('mode') == 'obligation': pending.append(p)
        self.state['pending_local_attempts'] = len(pending)
        for path in pending[:3]:
            self.elapsed()
            allowance = min(self.state['check_seconds'], self.state['total_seconds'] - self.state['seconds_spent'] - self.state['final_seconds'])
            if allowance < 1: break
            # Only this local check directory is consulted, never its claimed paths.
            candidate = path.parent / 'candidate'
            if not candidate.resolve().is_relative_to(path.parent.resolve()):
                raise ValueError('Microcheck snapshot outside this experiment check directory')
            tag = 'replay-' + str(time.time_ns())
            def replay(seconds, started):
                source = files_under(candidate / 'madraft')
                target = self.control / tag / 'input'
                copy_files(self.authority, target, self.state['input_files'])
                shutil.rmtree(target / 'madraft')
                copy_files(candidate / 'madraft', target / 'madraft', source)
                note = candidate / 'PROGRESS.md'
                if note.is_symlink(): raise ValueError('Microcheck progress must not be an external link')
                if note.is_file(): shutil.copy2(note, target / 'PROGRESS.md')
                checks = Checks(target, self.authority, self.root, self.control / tag / 'checked', seconds, tag)
                checks.obligation()
                return checks.finish('obligation-replay', list(checks.results))
            report = self.charge('attempt-replay', allowance, replay)
            if report:
                self.state['replayed'].append(path.parent.name)
                self.state['pending_local_attempts'] -= 1
                self.remember(report)

    def rollback(self, report):
        """Restore a scoped same-run snapshot; source state never restores controller state."""
        plan = report.get('current_obligation', {}).get('plan', {})
        if not plan.get('rollback'):
            return False
        if report.get('status') in ('BLOCKED', 'NEEDS_REVIEW') or report.get('concurrent_changes'):
            return False
        if differences(Path(report['snapshot']) / 'madraft', self.workspace / 'madraft'):
            raise ValueError('Candidate changed after checking; refuse to overwrite concurrent work')
        cp = next((c for c in self.state['checkpoints'] if c['id'] == plan['rollback']), None)
        if not cp or cp['run_id'] != self.state['run_id'] or cp['input_fingerprint'] != self.state['input_fingerprint']:
            raise ValueError('Rollback checkpoint must belong to this experiment and inputs')
        record = Path(cp['report']).resolve()
        if not record.is_relative_to(self.control.resolve()):
            raise ValueError('Checkpoint outside this experiment controller')
        old = json.loads(record.read_text())
        snapshot = Path(old['snapshot']).resolve()
        if not snapshot.is_relative_to(self.control.resolve()):
            raise ValueError('Checkpoint source outside this experiment')
        # Named roots include every recorded proof dependency, plus current ones
        # so an abandoned newly-created helper cannot silently survive rollback.
        scope = sorted(set(cp['scope']) | set(checkpoint_scope(report)) | set(plan.get('restore', [])))
        if not scope: raise ValueError('Rollback needs a concrete source/proof scope')
        protected = set(protected_files(self.authority))
        for name in scope:
            p = Path(name)
            if p.is_absolute() or '..' in p.parts or p.parts[0] not in ('src', 'proof') or 'madraft/' + name in protected:
                raise ValueError('Rollback cannot change protected or outside-workspace files: ' + name)
        before = files_under(self.workspace / 'madraft')
        saved = files_under(snapshot / 'madraft')
        number = len(self.state.get('rollbacks', [])) + 1
        output = self.control / ('rollback-' + str(number))
        copy_files(self.workspace / 'madraft', output / 'failed-madraft', before)
        patch = []
        for name in scope:
            a = before[name].read_text() if name in before else ''
            b = saved[name].read_text() if name in saved else ''
            patch.extend(difflib.unified_diff(a.splitlines(True), b.splitlines(True), 'failed/' + name, 'restored/' + name))
        (output / 'changes.patch').write_text(''.join(patch))
        note = self.workspace / 'PROGRESS.md'
        text = note.read_text()
        (output / 'failed-PROGRESS.md').write_text(text)
        event = dict(checkpoint=cp['id'], scope=scope, failed_report=self.state.get('last_check'),
                     patch=str(output / 'changes.patch'), recheck='REQUIRED', status='RESTORING',
                     seconds_spent=self.state['seconds_spent'])
        self.state.setdefault('rollbacks', []).append(event)
        self.state['last_check'] = None
        self.save()  # An interrupted restoration must not leave a current PASS.
        with (self.control / 'attempts.jsonl').open('a') as file:
            file.write(json.dumps(dict(event, outcome='ROLLBACK_STARTED', run_id=self.state['run_id'])) + '\n')
        # All validation and failure preservation precede the first mutation.
        for name in scope:
            target = self.workspace / 'madraft' / name
            if name in saved:
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(saved[name], target)
            elif target.is_file(): target.unlink()
        old_note = snapshot / 'PROGRESS.md'
        block = re.search(r'<!-- obligation -->.*?<!-- /obligation -->', old_note.read_text() if old_note.exists() else '', re.S)
        if block:
            restored = re.sub(r'^\s*(?:rollback|restore)\s*=.*$', '', block[0], flags=re.M)
            text = re.sub(r'<!-- obligation -->.*?<!-- /obligation -->', lambda _: restored, text, flags=re.S)
        else:
            text = re.sub(r'^\s*(?:rollback|restore)\s*=.*$', '', text, flags=re.M)
        note.write_text(text)
        event['status'] = 'RESTORED_UNCHECKED'
        with (self.control / 'attempts.jsonl').open('a') as file:
            file.write(json.dumps(dict(event, outcome='ROLLBACK', run_id=self.state['run_id'])) + '\n')
        shutil.copy2(self.control / 'attempts.jsonl', self.workspace / 'runs/attempts.jsonl')
        self.state['history'][-1]['changed_files'] = sorted(set(self.state['history'][-1]['changed_files']) | set(scope))
        self.state['last_check'] = None  # No restored PASS is current evidence.
        self.save()
        return True

    def inspect(self, initial=False, final=False, advance=True):
        tag = f'check-{self.state["attempts"]}-{time.time_ns()}'
        limit = self.state['final_seconds'] if final else min(self.state['check_seconds'],
            max(0, self.state['total_seconds'] - self.state['seconds_spent'] - self.state['final_seconds']))
        report = self.charge('final' if final else 'check', limit, lambda seconds, started:
            evaluate(self.workspace, authority=self.authority, tool_root=self.root,
                     output=self.control / tag, timeout=seconds, seeds=self.state['seeds'],
                     revision=tag, initial=initial, mode='final' if final else 'feedback',
                     changed_files=self.state['history'][-1]['changed_files'] if self.state['history'] else (),
                     test_history=self.state.get('test_history', {})))
        if report:
            report['tool_versions'] = self.state.get('tools')
            save_json(self.control / tag / 'results.json', report)
            for item in report['checks'].values():
                family = item.get('family', '')
                family = next((f for f in FAMILIES if family.startswith(f)), None)
                if family and item.get('command'):
                    record = self.state.setdefault('test_history', {}).setdefault(family, {})
                    record['last_attempt'] = self.state['attempts']
                    if item.get('process') == 'COMPLETED': record['last_completed'] = self.state['attempts']
                    record['last_result'] = item['status']
                    record.update(candidate_revision=report['candidate_revision'], log=item.get('log'),
                                  uncovered_tests=item.get('uncovered_tests', []))
            self.state['last_check'] = str(self.control / tag / 'results.json')
            visible = self.workspace / 'runs' / tag
            visible.mkdir(parents=True)
            for path in (self.control / tag).glob('*.log'):
                shutil.copy2(path, visible / path.name)
            if (self.control / tag / 'results.md').exists():
                shutil.copy2(self.control / tag / 'results.md', visible / 'results.md')
            save_json(visible / 'results.json', report)
            if not initial:
                self.remember(report, advance=advance)
                plan = report.get('current_obligation', {}).get('plan', {})
                current = report.get('current_obligation', {})
                checked = report['checks'].get('current_entry', {})
                connected = plan.get('kind') in ('initialization', 'step') and checked.get('status') == 'PASS' and current.get('imported_references')
                protocol_changes = self.state['history'] and any(n.startswith('src/') and n.endswith('.rs') for n in self.state['history'][-1]['changed_files'])
                if advance and (connected or protocol_changes or self.state.get('integration_wait')):
                    self.state['integration_wait'] = 0 if connected else self.state.get('integration_wait', 0) + int(
                        report['checks'].get('build', {}).get('status') == 'PASS' and
                        (checked.get('status') == 'PASS' or report['checks'].get('proof', {}).get('status') == 'PASS'))
            self.save()
        return report

    def stop(self, status, reason, report=None):
        self.elapsed()
        self.state.update(status=status, reason=reason)
        self.save()
        summary = {k: self.state[k] for k in ('status', 'reason', 'run_id', 'input_version', 'git_revision',
                   'model', 'effort', 'attempts', 'redesigns', 'seconds_spent', 'total_seconds', 'seeds')}
        summary['last_check'] = self.state.get('last_check')
        summary['semantic_boundary'] = SEMANTIC_BOUNDARY
        summary['liveness'] = 'OPEN'
        summary.update(guidance=self.state.get('guidance', 'B'), system_version=FORMAT,
            input_fingerprint=self.state.get('input_fingerprint'), prompt_version=self.state.get('prompt_version'),
            unused_seconds=max(0, self.state['total_seconds'] - self.state['seconds_spent']),
            costs=self.state.get('costs', []), interventions=self.state.get('interventions', []),
            historical_test_coverage=self.state.get('test_history', {}),
            semantic_review='NOT_PERFORMED', implementation_acceptance='NOT_ESTABLISHED',
            generation_experiments=1, continuation_rounds=max(0,self.state['attempts']-1),
            token_usage=[h.get('token_usage') for h in self.state['history']],
            tools=self.state.get('tools'), assessments=(report or {}).get('assessments', {}))
        summary.update(current_obligation=(report or {}).get('current_obligation'),
                       obligations=self.state.get('obligations', {}), checkpoints=self.state.get('checkpoints', []),
                       rollbacks=self.state.get('rollbacks', []), pending_local_attempts=self.state.get('pending_local_attempts', 0))
        summary['unchanged_source_agent_seconds'] = sum(h.get('process_seconds', 0) for h in self.state['history'] if h.get('no_source_change'))
        summary['idle_or_wasted_seconds'] = None  # No-source-change time need not be wasted reasoning.
        summary['interrupted_budget_seconds'] = self.state.get('interrupted_budget_seconds', 0)
        summary['controller_overhead_seconds'] = max(0, self.state['seconds_spent'] - sum(
            c['actual_seconds'] for c in summary['costs']) - summary['interrupted_budget_seconds'])
        save_json(self.workspace / 'runs/EXPERIMENT.json', summary)
        lines = ['# Experiment status', '', f'{status}: {reason}', '',
                 f'Input: {INPUT_VERSION}; origin: {self.state["run_id"]}; source revision: {self.state["git_revision"]}.', '',
                 f'Model: {self.state["model"]}; effort: {self.state["effort"]}; rounds: {self.state["attempts"]}; cumulative seconds: {self.state["seconds_spent"]:.1f}/{self.state["total_seconds"]:.1f}.', '',
                 f'Latest independent evidence: {self.state.get("last_check", "not run")}. Historical results are not reused after changes.', '',
                 '| Check or obligation | Independent result |', '| --- | --- |']
        for name in dict.fromkeys([*(report or {}).get('checks', {}), *OBLIGATIONS]):
            item = (report or {}).get('checks', {}).get(name, {})
            lines.append(f'| {name} | {item.get("status", "OPEN")}: {item.get("reason", "not checked")} |')
        lines += ['', SEMANTIC_BOUNDARY, '', 'Fixed environment/host assumptions: requirements/environment.md; coverage responsibilities: requirements/coverage.md. Candidate dependency notes in PROGRESS.md and RESULT.md remain unverified until actual callers discharge them. Agent RESULT.md does not override the independent results.', '']
        lines += [f'Guidance: {summary["guidance"]}; unused budget: {summary["unused_seconds"]:.1f}s. Semantic review: NOT_PERFORMED; implementation acceptance: NOT_ESTABLISHED.', '',
                  '| Goal | Formula | Actual connection | Implementation task |', '| --- | --- | --- | --- |']
        for name, item in summary['assessments'].items():
            lines.append(f'| {name} | {item["formula_status"]} | {item["connection_status"]} | {item["implementation_task"]} |')
        lines += ['', 'Reported token counters retain UNKNOWN scope and are not summed across resumed calls. Dollar cost is unknown. One fresh generation experiment contains multiple continuation rounds and multiple test seeds; these are not independent generation successes.', '']
        (self.workspace / 'runs/EXPERIMENT.md').write_text('\n'.join(lines))
        print(f'{status}: {reason}; state: {self.statefile}', flush=True)
        return 0 if status == 'MACHINE_CHECKS_PASSED' else 2 if status in ('BLOCKED', 'NEEDS_REVIEW') else 1

    def run(self):
        state = self.state
        changes = origin_changes(self.root, self.control, state)
        if changes:
            return self.stop('NEEDS_REVIEW', 'Experiment inputs/controller changed: ' + ', '.join(changes))
        if state['in_flight']:
            pid = state['in_flight'].get('pid')
            if pid and Path(f'/proc/{pid}/stat').exists() and Path(f'/proc/{pid}/stat').read_text().split()[2] != 'Z':
                return self.stop('BLOCKED', 'Recorded worker is still present; its watcher must finish cleanup before resume')
            # The entire interrupted allowance remains charged. Recover only this
            # run's thread; never open a new conversation on a partly generated answer.
            state['interrupted_budget_seconds'] = state.get('interrupted_budget_seconds', 0) + state['in_flight']['allowance']
            state.setdefault('interventions', []).append(dict(action='resume_after_interruption', actual_seconds=None,
                charged_reservation=state['in_flight']['allowance']))
            state['in_flight'] = None
            self.save()
        if state['attempts']:
            directory = self.workspace / f'runs/round-{state["attempts"]}'
            state['thread_id'] = session_id(directory / 'agent.jsonl', state['thread_id'])
            if state['history'] and state['history'][-1]['process'] == 'INTERRUPTED':
                progress = self.workspace / 'PROGRESS.md'
                state['history'][-1]['progress'] = progress.read_text() if progress.exists() else '(No candidate progress note)'
            self.save()
        if state['attempts'] and not state['thread_id']:
            return self.stop('BLOCKED', 'No resumable session ID for this generated candidate')
        if state['seconds_spent'] >= state['total_seconds']:
            return self.stop('INCOMPLETE', 'Cumulative budget exhausted')
        probes = self.control / ('tools-' + str(time.time_ns()))
        probes.mkdir()
        capability = self.charge('preflight', 40, lambda seconds, started:
                                probe_tools(self.root, self.workspace, state, probes, seconds))
        save_json(probes / 'results.json', capability)
        if not capability or capability['status'] != 'PASS':
            return self.stop('BLOCKED', (capability or {}).get('reason', 'No budget for tool checks'))
        if state['tools'] and state['tools'] != capability['evidence']:
            return self.stop('NEEDS_REVIEW', 'Tool versions/capabilities changed since this experiment started')
        state['tools'] = capability['evidence']
        self.save()
        # Resume always refreshes checks; no historical PASS is reusable after edits.
        report = self.inspect(initial=state['attempts'] == 0, advance=False)
        reason = 'Insufficient budget for useful work and final checks'
        while report:
            if report['status'] in ('BLOCKED', 'NEEDS_REVIEW'):
                return self.stop(report['status'], 'Independent check cannot proceed under the fixed inputs', report)
            if report['status'] == 'MACHINE_CHECKS_PASSED' and report.get('mode') == 'final':
                return self.stop('MACHINE_CHECKS_PASSED', 'Machine checks passed; semantic review and implementation acceptance remain outstanding', report)
            if report.get('mode') != 'final' and all(report['checks'].get(n, {}).get('status') == 'PASS' for n in (*OBLIGATIONS, 'safety', 'linkage_review', *('connection_' + a for a in CONNECTIONS))):
                report = self.inspect(final=True) or report
                if report and report['status'] == 'MACHINE_CHECKS_PASSED' and report.get('mode') == 'final':
                    return self.stop('MACHINE_CHECKS_PASSED', 'Machine checks passed; semantic review and implementation acceptance remain outstanding', report)
            if report['status'] in ('BLOCKED', 'NEEDS_REVIEW'):
                return self.stop(report['status'], 'Final check cannot proceed under the fixed inputs', report)
            self.elapsed()
            allowance, next_check = round_plan(state)
            if not allowance or state['attempts'] >= state['max_rounds']:
                if state['attempts'] >= state['max_rounds']: reason = 'Maximum generation rounds reached'
                break
            mode = select_mode(state, report)
            blocked = state.get('obligations', {}).get(state.get('current_key'), {})
            if blocked.get('blocked') and blocked.get('notified'):
                reason = 'Current obligation exhausted redesigns; no different necessary dependency was selected'
                break
            if blocked.get('blocked'): blocked['notified'] = True
            prompt = feedback(self.authority, state, report, mode, self.workspace, allowance)
            state['attempts'] += 1
            number = state['attempts']
            directory = self.workspace / f'runs/round-{number}'
            directory.mkdir(exist_ok=True)
            (directory / 'prompt.md').write_text(prompt)
            before = {n: p.read_bytes() for n, p in files_under(self.workspace / 'madraft').items()}
            claim_before = (self.workspace / 'PROGRESS.md').read_bytes() if (self.workspace / 'PROGRESS.md').is_file() else None
            argv = command(state, self.workspace, directory / 'agent.md')
            record = dict(round=number, mode=mode, process='INTERRUPTED', exit=None,
                          soft_seconds=allowance - state['handoff_seconds'], hard_seconds=allowance,
                          next_check=next_check,
                          command=argv, changed_files=[], checks=None, check_status='INCOMPLETE',
                          progress='(Work slice interrupted before its outcome was saved)')
            state['history'].append(record)
            self.save()
            _, env = environment(self.root)
            env['CODEX_HOME'] = str(self.workspace / 'runs/codex-home')
            result = self.charge('agent', allowance, lambda seconds, started: run_process(
                argv, self.workspace, stdin=prompt, env=env, timeout=seconds,
                max_log_bytes=state['max_log_bytes'], on_start=started,
                output_paths=(directory / 'agent.jsonl', directory / 'agent.stderr.log')))
            if not result:
                break
            state['thread_id'] = session_id(directory / 'agent.jsonl', state['thread_id'])
            state['last_process'], state['last_exit'] = result.status, result.exit_code
            progress = self.workspace / 'PROGRESS.md'
            record.update(process=result.status, exit=result.exit_code,
                          process_seconds=result.elapsed_seconds, token_usage=token_usage(directory / 'agent.jsonl'),
                          handoff='normal' if result.status == 'COMPLETED' and result.exit_code == 0 else 'hard_timeout' if result.status == 'TIMEOUT' else 'failed',
                          progress=progress.read_text() if progress.exists() else '(No candidate progress note)')
            self.save()
            after = files_under(self.workspace / 'madraft')
            changed = sorted(n for n in before.keys() | after.keys() if n not in before or n not in after or before[n] != after[n].read_bytes())
            patch = []
            for name in changed:
                old = before.get(name, b'').decode('utf-8', errors='replace').splitlines(True)
                new = after[name].read_text(errors='replace').splitlines(True) if name in after else []
                patch.extend(difflib.unified_diff(old, new, 'before/madraft/' + name, 'after/madraft/' + name))
            (directory / 'changes.patch').write_text(''.join(patch))
            record.update(changed_files=changed, patch=str(directory / 'changes.patch'))
            record['no_source_change'] = not changed
            self.save()
            if result.status == 'CANCELLED':
                return self.stop('INCOMPLETE', 'Controller interrupted; candidate and charged budget retained')
            if not changed and report.get('mode') == 'final' and claim_before == (
                    progress.read_bytes() if progress.is_file() else None) and not differences(
                        self.authority, self.workspace, protected_files(self.authority)):
                # No new check is represented as run. A stopped, unchanged
                # candidate already has a final result; do not burn another final.
                record.update(checks=state.get('last_check'), check_status=report['status'],
                              unchanged_final_evidence=state.get('last_check'))
                return self.stop('INCOMPLETE', 'Agent returned an unchanged candidate after final checks; no duplicate final invocation', report)
            self.replay_attempts()
            report = self.inspect(final=next_check == 'final')
            if report and self.rollback(report):
                self.elapsed()  # Restoration is charged wall time, never refunded.
                report = self.inspect(final=next_check == 'final')
                state['rollbacks'][-1]['recheck'] = state.get('last_check') or 'NOT_RUN_BUDGET_EXHAUSTED'
                if report:
                    state['rollbacks'][-1]['status'] = 'RECHECKED'
            record.update(checks=state.get('last_check') if report else None,
                          check_status=(report or {}).get('status', 'INCOMPLETE'))
            if report:
                current_record = state.get('obligations', {}).get(state.get('current_key'), {})
                record['newly_checked_dependencies'] = current_record.get('newly_checked', [])
            self.save()
            if result.status == 'BLOCKED' or (result.status == 'COMPLETED' and result.exit_code != 0) or not state['thread_id']:
                return self.stop('BLOCKED', 'Agent execution/authentication failed or no session ID was produced', report)
            # TIMEOUT/LOG_LIMIT are bounded failed slices, not success. A captured
            # thread may continue with actual diagnostics while budgets permit.
        # The final reserve belongs to checks of the current stopped candidate.
        final_report = report if report and report.get('mode') == 'final' else self.inspect(final=True)
        if final_report:
            report = final_report
        if report and report['status'] == 'MACHINE_CHECKS_PASSED' and report.get('mode') == 'final':
            return self.stop('MACHINE_CHECKS_PASSED', 'Machine checks passed; semantic review and implementation acceptance remain outstanding', report)
        return self.stop((report or {}).get('status') if (report or {}).get('status') in ('BLOCKED', 'NEEDS_REVIEW') else 'INCOMPLETE', reason, report)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['new', 'resume'])
    parser.add_argument('directory', type=Path)
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--model')
    parser.add_argument('--effort', choices=['low', 'medium', 'high', 'xhigh'])
    parser.add_argument('--codex')
    parser.add_argument('--guidance', choices=['A', 'B'], help='Same inputs/checker/budgets; B adds actual-call connection guidance')
    parser.add_argument('--total-seconds', type=float)
    parser.add_argument('--max-rounds', type=int)
    args = parser.parse_args()
    root, workspace = ROOT.resolve(), args.directory.expanduser().resolve()
    if workspace.parent != root / 'runs' or workspace.name.startswith('.'):
        parser.error('Experiment must be a fresh direct child of this checkout\'s runs/')
    control = control_path(root, workspace)
    if args.action == 'new':
        if workspace.exists() or control.exists():
            parser.error('New requires an unused workspace and controller record')
        cfg = settings(root)
        a = cfg['agent']
        state = dict(thread_id=None, seconds_spent=0.0, attempts=0, model=args.model or a['model'],
                     effort=args.effort or a['reasoning_effort'], codex=args.codex or cfg['tools']['codex'],
                     guidance=args.guidance or a.get('guidance', 'B'),
                     seeds=configured_seeds(root))
        for key in ('total_seconds', 'slice_seconds', 'check_seconds', 'final_seconds', 'min_work_seconds', 'handoff_seconds'):
            state[key] = float(a[key])
        for key in ('max_rounds', 'stall_attempts', 'max_obligation_redesigns', 'max_redesigns', 'max_log_bytes'):
            state[key] = int(a[key])
    else:
        if not (control / 'state.json').is_file():
            parser.error('No new-system origin record: historical runs are read-only, not warm-start inputs')
        state = json.loads((control / 'state.json').read_text())
        if state.get('format') != FORMAT or state.get('input_version') != INPUT_VERSION or state.get('workspace') != str(workspace):
            parser.error('Run origin/input version mismatch; start a fresh experiment')
        for flag in ('model', 'effort', 'codex', 'guidance'):
            value = getattr(args, flag)
            if value and value != state[flag]:
                parser.error('A resumed experiment retains its model, effort and Codex command; use new to change them')
    previous_limits = {key: state[key] for key in ('total_seconds', 'max_rounds')}
    for key in ('total_seconds', 'max_rounds'):
        if getattr(args, key) is not None:
            state[key] = getattr(args, key)
    for key in ('total_seconds', 'slice_seconds', 'check_seconds', 'final_seconds', 'min_work_seconds', 'handoff_seconds', 'max_rounds', 'stall_attempts', 'max_log_bytes'):
        if not math.isfinite(state[key]) or state[key] <= 0:
            parser.error(key + ' must be finite and positive')
    if state['slice_seconds'] < state['min_work_seconds'] + state['handoff_seconds']:
        parser.error('slice_seconds must include minimum work and handoff grace')
    if state['max_redesigns'] < 0 or state['max_obligation_redesigns'] < 0 or not state['model']:
        parser.error('Set an explicit model and nonnegative redesign budget')
    print(f'Workspace: {workspace}\nModel: {state["model"]}; effort: {state["effort"]}; guidance: {state["guidance"]}\n'
          f'Budget remaining: {state["total_seconds"] - state["seconds_spent"]:.1f}s; '
          f'maximum total rounds: {state["max_rounds"]}; seeds: {state["seeds"]}\n'
          f'Hard slice: {state["slice_seconds"]}s; handoff grace: {state["handoff_seconds"]}s; minimum work: {state["min_work_seconds"]}s; '
          f'feedback: {state["check_seconds"]}s; final reserve: {state["final_seconds"]}s', flush=True)
    if not args.execute:
        names = input_files(root)
        print('Input version: ' + INPUT_VERSION + '; source input fingerprint: ' + fingerprint({n:root/n for n in names}))
        print('Inputs: ' + ', '.join(names))
        print('One obligation at a time; independent checks, same-session repair and same-run checkpoints. Total budget includes agents and checks.')
        print('PREVIEW: no candidate created, no tools or agent called. Add --execute to run the independent feedback loop.')
        return 0
    setup_started = time.monotonic()
    if args.action == 'new':
        control = prepare(root, workspace, state)
    else:
        state.setdefault('interventions', []).append(dict(action='operator_resume', at=time.time(),
            previous_limits=previous_limits, requested_limits={k:state[k] for k in previous_limits}))
    with (control / 'session.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        current = json.loads((control / 'state.json').read_text())
        if current['attempts'] != state['attempts'] or current['seconds_spent'] != state['seconds_spent']:
            raise ValueError('Concurrent controller changed this experiment; reload its state')
        state['seconds_spent'] += time.monotonic() - setup_started
        loop = Loop(root, workspace, control, state)
        previous = signal.signal(signal.SIGTERM, lambda *_: (_ for _ in ()).throw(KeyboardInterrupt()))
        try:
            return loop.run()
        except KeyboardInterrupt:
            return loop.stop('INCOMPLETE', 'Controller interrupted; resume retains cumulative spending')
        except ValueError as exc:
            return loop.stop('NEEDS_REVIEW', str(exc))
        except OSError as exc:
            return loop.stop('BLOCKED', str(exc))
        finally:
            signal.signal(signal.SIGTERM, previous)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, configparser.Error) as exc:
        print(f'Cannot run: {exc}', file=sys.stderr)
        sys.exit(2)

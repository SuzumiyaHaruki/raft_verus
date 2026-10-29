#!/usr/bin/env python3
"""A bounded generate/check/feedback loop in one fresh workspace and conversation."""
import argparse
import configparser
import difflib
import fcntl
import json
import math
import os
from pathlib import Path
import shlex
import shutil
import signal
import sys
import time
import uuid

from check import OBLIGATIONS, SEMANTIC_BOUNDARY, configured_seeds, evaluate
from inputs import INPUT_VERSION, copy_files, differences, files_under, git_revision, input_files, save_json
from process import run_process
from tools import ROOT, environment, settings

PROMPTS = ('start', 'continue', 'redesign')
FORMAT = 'madraft-generate-check-2'


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
                 status='INCOMPLETE', reason='created', stale_rounds=0, redesigns=0)
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


def feedback(authority, state, report, mode, workspace, seconds):
    parts = [(authority / f'scripts/prompts/{mode}.md').read_text().strip(),
             f'Workspace: {workspace}. Work slice: at most {seconds:.1f} seconds. '
             f'Cumulative budget remaining: {state["total_seconds"] - state["seconds_spent"]:.1f} seconds. '
             f'Round: {state["attempts"] + 1}/{state["max_rounds"]}.',
             'All diagnostics below belong to this experiment. Snapshot paths identify the stopped candidate; edit this workspace\'s madraft/ files.']
    if report:
        parts.append('Independent candidate revision: ' + report['candidate_revision'])
        for name, item in report['checks'].items():
            parts.append(f'{name}: {item["status"]} — {item["reason"]}')
            if item.get('command'):
                parts.append('Actual command: ' + shlex.join(item['command']))
            if item.get('log'):
                parts.append('Raw diagnostic: ' + item['log'])
            if item['status'] != 'PASS':
                parts.append(item.get('diagnostic', '')[-1800:])
                if item.get('uncovered_tests'):
                    parts.append('Uncovered required tests: ' + ', '.join(item['uncovered_tests']))
        parts.append('Reproduce on current files: python3 scripts/check.py final --timeout 300')
        parts.append(SEMANTIC_BOUNDARY)
    for record in state['history'][-3:]:
        parts.append(f'Previous round {record["round"]}: {record["process"]}; independent result {record["check_status"]}.')
        parts.append('Candidate-reported approach/dependencies (UNVERIFIED, not verifier conclusions):\n' + record.get('progress', '')[-5000:])
    return '\n\n'.join(p for p in parts if p) + '\n'


def unresolved(report):
    """Compare actual gaps, ignoring changing snapshot paths and local counts."""
    result = []
    for name, item in (report or {}).get('checks', {}).items():
        if name == 'proof':
            continue
        diagnostic = item.get('diagnostic', '') if item['status'] != 'PASS' else ''
        for path in (report.get('snapshot'), str(Path(item['log']).parent) if item.get('log') else None):
            if path:
                diagnostic = diagnostic.replace(path, '<check>')
        result.append((name, item['status'], item['reason'], item.get('passed_tests', []), diagnostic))
    return result


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
            state['seconds_spent'] = spent + time.monotonic() - started
            state['in_flight'] = None
            self.save()

    def inspect(self, initial=False):
        tag = f'check-{self.state["attempts"]}-{time.time_ns()}'
        report = self.charge('check', self.state['check_seconds'], lambda seconds, started:
            evaluate(self.workspace, authority=self.authority, tool_root=self.root,
                     output=self.control / tag, timeout=seconds, seeds=self.state['seeds'],
                     revision=tag, initial=initial))
        if report:
            self.state['last_check'] = str(self.control / tag / 'results.json')
            visible = self.workspace / 'runs' / tag
            visible.mkdir(parents=True)
            for path in (self.control / tag).glob('*.log'):
                shutil.copy2(path, visible / path.name)
            save_json(visible / 'results.json', report)
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
        report = self.inspect(initial=state['attempts'] == 0)
        while report:
            if report['status'] in ('BLOCKED', 'NEEDS_REVIEW'):
                return self.stop(report['status'], 'Independent check cannot proceed under the fixed inputs', report)
            if report['status'] == 'MACHINE_CHECKS_PASSED':
                return self.stop('MACHINE_CHECKS_PASSED', 'Complete independent checks passed on one stopped candidate', report)
            left = state['total_seconds'] - state['seconds_spent']
            if left <= 1 or state['attempts'] >= state['max_rounds']:
                break
            mode = 'start' if state['attempts'] == 0 else 'continue'
            if state['stale_rounds'] >= state['stall_rounds']:
                if state['redesigns'] >= state['max_redesigns']:
                    return self.stop('INCOMPLETE', 'Stagnation persisted after the finite redesign budget', report)
                mode = 'redesign'
                state['redesigns'] += 1
                state['stale_rounds'] = 0
            allowance = min(state['slice_seconds'], left - min(state['check_seconds'], left / 3))
            prompt = feedback(self.authority, state, report, mode, self.workspace, allowance)
            state['attempts'] += 1
            number = state['attempts']
            directory = self.workspace / f'runs/round-{number}'
            directory.mkdir(exist_ok=True)
            (directory / 'prompt.md').write_text(prompt)
            before = {n: p.read_bytes() for n, p in files_under(self.workspace / 'madraft').items()}
            argv = command(state, self.workspace, directory / 'agent.md')
            record = dict(round=number, mode=mode, process='INTERRUPTED', exit=None,
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
            self.save()
            if result.status == 'CANCELLED':
                return self.stop('INCOMPLETE', 'Controller interrupted; candidate and charged budget retained')
            prior = report
            report = self.inspect()
            record.update(checks=state.get('last_check') if report else None,
                          check_status=(report or {}).get('status', 'INCOMPLETE'))
            if unresolved(report) == unresolved(prior):
                state['stale_rounds'] += 1
            else:
                state['stale_rounds'] = 0
            self.save()
            if result.status == 'BLOCKED' or (result.status == 'COMPLETED' and result.exit_code != 0) or not state['thread_id']:
                return self.stop('BLOCKED', 'Agent execution/authentication failed or no session ID was produced', report)
            # TIMEOUT/LOG_LIMIT are bounded failed slices, not success. A captured
            # thread may continue with actual diagnostics while budgets permit.
        return self.stop('INCOMPLETE', 'Cumulative time or maximum rounds exhausted', report)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['new', 'resume'])
    parser.add_argument('directory', type=Path)
    parser.add_argument('--execute', action='store_true')
    parser.add_argument('--model')
    parser.add_argument('--effort', choices=['low', 'medium', 'high', 'xhigh'])
    parser.add_argument('--codex')
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
                     seeds=configured_seeds(root))
        for key in ('total_seconds', 'slice_seconds', 'check_seconds'):
            state[key] = float(a[key])
        for key in ('max_rounds', 'stall_rounds', 'max_redesigns', 'max_log_bytes'):
            state[key] = int(a[key])
    else:
        if not (control / 'state.json').is_file():
            parser.error('No new-system origin record: historical runs are read-only, not warm-start inputs')
        state = json.loads((control / 'state.json').read_text())
        if state.get('format') != FORMAT or state.get('input_version') != INPUT_VERSION or state.get('workspace') != str(workspace):
            parser.error('Run origin/input version mismatch; start a fresh experiment')
        for flag in ('model', 'effort', 'codex'):
            value = getattr(args, flag)
            if value and value != state[flag]:
                parser.error('A resumed experiment retains its model, effort and Codex command; use new to change them')
    for key in ('total_seconds', 'max_rounds'):
        if getattr(args, key) is not None:
            state[key] = getattr(args, key)
    for key in ('total_seconds', 'slice_seconds', 'check_seconds', 'max_rounds', 'stall_rounds', 'max_log_bytes'):
        if not math.isfinite(state[key]) or state[key] <= 0:
            parser.error(key + ' must be finite and positive')
    if state['max_redesigns'] < 0 or not state['model']:
        parser.error('Set an explicit model and nonnegative redesign budget')
    print(f'Workspace: {workspace}\nModel: {state["model"]}; effort: {state["effort"]}\n'
          f'Budget remaining: {state["total_seconds"] - state["seconds_spent"]:.1f}s; '
          f'maximum total rounds: {state["max_rounds"]}; seeds: {state["seeds"]}', flush=True)
    if not args.execute:
        print('PREVIEW: no candidate created, no tools or agent called. Add --execute to run the independent feedback loop.')
        return 0
    if args.action == 'new':
        control = prepare(root, workspace, state)
    with (control / 'session.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        current = json.loads((control / 'state.json').read_text())
        if current['attempts'] != state['attempts'] or current['seconds_spent'] != state['seconds_spent']:
            raise ValueError('Concurrent controller changed this experiment; reload its state')
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

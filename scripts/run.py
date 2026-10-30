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
from pathlib import Path
import shlex
import shutil
import signal
import sys
import time
import uuid

from check import CONNECTIONS, FAMILIES, OBLIGATIONS, SEMANTIC_BOUNDARY, configured_seeds, evaluate, group_diagnostics
from inputs import INPUT_VERSION, copy_files, differences, files_under, fingerprint, git_revision, input_files, protected_files, save_json
from process import run_process
from tools import ROOT, environment, settings

PROMPTS = ('start', 'continue', 'redesign', 'behavior')
FORMAT = 'madraft-generate-check-4'


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
                 status='INCOMPLETE', reason='created', stale_rounds=0, redesigns=0, proof_gaps={},
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
        parts.append('Actual check outcomes:\n' + '\n'.join(
            f'{name}: {item["status"]} — {item["reason"]}' for name, item in report['checks'].items()))
        for group in report.get('diagnostics', group_diagnostics(report['checks'])):
            parts.append('Affected: ' + ', '.join(group['affected']) + '\n' +
                         json.dumps({k:v for k,v in group.items() if k not in ('affected', 'logs')}, ensure_ascii=False) +
                         '\nRaw logs: ' + ', '.join(group['logs']))
        gaps = [(n, g) for n, g in state.get('proof_gaps', {}).items() if g['repeats'] and g.get('active', True)]
        if gaps:
            parts.append('Repeated checked gaps (unrelated progress does not close these):\n' +
                         '\n'.join(f'{n}: unchanged for {g["repeats"]} checked rounds' for n,g in gaps))
        if state.get('guidance', 'B') == 'B':
            parts.append('Formula / actual-connection / implementation-task outcomes:\n' + json.dumps(report.get('assessments', {}), indent=2))
            for area, obligation in CONNECTIONS.items():
                item = report['checks'].get('connection_' + area, {})
                parts.append(f'{area}: {item.get("status", "OPEN")} ({item.get("reason", "not checked")}). {obligation}\n'
                             'Candidate declaration, not accepted coverage: ' + json.dumps(item.get('candidate_claim', {})))
            parts.append('Still-present functional placeholders (source diagnostic, not exhaustive): ' + ', '.join(report.get('placeholders', [])))
        parts.append('Next: select an unresolved required behavior or proof dependency; record the chosen next action in PROGRESS.md. '
                     'Use python3 scripts/check.py top NAME for an isolated unit, feedback for bounded mixed checks, or safety for formal composition. '
                     'Exact commands, raw errors and uncovered test names are in the independent results.json. Final requires all test families/seeds on one stopped candidate.')
        parts.append(SEMANTIC_BOUNDARY)
        parts.append('Historical functional coverage (not current-candidate acceptance):\n' +
                     json.dumps(state.get('test_history', {}), indent=2))
        if state.get('newly_checked_dependencies'):
            parts.append('Newly passing checked units, with other gaps retained: ' + ', '.join(state['newly_checked_dependencies']))
    for record in state['history'][-2:]:
        parts.append(f'Previous round {record["round"]}: {record["process"]}; independent result {record["check_status"]}.')
        parts.append('Candidate-reported approach/dependencies (UNVERIFIED):\n' + record.get('progress', '')[-3000:])
    return '\n\n'.join(parts) + '\n'


def unresolved(report):
    """Separate checked dependencies and test-family failures; never use verified counts."""
    result, dependencies = [], {}
    for name in (*OBLIGATIONS, 'safety', *('connection_' + area for area in CONNECTIONS)):
        item = (report or {}).get('checks', {}).get(name)
        if not item:
            continue
        errors = item.get('errors', [])
        # Location is shown in feedback, but moving a function is not proof progress.
        diagnostic = [{k:v for k,v in e.items() if k not in ('file', 'line', 'column')} for e in errors] or item.get('diagnostic', '')
        diagnostic = json.dumps(diagnostic, sort_keys=True)
        for path in (report.get('snapshot'), str(Path(item['log']).parent) if item.get('log') else None):
            if path:
                diagnostic = diagnostic.replace(path, '<check>')
        result.append((name, item['status'], item['reason'], diagnostic if item['status'] != 'PASS' else ''))
        for error in errors:
            if error.get('category') == 'dependency_failure' and error.get('function'):
                key = 'dependency:' + error['file'] + '::' + error['function']
                dependencies.setdefault(key, set()).add(error['message'])
    result.extend((key, 'OPEN', 'dependency_failure', '\n'.join(sorted(messages)))
                  for key, messages in sorted(dependencies.items()))
    for family in FAMILIES:
        items = [i for i in (report or {}).get('checks', {}).values() if i.get('family') == family]
        if items:
            failed = sorted({n for i in items for n in i.get('uncovered_tests', [])})
            status = 'PASS' if all(i['status'] == 'PASS' for i in items) else 'OPEN'
            reason = next((i['reason'] for i in items if i['status'] != 'PASS'), 'checked')
            result.append(('functional:' + family, status, reason, json.dumps(failed)))
    return result


def track_proofs(state, report, advance=True):
    gaps = state.setdefault('proof_gaps', {})
    checked_progress = []
    for gap in gaps.values():
        gap['active'] = False
    for name, status, reason, diagnostic in unresolved(report):
        signature = [status, reason, diagnostic]
        old = gaps.get(name, {})
        not_run = reason in ('check_budget_exhausted', 'not_run_in_feedback', 'verification_timeout', 'test_timeout',
                             'tool_unavailable', 'tool_dependency_unavailable', 'dependency_or_toolchain_unavailable',
                             'build_dependency_failed', 'interrupted', 'log_limit') or status == 'BLOCKED'
        if status == 'PASS' and old.get('signature', ['PASS'])[0] != 'PASS':
            checked_progress.append(name)
        gaps[name] = dict(signature=signature, active=True,
            repeats=(old.get('repeats', 0) + int(advance) if old.get('signature') == signature else 0)
                if status != 'PASS' and not not_run else 0,
            missed=old.get('missed', 0) + int(advance) if not_run else 0)
    state['newly_checked_dependencies'] = checked_progress
    state['stale_rounds'] = max((g['repeats'] for n,g in gaps.items() if g['active'] and not n.startswith('functional:')), default=0)


def round_plan(state):
    left = state['total_seconds'] - state['seconds_spent']
    minimum = state['min_work_seconds'] + state['handoff_seconds']
    if state.get('attempts', 0) >= state.get('max_rounds', 8):
        return 0, 'final'
    normal = min(state['slice_seconds'], left - state['check_seconds'] - state['final_seconds'])
    if normal >= minimum and state.get('attempts', 0) + 1 < state.get('max_rounds', 8):
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

    def inspect(self, initial=False, final=False):
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
        report = self.inspect(initial=state['attempts'] == 0)
        if report:
            track_proofs(state, report, advance=False)
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
            if any(g['active'] and g['missed'] >= state['stall_rounds'] for n,g in state.get('proof_gaps', {}).items()
                   if not n.startswith('functional:')):
                reason = 'Proof checks repeatedly not run: scheduling/budget issue; no model redesign inferred'
                break
            mode = 'start' if state['attempts'] == 0 else 'continue'
            if state['stale_rounds'] >= state['stall_rounds'] and state['redesigns'] < state['max_redesigns']:
                mode = 'redesign'
                state['redesigns'] += 1
                state['stale_rounds'] = 0
                for gap in state['proof_gaps'].values():
                    gap['repeats'] = 0
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
            report = self.inspect(final=next_check == 'final')
            record.update(checks=state.get('last_check') if report else None,
                          check_status=(report or {}).get('status', 'INCOMPLETE'))
            if report:
                track_proofs(state, report)
                record['newly_checked_dependencies'] = state.get('newly_checked_dependencies', [])
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
        for key in ('max_rounds', 'stall_rounds', 'max_redesigns', 'max_log_bytes'):
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
    for key in ('total_seconds', 'slice_seconds', 'check_seconds', 'final_seconds', 'min_work_seconds', 'handoff_seconds', 'max_rounds', 'stall_rounds', 'max_log_bytes'):
        if not math.isfinite(state[key]) or state[key] <= 0:
            parser.error(key + ' must be finite and positive')
    if state['slice_seconds'] < state['min_work_seconds'] + state['handoff_seconds']:
        parser.error('slice_seconds must include minimum work and handoff grace')
    if state['max_redesigns'] < 0 or not state['model']:
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
        print('Final checks, seeds, tools and scaffold are identical for A/B. B adds scripts/prompts/behavior.md and actionable connection feedback.')
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

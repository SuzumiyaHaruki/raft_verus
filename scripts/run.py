#!/usr/bin/env python3
"""One Codex session in one persistent workspace. Preview never invokes Codex."""
import argparse
import fcntl
import json
import math
import os
from pathlib import Path
import shlex
import shutil
import sys

from process import run_process
from tools import ROOT, environment, settings

INPUTS = ('AGENTS.md', 'tools.ini', 'madraft', 'requirements', '.gitignore')
CHECK_HELPERS = ('check.py', 'process.py', 'tools.py')
START = ('Read AGENTS.md and the requirements. Complete the MadRaft student work and jointly '
         'construct and repair proofs of the actual executable implementation. Start from the '
         'supplied student scaffold in the current working directory; modify only its MadRaft '
         'copy, never the preparation checkout. Record real commands and remaining gaps in '
         'PROGRESS.md and finish with RESULT.md.')
CONTINUE = ('Continue the same MadRaft experiment in this workspace. Inspect PROGRESS.md, the '
            'current source and the latest real diagnostics. Repair affected earlier results and '
            'continue implementation and proofs toward all five targets; do not restart or import a solution.')


def command(cfg, state, model='', effort='', codex=''):
    agent = cfg['agent']
    argv = [codex or cfg['tools']['codex'], '--ask-for-approval', 'never',
            '-c', 'sandbox_mode="workspace-write"',
            '-c', 'model_reasoning_effort=' + json.dumps(effort or agent['reasoning_effort']),
            'exec']
    if state.get('thread_id'):
        argv += ['resume', state['thread_id']]
    argv += ['--skip-git-repo-check', '--json']
    if model or agent['model']:
        argv += ['--model', model or agent['model']]
    argv += ['--output-last-message', str(state['last_message']), '-']
    return argv


def prepare(destination):
    destination.mkdir(parents=True, exist_ok=False)
    for name in INPUTS:
        src, dst = ROOT / name, destination / name
        if src.is_dir():
            shutil.copytree(src, dst, ignore=shutil.ignore_patterns('target', '__pycache__', '*.pyc'))
        else:
            shutil.copy2(src, dst)
    (destination / 'scripts').mkdir()
    for name in CHECK_HELPERS:
        shutil.copy2(ROOT / 'scripts' / name, destination / 'scripts' / name)
    # Share only dependency/tool caches, never another run's source or evidence.
    cfg = settings(destination)
    resolved, _ = environment(ROOT)
    for key in ('verus', 'rust_bin', 'rustup_home', 'cargo_home'):
        cfg['tools'][key] = resolved[key]
    with (destination / 'tools.ini').open('w') as file:
        cfg.write(file)
    # Keep sessions local without loading the operator's global AGENTS files.
    personal_home = Path(os.environ.get('CODEX_HOME', Path.home() / '.codex')).expanduser().resolve()
    codex_home = destination / 'runs/codex-home'
    codex_home.mkdir(parents=True, mode=0o700)
    # Reuse authentication and execution rules, never personal configuration,
    # instruction files, previous sessions, memories or agent definitions.
    for name in ('auth.json', 'rules'):
        source = personal_home / name
        if source.exists():
            (codex_home / name).symlink_to(source)
    return {'thread_id': None, 'seconds_spent': 0.0, 'attempts': 0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['new', 'resume'])
    parser.add_argument('directory', type=Path)
    parser.add_argument('--execute', action='store_true', help='Explicitly make a real Codex call')
    parser.add_argument('--model', default='')
    parser.add_argument('--effort', choices=['low', 'medium', 'high', 'xhigh'], default='')
    parser.add_argument('--codex', default='')
    parser.add_argument('--total-seconds', type=float, help='Cumulative wall-clock budget across continuations')
    args = parser.parse_args()
    dest = args.directory.expanduser().resolve()
    if dest.parent != (ROOT / 'runs').resolve():
        parser.error('Experiment directory must be a direct child of this checkout\'s runs/ directory')
    if args.action == 'new' and dest.exists():
        parser.error('New requires a fresh directory; use resume to continue it')
    if args.action == 'resume':
        state = json.loads((dest / 'runs/session.json').read_text())
        cfg = settings(dest)
    else:
        state, cfg = {'thread_id': None, 'seconds_spent': 0.0, 'attempts': 0}, settings()
    budget = args.total_seconds if args.total_seconds is not None else state.get('total_seconds', float(cfg['agent']['total_seconds']))
    if not math.isfinite(budget) or budget <= 0:
        parser.error('Budget must be a positive finite number of seconds')
    remaining = budget - state['seconds_spent']
    if remaining <= 0:
        parser.error('Cumulative budget exhausted; explicitly raise --total-seconds to continue')
    state['last_message'] = str(dest / f'runs/agent-{state["attempts"] + 1}.md')
    state['model'] = args.model or state.get('model', cfg['agent']['model'])
    state['effort'] = args.effort or state.get('effort', cfg['agent']['reasoning_effort'])
    state['codex'] = args.codex or state.get('codex', cfg['tools']['codex'])
    argv = command(cfg, state, state['model'], state['effort'], state['codex'])
    print(f'Workspace: {dest}\nCODEX_HOME: {dest / "runs/codex-home"}\n'
          f'Budget remaining: {remaining:.1f}s\n' + shlex.join(argv), flush=True)
    if not args.execute:
        print('PREVIEW: no directory created and no agent called. Add --execute to start.')
        return 0
    if args.action == 'new':
        initial = prepare(dest)
        initial.update(state)
        state = initial
    logs = dest / 'runs'
    logs.mkdir(exist_ok=True)
    with (logs / 'session.lock').open('w') as lock:
        # Ordinary OS locking prevents two resumed writers sharing one workspace.
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        statefile = logs / 'session.json'
        current = json.loads(statefile.read_text()) if statefile.exists() else None
        if current and current['attempts'] != state['attempts']:
            raise ValueError('Session changed concurrently; rerun resume with its latest state')
        if current and current.get('in_flight'):
            raise ValueError('Previous controller stopped unexpectedly: inspect its process/logs before editing in_flight in runs/session.json')
        state.update(total_seconds=budget, in_flight=True)
        # Charge the whole available budget before spawning; a controller crash
        # cannot reset spent time. On normal return, replace with actual elapsed.
        spent = state['seconds_spent']
        state['seconds_spent'] = budget
        statefile.write_text(json.dumps(state, indent=2) + '\n')
        number = state['attempts'] + 1
        (logs / f'prompt-{number}.txt').write_text(START if args.action == 'new' else CONTINUE)
        _, env = environment(dest)
        env['CODEX_HOME'] = str(dest / 'runs/codex-home')
        result = run_process(argv, dest, stdin=START if args.action == 'new' else CONTINUE,
                             timeout=remaining, max_log_bytes=int(cfg['agent']['max_log_bytes']), env=env,
                             output_paths=(logs / f'agent-{number}.jsonl', logs / f'agent-{number}.stderr.log'))
        for path, text in [(logs / f'agent-{number}.jsonl', result.stdout),
                           (logs / f'agent-{number}.stderr.log', result.stderr)]:
            if not path.exists():
                path.write_text(text)
        if result.status == 'BLOCKED':
            print(result.stderr, file=sys.stderr)
        for line in result.stdout.splitlines():
            try:
                event = json.loads(line)
            except ValueError:
                continue
            if event.get('type') == 'thread.started':
                state['thread_id'] = event['thread_id']
        state.update(seconds_spent=spent + result.elapsed_seconds, attempts=number,
                     in_flight=False, last_exit=result.exit_code, last_process=result.status)
        statefile.write_text(json.dumps(state, indent=2) + '\n')
        print(f'Agent process: {result.status}, exit={result.exit_code}; logs: {logs}')
        # A successful agent exit is not experiment acceptance.
        return 0 if result.status == 'COMPLETED' and result.exit_code == 0 else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError) as exc:
        print(f'Cannot run: {exc}', file=sys.stderr)
        sys.exit(2)

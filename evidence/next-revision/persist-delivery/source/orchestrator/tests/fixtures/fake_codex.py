#!/usr/bin/env python3
"""Offline process fixture. Never contacts Codex or a model service."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time

args = sys.argv[1:]
if '--version' in args:
    print('codex-cli FIXTURE')
    sys.exit(0)
if '--help' in args:
    print('--ask-for-approval --sandbox --json --output-schema --output-last-message --model --config --skip-git-repo-check --add-dir')
    sys.exit(0)

prompt = sys.stdin.read()
try:
    options = json.loads(prompt)
except json.JSONDecodeError:
    options = {}
    for line in prompt.splitlines():
        if line.startswith('FIXTURE_JSON='):
            options = json.loads(line.removeprefix('FIXTURE_JSON='))
scenario = options.get('scenario', 'ok')
if options.get('expect_model'):
    assert args[args.index('--model') + 1] == options['expect_model']
if options.get('expect_effort'):
    assert args[args.index('--config') + 1] == 'model_reasoning_effort=' + json.dumps(options['expect_effort'])
if options.get('stdin_copy'):
    Path(options['stdin_copy']).write_text(prompt)
if scenario in ('timeout', 'child'):
    if scenario == 'child':
        child = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(120)'])
        Path(options['pid_file']).write_text(str(child.pid))
    time.sleep(120)
if scenario in ('nonzero', 'auth', 'permission'):
    print({'nonzero': 'fixture failed', 'auth': 'authentication required',
           'permission': 'permission denied'}[scenario], file=sys.stderr)
    sys.exit(1)
if scenario == 'signal':
    os.kill(os.getpid(), 15)
if scenario == 'large':
    print('x' * 200000, file=sys.stderr)
if scenario == 'broken_jsonl':
    print('{not json')
else:
    print(json.dumps({'type': 'thread.started', 'thread_id': 'fixture-thread'}))
    if scenario == 'unknown':
        print(json.dumps({'type': 'future.event', 'value': 1}))
    print(json.dumps({'type': 'turn.completed', 'usage': {'input_tokens': 2, 'output_tokens': 3}}))

if options.get('write_path'):
    Path(options['write_path']).write_text(options.get('write_text', 'candidate'))
for change in options.get('writes', []):
    Path(change['path']).write_text(change['text'])
if options.get('build_fixture'):
    build = Path(os.environ['CARGO_TARGET_DIR'])
    build.mkdir(parents=True, exist_ok=True)
    (build/'fixture-build-output').write_text('SIMULATED local agent build artifact')
report_path = Path(args[args.index('--output-last-message') + 1])
if scenario == 'symlink_report_dir':
    outside = report_path.parent.parent.parent/'outside-report'
    outside.mkdir()
    report_path.parent.rmdir()
    report_path.parent.symlink_to(outside, target_is_directory=True)
if scenario == 'missing':
    sys.exit(0)
if scenario == 'empty':
    report_path.write_text('')
elif scenario == 'bad_schema':
    report_path.write_text('{"claimed_outcome":"candidate_ready"}')
else:
    report_path.write_text(json.dumps({
        'schema_version': '1', 'task_id': options.get('task_id', 'fixture-task'),
        'claimed_outcome': 'candidate_ready', 'summary': 'SIMULATED fixture candidate',
        'changed_files': ([options['write_path']] if options.get('write_path') else []) + [c['path'] for c in options.get('writes', [])],
        'attempted_obligation_ids': [], 'commands_reported_by_agent': [],
        'proposed_spec_changes': [], 'proposed_trust_changes': [], 'blockers': []
    }))

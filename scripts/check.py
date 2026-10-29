#!/usr/bin/env python3
"""Run real local tools on current files; retain raw diagnostics under runs/."""
import argparse
from pathlib import Path
import shlex
import re
import sys
import time

from process import run_process
from tools import ROOT, environment


def check(kind, root=ROOT, *, entry=None, test_filter='', seed=1, timeout=600):
    cfg, env = environment(ROOT)
    env['MADSIM_TEST_SEED'] = str(seed)
    cargo = str(Path(cfg['rust_bin']) / 'cargo')
    base = [cargo, 'test', '--manifest-path', 'madraft/Cargo.toml',
            '--locked', '--offline', '--release']
    if kind == 'build':
        command = base + ['--no-run']
    elif kind == 'inventory':
        command = base + ['--lib', '--', '--list']
    elif kind == 'test':
        command = base + ['--no-fail-fast', test_filter, '--', '--test-threads=1']
    else:
        entry = entry or {'definitions': 'requirements/target_checks.rs',
                          'proof': 'madraft/proof/progress.rs',
                          'safety': 'requirements/final.rs'}[kind]
        path = (root / entry).resolve()
        if not path.is_relative_to(root.resolve()):
            raise ValueError('Proof entry must be inside this workspace')
        if kind == 'safety':
            # The independent caller and targets come from this checker checkout.
            # Only its implementation import points at the delivered workspace.
            caller = (ROOT / 'requirements/final.rs').read_text()
            caller = caller.replace('\"targets.rs\"', '\"' + str(ROOT / 'requirements/targets.rs') + '\"')
            caller = caller.replace('\"../madraft/proof/implementation.rs\"', '\"' + str(root / 'madraft/proof/implementation.rs') + '\"')
            callers = ROOT / 'runs/checks'
            callers.mkdir(parents=True, exist_ok=True)
            path = callers / f'caller_{time.time_ns()}.rs'
            path.write_text(caller)
        command = [cfg['verus'], str(path), '--crate-type', 'lib',
                   '--no-cheating', '--triggers-mode', 'silent']
    logs = root / 'runs' / 'checks'
    logs.mkdir(parents=True, exist_ok=True)
    logfile = logs / f'{time.time_ns()}-{kind}.log'
    result = run_process(command, root, env=env, timeout=timeout, max_log_bytes=32 * 1024 * 1024)
    logfile.write_text('$ ' + shlex.join(command) + '\n' + result.stdout + result.stderr
                       + f'\nprocess={result.status} exit={result.exit_code}\n')
    print(result.stdout + result.stderr, end='')
    print(f'Raw check output: {logfile}')
    if result.status != 'COMPLETED' or result.exit_code != 0:
        return result.exit_code or 1
    if kind == 'test':
        # Check each selected family through inventory instead of accepting a typo.
        passed = sum(map(int, re.findall(r'test result: ok\. (\d+) passed', result.stdout)))
        if passed == 0:
            print('INCOMPLETE: no tests ran')
            return 1
    if kind == 'definitions':
        print('Definition regressions checked. No implementation safety theorem was proved.')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=['build', 'inventory', 'test', 'definitions', 'proof', 'safety', 'final'])
    parser.add_argument('selection', nargs='?', help='Test filter, or local proof entry .rs')
    parser.add_argument('--workspace', type=Path, default=ROOT, help='Check delivered files with this checkout’s tools and final callers')
    parser.add_argument('--seed', type=int, default=1)
    parser.add_argument('--timeout', type=float, default=600)
    args = parser.parse_args()
    if args.kind == 'final':
        # Both run even if one fails. There is no hard-coded missing-proof verdict.
        results = [check('test', root=args.workspace.resolve(), seed=args.seed, timeout=args.timeout),
                   check('safety', root=args.workspace.resolve(), timeout=args.timeout)]
        if all(code == 0 for code in results):
            print('Tests and aggregate proof callers passed; review execution coverage and TCB before claiming correctness.')
        return int(any(results))
    return check(args.kind, root=args.workspace.resolve(), entry=args.selection if args.kind == 'proof' else None,
                 test_filter=args.selection or '', seed=args.seed, timeout=args.timeout)


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError) as exc:
        print(f'Check unavailable: {exc}', file=sys.stderr)
        sys.exit(2)

#!/usr/bin/env python3
"""Real written-but-unacknowledged recovery checks; no simulated tools or agent calls."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import difflib
import json
from pathlib import Path
import re
import sys
import tempfile

LAB = Path(__file__).resolve().parent
ROOT = LAB.parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from process import run_process
from tools import environment


def proof_counts(output):
    return re.search(r'verification results:: (\d+) verified, (\d+) errors', output)


def proof_passed(result):
    counts = proof_counts(result.stdout + result.stderr)
    return result.exit_code == 0 and counts and int(counts[1]) > 0 and int(counts[2]) == 0


def core_rejected(result):
    output = result.stdout + result.stderr
    counts = proof_counts(output)
    return (result.exit_code not in (None, 0) and counts and int(counts[2]) > 0
            and 'src/core.rs:' in output
            and any(message in output for message in
                    ('precondition not satisfied', 'postcondition not satisfied',
                     'constructed value may fail to meet its declared type invariant',
                     'type invariant is not satisfied', 'assertion failed')))


REQUIRED_TESTS = (
    'written_but_uncompleted_kill_restart_and_continue',
    'recovered_core_preserves_rules_and_rejects_old_notifications',
    'recovery_id_boundaries_are_checked_without_fake_history',
    'recovery_mode_rejects_missing_bad_length_and_illegal_id',
)
RECOVERY_ASSERTION = 'recovery must restore the unacknowledged file value'
FILE_ASSERTION = 'post-write barrier must follow the actual op1/9 write'


def tests_passed(result):
    return (result.exit_code == 0 and all(
        re.search(r'^test ' + re.escape(name) + r' \.\.\. ok$', result.stdout, re.MULTILINE)
        for name in REQUIRED_TESTS))


def runtime_rejected(result, message):
    output = result.stdout + result.stderr
    return (result.exit_code == 101
            and f'test {REQUIRED_TESTS[0]} ... FAILED' in output
            and f'assertion `left == right` failed: {message}' in output)


def input_names():
    return ['Cargo.toml', 'Cargo.lock', 'proof.rs', 'check.py',
            *[str(p.relative_to(LAB)) for d in ('src', 'tests') for p in sorted((LAB / d).rglob('*.rs'))]]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='New directory for real command logs/results')
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    output = (args.output or LAB / 'evidence' / stamp).resolve()
    output.mkdir(parents=True, exist_ok=False)
    report = {'status': 'RUNNING', 'input_version': 'unacknowledged-recovery-0.1.0',
              'python_version': sys.version, 'checks': [], 'not_run': []}
    inputs = input_names()
    original = {name: (LAB / name).read_bytes() for name in inputs}
    for name, data in original.items():
        saved = output / 'inputs' / name
        saved.parent.mkdir(parents=True, exist_ok=True)
        saved.write_bytes(data)
    planned = ['rust-version', 'cargo-version', 'verus-version', 'positive-proof',
               'positive-tests-seed-1', 'positive-tests-seed-7',
               'core-reused-id-build', 'core-reused-id-proof',
               'adapter-before-write-proof', 'adapter-before-write-test',
               'adapter-ignore-file-proof', 'adapter-ignore-file-test',
               'restored-proof', 'restored-tests-seed-1', 'restored-tests-seed-7']
    cfg = env = None

    def save():
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')

    def check(name, argv, cwd, expectation, predicate, seed=1):
        actual_env = dict(env, MADSIM_TEST_SEED=str(seed))
        # Same package names and preserved source mtimes can alias Cargo's
        # outputs across temporary workspaces. Never reuse a mutant binary.
        actual_env['CARGO_TARGET_DIR'] = str(LAB / 'target' / 'correct' if cwd == LAB else cwd / 'target')
        result = run_process(argv, cwd, env=actual_env, timeout=180, max_log_bytes=16 * 1024 * 1024)
        (output / f'{name}.stdout.log').write_text(result.stdout)
        (output / f'{name}.stderr.log').write_text(result.stderr)
        complete = (result.status == 'COMPLETED'
                    and not result.stdout_truncated and not result.stderr_truncated)
        matches = bool(complete and predicate(result))
        report['checks'].append({
            'name': name, 'argv': argv, 'cwd': str(cwd), 'seed': seed,
            'target_dir': actual_env['CARGO_TARGET_DIR'],
            'expected': expectation, 'matches': matches, 'process_status': result.status,
            'exit_code': result.exit_code, 'elapsed_seconds': result.elapsed_seconds,
            'stdout': f'{name}.stdout.log', 'stderr': f'{name}.stderr.log',
        })
        save()
        print(f'{name}: {"PASS" if matches else "UNEXPECTED/BLOCKED"} '
              f'({result.status}, exit={result.exit_code}; {expectation})', flush=True)

    def proof(name, cwd, negative=False):
        check(name, [cfg['verus'], str(cwd / 'proof.rs'), '--crate-type', 'lib',
                     '--no-cheating', '--triggers-mode', 'silent'], cwd,
              'VERIFICATION_FAILURE' if negative else 'VERIFIED',
              core_rejected if negative else proof_passed)

    def tests(name, cwd, seed=1, failure=None):
        argv = [cargo, 'test', '--offline', '--locked', '--test', 'service']
        if failure:
            argv += [REQUIRED_TESTS[0], '--', '--exact', '--nocapture']
        else:
            argv += ['--', '--nocapture']
        check(name, argv, cwd, f'ASSERTION_FAILURE: {failure}' if failure else 'REQUIRED_TESTS_PASS',
              (lambda r: runtime_rejected(r, failure)) if failure else tests_passed, seed)

    try:
        cfg, env = environment(ROOT)
        env['CARGO_TARGET_DIR'] = str(LAB / 'target' / 'correct')
        cargo = str(Path(cfg['rust_bin']) / 'cargo')
        locked_madsim = re.search(
            r'\[\[package\]\]\nname = "madsim"\nversion = "([^"]+)"\nsource = "([^"]+)"',
            (LAB / 'Cargo.lock').read_text())
        if not locked_madsim:
            raise ValueError('Cannot identify the locked MadSim dependency')
        report['madsim'] = dict(name='madsim', version=locked_madsim[1], source=locked_madsim[2])
        report['environment'] = {key: env[key] for key in (
            'RUSTC', 'RUSTDOC', 'RUSTUP_HOME', 'CARGO_HOME', 'CARGO_TARGET_DIR',
            'CARGO_ENCODED_RUSTFLAGS')}
        for name, binary in [('rust-version', env['RUSTC']), ('cargo-version', cargo),
                             ('verus-version', cfg['verus'])]:
            check(name, [binary, '--version'], LAB, 'EXIT_ZERO', lambda r: r.exit_code == 0)
        proof('positive-proof', LAB)
        for seed in (1, 7):
            tests(f'positive-tests-seed-{seed}', LAB, seed)

        post_write = ('        let result = write_record(path, write).await;\n'
                      '        if result.is_ok() { checkpoint(control.as_ref(), write, &core).await?; }')
        pre_write = ('        checkpoint(control.as_ref(), write, &core).await?;\n'
                     '        let result = write_record(path, write).await;')
        mutations = [
            ('core-reused-id', 'src/core.rs',
             'Some(Self { completed: record.value, next: record.id + 1, pending: None,',
             'Some(Self { completed: record.value, next: record.id, pending: None,'),
            ('adapter-ignore-file', 'src/adapter.rs',
             'Mode::Recover => recover_file(path).await,',
             'Mode::Recover => Ok(Core::new()),'),
            ('adapter-before-write', 'src/adapter.rs', post_write, pre_write),
        ]
        for name, filename, before, after in mutations:
            # Only the indicated executable statement changes. Proofs, specs and
            # tests remain byte-identical; the delivered source is never edited.
            with tempfile.TemporaryDirectory(prefix=f'unacknowledged-recovery-{name}-') as temporary:
                candidate = Path(temporary)
                for relative in inputs:
                    destination = candidate / relative
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(original[relative])
                source = original[filename].decode()
                if source.count(before) != 1:
                    raise ValueError(f'{name}: expected exactly one executable mutation site')
                changed = source.replace(before, after)
                (candidate / filename).write_text(changed)
                (output / f'{name}.patch').write_text(''.join(difflib.unified_diff(
                    source.splitlines(keepends=True), changed.splitlines(keepends=True),
                    fromfile=filename, tofile=filename)))
                if any((candidate / f).read_bytes() != data for f, data in original.items() if f != filename):
                    raise ValueError(f'{name}: unrelated input changed')
                if name.startswith('core-'):
                    check(f'{name}-build', [cargo, 'test', '--offline', '--locked', '--no-run'],
                          candidate, 'COMPILES', lambda r: r.exit_code == 0)
                    proof(f'{name}-proof', candidate, negative=True)
                else:
                    proof(f'{name}-proof', candidate)
                    tests(f'{name}-test', candidate, failure=(RECOVERY_ASSERTION
                          if name == 'adapter-ignore-file' else FILE_ASSERTION))
    except Exception as exc:
        report['blocker'] = f'{type(exc).__name__}: {exc}'
        print(report['blocker'], file=sys.stderr)
    finally:
        if cfg is not None and env is not None:
            try:
                proof('restored-proof', LAB)
                for seed in (1, 7):
                    tests(f'restored-tests-seed-{seed}', LAB, seed)
            except Exception as exc:
                report['recheck_blocker'] = f'{type(exc).__name__}: {exc}'
        report['original_inputs_unchanged'] = (set(input_names()) == set(original) and all(
            (LAB / name).is_file() and (LAB / name).read_bytes() == data
            for name, data in original.items()))
        attempted = {c['name']: c for c in report['checks']}
        report['not_run'] = [name for name in planned if name not in attempted
                            or attempted[name]['process_status'] == 'BLOCKED']
        passed = (not report['not_run'] and not report.get('blocker') and not report.get('recheck_blocker')
                  and report['original_inputs_unchanged'] and all(c['matches'] for c in report['checks']))
        report['status'] = 'PASS' if passed else 'INCOMPLETE_OR_UNEXPECTED'
        save()
        print(f'{report["status"]}: {output}', flush=True)
    return 0 if passed else 1


if __name__ == '__main__':
    sys.exit(main())

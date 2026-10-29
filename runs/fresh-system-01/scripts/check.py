#!/usr/bin/env python3
"""Independent real checks with per-obligation diagnostics and bounded wall time."""
import argparse
import json
import math
from pathlib import Path
import re
import shlex
import sys
import time

from inputs import INPUT_VERSION, copy_files, differences, files_under, protected_files, save_json
from process import run_process
from tools import ROOT, environment, settings

SAFETY = ('election_safety', 'leader_append_only', 'log_matching',
          'leader_completeness', 'state_machine_safety')
OBLIGATIONS = (*SAFETY, 'execution_link', 'snapshot_observation', 'nonempty_execution')
IGNORED = {'shardkv::tests::unreliable3_4b'}
SEMANTIC_BOUNDARY = ('Verus checks the supplied definitions and imported bodies. Source linkage and '
    'nonempty witnesses reject some disconnected/vacuous candidates; they do not decide arbitrary '
    'execution/observation adequacy. Review all runtime paths, crash histories, snapshot decoders, '
    'caller preconditions and the declared host TCB. Conditional liveness remains OPEN.')


def required_tests(authority):
    text = (authority / 'requirements/test-inventory.md').read_text()
    names = set(re.findall(r'^(?:raft|kvraft|shard_ctrler|shardkv)::tests::\w+$', text, re.M))
    host = (authority / 'madraft/src/test_support.rs').read_text()
    names.update('test_support::' + name for name in re.findall(r'#\[madsim::test\]\s*(?:async\s+)?fn\s+(\w+)', host))
    return names


def classify(result, proof=False):
    text = result.stdout + result.stderr
    if result.status == 'BLOCKED':
        return 'BLOCKED', 'tool_unavailable'
    if result.status != 'COMPLETED':
        return 'OPEN', {'TIMEOUT': 'verification_timeout' if proof else 'test_timeout',
                        'LOG_LIMIT': 'log_limit', 'CANCELLED': 'interrupted'}.get(result.status, result.status.lower())
    if result.exit_code:
        if re.search(r'no matching package|failed to download|attempting to make an HTTP request|toolchain .*not installed|required rust toolchain .*not found', text, re.I):
            return 'BLOCKED', 'dependency_or_toolchain_unavailable'
        if "couldn't read" in text and 'No such file' in text:
            return 'OPEN', 'missing_entry_or_dependency'
        if proof and re.search(r'assertion failed|postcondition not satisfied|precondition not satisfied|invariant.*not satisfied|verification results::.*[1-9][0-9]* errors', text):
            return 'OPEN', 'verification_failure'
        if re.search(r'error\[E\d+\]|error:.*(?:expected|cannot|could not|couldn.t|unresolved|aborting)', text):
            return 'OPEN', 'compilation_error'
        return 'OPEN', 'verification_failure' if proof else 'test_or_build_failure'
    if proof and not re.search(r'verification results:: [1-9][0-9]* verified, 0 errors', text):
        return 'OPEN', 'no_verified_obligation'
    return 'PASS', 'checked'


def caller_text(authority, entry, obligation=None):
    text = (authority / 'requirements/final.rs').read_text()
    if obligation:
        sections = re.split(r'^// obligation: (\w+)\s*$', text, flags=re.M)
        bodies = dict(zip(sections[1::2], sections[2::2]))
        body = bodies[obligation].split('} // verus!')[0]
        text = sections[0] + body + '\n} // verus!\n'
    return text.replace('"targets.rs"', json.dumps(str(authority / 'requirements/targets.rs'))).replace(
        '"../madraft/proof/implementation.rs"', json.dumps(str(entry)))


def source_links(entry, candidate):
    """Conservative literal-module inspection, not a semantic refinement proof."""
    pending, visited, outside = [entry], set(), []
    pattern = r'(?:#\[path\s*=\s*"([^"]+)"\]\s*)?(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;'
    while pending:
        path = pending.pop().resolve()
        if path in visited or not path.is_file():
            continue
        if not path.is_relative_to(candidate.resolve()):
            outside.append(str(path)); continue
        visited.add(path)
        text = re.sub(r'//[^\n]*|/\*[\s\S]*?\*/', '', path.read_text())
        for explicit, name in re.findall(pattern, text):
            if explicit:
                pending.append(path.parent / explicit)
            else:
                pending.extend(p for p in (path.parent / (name + '.rs'), path.parent / name / 'mod.rs',
                                            path.parent / path.stem / (name + '.rs')) if p.exists())
        pending.extend(path.parent / name for name in re.findall(r'include!\(\s*"([^"]+)"\s*\)', text))
    linked = sorted(str(p.relative_to(candidate)) for p in visited if p.is_relative_to(candidate / 'madraft/src'))
    return linked, sorted(outside)


def vacuity_findings(source):
    """Known bad forms only. Absence of a finding is NOT semantic approval."""
    source = re.sub(r'//[^\n]*|/\*[\s\S]*?\*/', '', source)
    findings = []
    for name, literal in [('execution', 'false'), ('refines_executable', 'true')]:
        if re.search(r'\bfn\s+' + name + r'\b[^{}]*\{\s*' + literal + r'\s*\}', source):
            findings.append(f'{name} is the constant {literal}')
    match = re.search(r'\bfn\s+observe\b[^{}]*\{', source)
    if match:
        depth, end = 1, match.end()
        while end < len(source) and depth:
            depth += (source[end] == '{') - (source[end] == '}')
            end += 1
        body = source[match.end():end - 1]
        if re.search(r'events\s*:\s*(?:Seq::empty\(\)|seq!\[\s*\])', body):
            findings.append('observe appears to return an empty event history; inspect the definition')
    return findings


class Checks:
    def __init__(self, workspace, authority, tool_root, output, timeout, revision):
        self.workspace, self.authority = Path(workspace).resolve(), Path(authority).resolve()
        self.output = Path(output).resolve()
        self.output.mkdir(parents=True, exist_ok=False)
        self.snapshot = self.output / 'candidate'
        self.deadline = time.monotonic() + timeout
        self.revision = revision
        self.cfg, self.env = environment(tool_root)
        self.env['CARGO_TARGET_DIR'] = str(self.output.parent / 'cargo-target')
        self.env['MADSIM_TEST_NUM'] = '1'
        self.expected = required_tests(self.authority)
        self.results = {}
        # No executable from an earlier experiment is copied. These source-only
        # snapshots retain the exact inputs to each real independent check.
        source = files_under(self.workspace / 'madraft')
        self.before = {name: path.read_bytes() for name, path in source.items()}
        copy_files(self.workspace / 'madraft', self.snapshot / 'madraft', source)
        copy_files(self.authority, self.snapshot, [f'requirements/{n}' for n in files_under(self.authority / 'requirements')])
        protected = protected_files(self.authority)
        self.changed_standards = differences(self.authority, self.workspace, [n for n in protected if n != 'tools.ini'])
        expected_config = settings(self.authority)
        for key in ('verus', 'rust_bin', 'rustup_home', 'cargo_home'):
            expected_config['tools'][key] = self.cfg[key]
        actual_config = settings(self.workspace)
        for key in ('verus', 'rust_bin', 'rustup_home', 'cargo_home'):
            if actual_config.has_option('tools', key):
                actual_config['tools'][key] = str((self.workspace / Path(actual_config['tools'][key]).expanduser()).resolve())
        if {s: dict(expected_config[s]) for s in expected_config.sections()} != {s: dict(actual_config[s]) for s in actual_config.sections()}:
            self.changed_standards.append('tools.ini')
        for name in protected:
            if name.startswith('madraft/'):
                copy_files(self.authority, self.snapshot, [name])
        # Build overrides are not a way to drop required tests or substitute code.
        self.changed_standards += [f'madraft/{n}' for n in source
                                   if n == 'build.rs' or '.cargo' in Path(n).parts or n.startswith('rust-toolchain')]
        self.protected_before = {n: (self.workspace / n).read_bytes() for n in protected if (self.workspace / n).is_file()}
        self.requirements_before = {n: p.read_bytes() for n, p in files_under(self.snapshot / 'requirements').items()}

    def record(self, name, status, reason, **extra):
        item = dict(status=status, reason=reason, input_version=INPUT_VERSION,
                    candidate_revision=self.revision, command=[], log=None, scope=name, **extra)
        self.results[name] = item
        return item

    def run(self, name, argv, *, proof=False, seed=None, scope=None):
        left = self.deadline - time.monotonic()
        if left <= 0:
            return self.record(name, 'OPEN', 'check_budget_exhausted')
        env = self.env.copy()
        if seed is not None:
            env['MADSIM_TEST_SEED'] = str(seed)
        result = run_process(argv, self.snapshot, env=env, timeout=left, max_log_bytes=16 * 1024 * 1024)
        log = self.output / (name + '.log')
        log.write_text('$ ' + shlex.join(argv) + '\n' + result.stdout + result.stderr
                       + f'\nprocess={result.status} exit={result.exit_code}\n')
        status, reason = classify(result, proof)
        item = self.record(name, status, reason, process=result.status, exit_code=result.exit_code,
                           elapsed_seconds=result.elapsed_seconds, seed=seed,
                           diagnostic=(result.stderr + result.stdout)[-10000:])
        item.update(command=argv, log=str(log), scope=scope or name)
        item['environment'] = {k: env[k] for k in ('RUSTC', 'RUSTDOC', 'RUSTUP_HOME', 'CARGO_HOME',
            'CARGO_TARGET_DIR', 'CARGO_ENCODED_RUSTFLAGS', 'MADSIM_TEST_SEED', 'MADSIM_TEST_NUM',
            'MADSIM_TEST_CHECK_DETERMINISTIC') if k in env}
        return item

    def cargo(self, kind, seed=1, selection=''):
        argv = [str(Path(self.cfg['rust_bin']) / 'cargo'), 'test', '--manifest-path',
                'madraft/Cargo.toml', '--locked', '--offline', '--release', '--lib']
        name = kind if kind != 'test' else f'tests_seed_{seed}'
        if kind == 'build':
            argv += ['--no-run']
        elif kind == 'inventory':
            argv += ['--', '--list']
        else:
            argv += ['--no-fail-fast', selection, '--', '--test-threads=1']
        item = self.run(name, argv, seed=seed)
        if item.get('log') and kind in ('inventory', 'test'):
            text = Path(item['log']).read_text()
            expected = {n for n in self.expected if selection in n}
            if kind == 'inventory':
                observed = set(re.findall(r'^(\S+): test$', text, re.M))
                missing = sorted(expected - observed)
            else:
                observed = dict(re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)', text, re.M))
                missing = sorted(n for n in expected - IGNORED if observed.get(n) != 'ok')
                item['passed_tests'] = sorted(n for n, state in observed.items() if state == 'ok')
                item['ignored_tests'] = sorted(n for n, state in observed.items() if state == 'ignored')
            item['uncovered_tests'] = missing
            if item['status'] == 'PASS' and (missing or not observed):
                item.update(status='OPEN', reason='missing_test_coverage' if missing else 'empty_test_selection')
            if kind == 'test' and not selection and not expected:
                item.update(status='OPEN', reason='missing_required_inventory')
        return item

    def verus(self, kind, entry=None, obligation=None):
        proof_entry = self.snapshot / (entry or 'madraft/proof/progress.rs')
        name = obligation or kind
        if not proof_entry.resolve().is_relative_to(self.snapshot):
            return self.record(name, 'NEEDS_REVIEW', 'proof_entry_outside_candidate')
        if kind in ('safety', 'top'):
            optional = self.snapshot / f'madraft/proof/{obligation}.rs'
            implementation = optional if obligation and optional.is_file() else self.snapshot / 'madraft/proof/implementation.rs'
            proof_entry = self.output / (name + '-caller.rs')
            proof_entry.write_text(caller_text(self.snapshot, implementation, obligation))
        elif kind == 'definitions':
            proof_entry = self.snapshot / 'requirements/target_checks.rs'
        argv = [self.cfg['verus'], str(proof_entry), '--crate-type', 'lib',
                '--no-cheating', '--triggers-mode', 'silent']
        return self.run(name, argv, proof=True, scope='entire imported compilation unit; no function filtering')

    def linkage(self):
        entry = self.snapshot / 'madraft/proof/implementation.rs'
        if not entry.exists():
            return self.record('linkage_review', 'OPEN', 'missing_implementation_entry',
                               diagnostic='Create the execution/observation/refinement interface and its real dependencies; do not supply assumed invariants.')
        linked, outside = source_links(entry, self.snapshot)
        findings = vacuity_findings(entry.read_text())
        status, reason = ('PASS', 'limited_source_checks_only')
        if outside:
            status, reason = 'NEEDS_REVIEW', 'external_source_import'
        elif findings or not linked:
            status, reason = 'OPEN', 'suspect_vacuity' if findings else 'no_executable_source_link'
        return self.record('linkage_review', status, reason, linked_executable_files=linked,
                           external_paths=outside, findings=findings, semantic_boundary=SEMANTIC_BOUNDARY)

    def finish(self, mode, required):
        self.changed_standards += [n for n, body in self.protected_before.items()
            if not (self.workspace / n).is_file() or (self.workspace / n).read_bytes() != body]
        self.changed_standards += ['checked requirements/' + n for n, body in self.requirements_before.items()
            if not (self.snapshot / 'requirements' / n).is_file() or (self.snapshot / 'requirements' / n).read_bytes() != body]
        self.changed_standards = sorted(set(self.changed_standards))
        after = files_under(self.workspace / 'madraft')
        changed = sorted(n for n in self.before.keys() | after.keys() if n not in self.before or n not in after
                         or self.before[n] != after[n].read_bytes())
        snapshot_changed = differences(self.snapshot / 'madraft', self.workspace / 'madraft')
        # Expected overlays of protected files are reported separately, never accepted.
        snapshot_changed = [n for n in snapshot_changed if 'madraft/' + n not in self.changed_standards]
        statuses = [self.results[n]['status'] for n in required]
        if self.changed_standards or 'NEEDS_REVIEW' in statuses:
            status = 'NEEDS_REVIEW'
        elif 'BLOCKED' in statuses:
            status = 'BLOCKED'
        elif changed or snapshot_changed or any(s != 'PASS' for s in statuses):
            status = 'INCOMPLETE'
        else:
            status = 'MACHINE_CHECKS_PASSED' if mode == 'final' else 'PASS'
        if mode == 'initial' and status == 'PASS':
            status = 'NEEDS_REVIEW'  # A working solution is not the expected starting scaffold.
        report = dict(status=status, mode=mode, input_version=INPUT_VERSION,
                      candidate_revision=self.revision, workspace=str(self.workspace),
                      checker_authority=str(self.authority), snapshot=str(self.snapshot),
                      source_files=sorted(self.before), checks=self.results, required=required,
                      changed_standards=self.changed_standards,
                      concurrent_changes=sorted(set(changed + snapshot_changed)),
                      liveness='OPEN — not accepted by safety checks', semantic_boundary=SEMANTIC_BOUNDARY)
        save_json(self.output / 'results.json', report)
        return report


def evaluate(workspace, *, authority=ROOT, tool_root=ROOT, output=None, timeout=600,
             seeds=None, revision='standalone', initial=False):
    output = output or ROOT / 'runs/checks' / str(time.time_ns())
    checks = Checks(workspace, authority, tool_root, output, timeout, revision)
    checks.cargo('build')
    checks.cargo('inventory')
    for seed in ([1] if initial else seeds or configured_seeds(tool_root)):
        checks.cargo('test', seed, 'raft::tests::initial_election_2a' if initial else '')
    checks.verus('proof')
    for obligation in OBLIGATIONS:
        checks.verus('top', obligation=obligation)
    checks.linkage()
    # Always verify the actual combined caller. A table of partial passes cannot
    # substitute for this composition, even if each separate module verified.
    checks.verus('safety')
    required = [n for n in checks.results if n != 'proof']
    return checks.finish('initial' if initial else 'final', required)


def configured_seeds(root):
    return parse_seeds(settings(root).get('checks', 'seeds', fallback='1,2,3,4,5'))


def parse_seeds(text):
    seeds = list(dict.fromkeys(int(s.strip()) for s in text.split(',')))
    if not seeds or any(s < 0 or s >= 2**64 for s in seeds):
        raise ValueError('Seeds must be unsigned 64-bit integers')
    return seeds


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=['build', 'inventory', 'test', 'definitions', 'proof', 'top', 'safety', 'final'])
    parser.add_argument('selection', nargs='?', help='Test filter, proof file, or top-level obligation')
    parser.add_argument('--workspace', type=Path, default=ROOT)
    parser.add_argument('--seed', type=int, default=1, help='Single-test seed; final uses --seeds or tools.ini')
    parser.add_argument('--seeds', help='Explicit complete final seed set, e.g. 1,2,3,4,5')
    parser.add_argument('--timeout', type=float, default=600, help='Wall time for this entire check invocation')
    parser.add_argument('--json', action='store_true', help='Print structured diagnostics; raw output remains in logs')
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('Timeout must be finite and positive')
    output = ROOT / 'runs/checks' / str(time.time_ns())
    if args.kind in ('final', 'safety'):
        report = evaluate(args.workspace, output=output, timeout=args.timeout,
                          seeds=parse_seeds(args.seeds) if args.seeds else None)
    else:
        checks = Checks(args.workspace, ROOT, ROOT, output, args.timeout, 'standalone-' + output.name)
        if args.kind in ('build', 'inventory', 'test'):
            checks.cargo(args.kind, args.seed, args.selection or '')
        else:
            if args.kind == 'top' and args.selection not in OBLIGATIONS:
                parser.error('top requires one of: ' + ', '.join(OBLIGATIONS))
            checks.verus(args.kind, entry=args.selection if args.kind == 'proof' else None,
                         obligation=args.selection if args.kind == 'top' else None)
        report = checks.finish(args.kind, list(checks.results))
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        for name, result in report['checks'].items():
            print(f"{name}: {result['status']} ({result['reason']})")
            if result.get('log'):
                print('  ' + result['log'])
        print(report['status'] + ': ' + str(output / 'results.json'))
        print(SEMANTIC_BOUNDARY)
    return 0 if report['status'] in ('PASS', 'MACHINE_CHECKS_PASSED') else 2 if report['status'] in ('BLOCKED', 'NEEDS_REVIEW') else 1


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError) as exc:
        print(f'Check unavailable: {exc}', file=sys.stderr)
        sys.exit(2)

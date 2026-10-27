"""Acceptance, diagnosis and resume regressions for incremental development.

Generation uses the executable fixture. A separate test invokes installed Verus
when available; it is never replaced with simulated proof evidence.
"""
import copy
import io
import json
from pathlib import Path
import shutil
import sys
import threading
import unittest
from unittest.mock import patch
from contextlib import redirect_stdout

from orchestrator.cli import main
from orchestrator.config import load_config
from orchestrator.feedback import classify, repair_context
from orchestrator.scheduler import load_tasks, run_plan, run_task, task_fingerprint
from orchestrator.verifier_runner import current_proof_evidence, verify_profile
from orchestrator.workspace import snapshot
from orchestrator.tests import test_scheduler as scheduler_cases

REPO, FIXTURES = scheduler_cases.REPO, scheduler_cases.FIXTURES


class RevisionTests(unittest.TestCase):
    setUp = scheduler_cases.SchedulerTests.setUp
    tearDown = scheduler_cases.SchedulerTests.tearDown
    save_task = scheduler_cases.SchedulerTests.save_task
    prompt = scheduler_cases.SchedulerTests.prompt
    execute = scheduler_cases.SchedulerTests.execute

    def test_cli_task_plan_and_resume_need_no_review_file(self):
        self.use_checks(self.real_test())
        self.config['codex'].update(model='gpt-6.1-sol', model_reasoning_effort='high')
        self.prompt(expect_model='gpt-6.1-sol', expect_effort='high')
        def invoke(arguments):
            output = io.StringIO()
            with patch('orchestrator.cli.load_config', return_value=self.config), redirect_stdout(output):
                exit_code = main(['--root', str(self.root), *arguments])
            result = json.loads(output.getvalue())
            self.assertEqual(exit_code, 0, result)
            self.assertEqual(result['status'], 'PASSED', result)
            self.assertFalse(result['full_acceptance'])
            return result
        task = invoke(['run-task', '--task', self.task['id'], '--execute-codex'])
        self.assertEqual(task['generation_kind'], 'SIMULATED')
        first = invoke(['run-plan', '--tasks', self.task['id'], '--execute-codex'])
        self.assertEqual(first['generation_kind'], 'SIMULATED')
        resumed = invoke(['resume', '--run-id', first['run_id'], '--execute-codex'])
        self.assertEqual(resumed['candidate_path'], first['candidate_path'])
        self.assertEqual(resumed['agent_calls'], first['agent_calls'])
        self.assertEqual((self.root/'src/lib.rs').read_text(), 'fn candidate() {}')

    def test_disabled_or_unimplemented_isolation_stops_plan_before_checks(self):
        for change in ({'allow_real_codex': False}, {'isolation': 'strict'}):
            with self.subTest(change=change):
                config = copy.deepcopy(self.config)
                config['workflow'].update(change)
                with patch('orchestrator.scheduler.run_codex', side_effect=AssertionError('no generation')):
                    result = run_plan(self.root, config, execute_codex=True)
                self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))
                self.assertFalse((self.root/'evidence/runs').exists())

    def test_disabling_execution_also_stops_resume(self):
        self.use_checks(self.real_test())
        first = run_plan(self.root, self.config, execute_codex=True)
        self.assertEqual(first['status'], 'PASSED', first)
        state_path = self.root/'evidence/runs'/first['run_id']/'plan-state.json'
        before = state_path.read_bytes()
        self.config['workflow']['allow_real_codex'] = False
        result = run_plan(self.root, self.config, execute_codex=True, resume_id=first['run_id'])
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))
        self.assertEqual(state_path.read_bytes(), before)

    def real_test(self, purpose='test'):
        # This subprocess actually checks a source property. It does not claim
        # Rust or mathematical verification; those have separate integration runs.
        return dict(id='actual-test', purpose=purpose, adapter='command-text-v1',
            argv=[sys.executable, '-c', 'from pathlib import Path; assert "candidate" in Path("src/lib.rs").read_text(); print("test result: ok. 1 passed")'],
            entrypoints=['src/lib.rs'], expected_targets=['candidate source regression'],
            success_pattern=r'test result: ok', evidence_kind='REAL_LOCAL', timeout_seconds=3)

    def use_checks(self, *profiles, obligations=()):
        self.config['profiles'] = list(profiles)
        self.task['check_profile_ids'] = [p['id'] for p in profiles]
        self.task['target_obligation_ids'] = list(obligations)
        self.save_task()
        self.task = load_tasks(self.root, self.config)[0]

    def test_real_tests_without_proof_targets_complete(self):
        self.use_checks(self.real_test())
        result = self.execute()
        self.assertEqual(result['status'], 'PASSED', result)
        self.assertEqual(result['generation_kind'], 'SIMULATED')
        self.assertEqual(result['evidence_kind'], 'REAL_LOCAL')
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertFalse(result['full_acceptance'])

    def test_typecheck_only_completes_without_theorem(self):
        self.use_checks(self.real_test(purpose='typecheck'))
        result = self.execute()
        self.assertEqual(result['status'], 'PASSED', result)
        self.assertEqual(result['obligation_ids_checked'], [])

    def test_simulated_check_cannot_close_obligation(self):
        self.task['target_obligation_ids'] = ['PO-FIXTURE']
        self.save_task()
        result = self.execute()
        self.assertEqual(result['status'], 'BLOCKED', result)
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertEqual(result['evidence_kind'], 'SIMULATED')

    def test_empty_test_selection_missing_entrypoint_and_tool_fail(self):
        cases = [dict(argv=[sys.executable, '-c', 'print("test result: ok. 0 passed")']),
                 dict(entrypoints=['missing.rs']), dict(argv=['/no/installed/checker'])]
        for index, change in enumerate(cases):
            with self.subTest(change=change):
                result = verify_profile(self.root, dict(self.real_test(), **change), self.outside / str(index))
                self.assertNotIn(result['result'], {'PASS', 'TYPECHECKED', 'MEASURED'})

    def test_negative_compiler_failure_not_accepted_as_proof_failure(self):
        profile = dict(id='negative', purpose='proof', adapter='verus-calibrated-text-v1',
            argv=[str(FIXTURES/'fake_verus.py'), '--errors', '1', '--compiler-error'],
            entrypoints=['src/lib.rs'], expected_targets=['wrong_increment'],
            evidence_kind='SIMULATED', expect_rejection=True,
            rejection_pattern='postcondition not satisfied: wrong_increment',
            calibration={'tool_version': 'verus FIXTURE 1',
                'summary_pattern': r'fixture summary: (?P<verified>\d+) verified; (?P<errors>\d+) errors',
                'positive_evidence': 'SIMULATED', 'negative_evidence': 'SIMULATED'})
        result = verify_profile(self.root, profile, self.outside/'negative')
        self.assertEqual(result['result'], 'FAIL')

    def test_fault_feedback_preserves_trace_seed_states_and_reproduction(self):
        (self.root/'trace.json').write_text('{"events": ["append empty"]}')
        profile = dict(self.real_test(), scenario={'seed': 17, 'trace_path': 'trace.json'},
            argv=[sys.executable, '-c', 'print("state before=[1,2], after=[1]; assertion failed"); raise SystemExit(1)'])
        report = verify_profile(self.root, profile, self.outside/'fault')
        feedback = repair_context(self.task, report, 'candidate revision')
        self.assertEqual(feedback['category'], 'scenario')
        self.assertEqual(feedback['scenario']['seed'], 17)
        self.assertEqual(Path(feedback['scenario']['saved_trace']).read_text(), '{"events": ["append empty"]}')
        self.assertIn('state before', feedback['error_excerpt'])
        self.assertIn('failed_command', feedback)
        self.assertIn('required_reruns', feedback)

    def test_proof_failure_and_solver_timeout_keep_uncertainty(self):
        proof = classify(dict(purpose='proof', result='FAIL'))
        timeout = classify(dict(purpose='proof', result='TIMEOUT'))
        self.assertIn('not itself a counterexample', proof['diagnosis'])
        self.assertEqual(timeout['category'], 'proof-timeout')
        self.assertIn('do not infer a code defect', timeout['diagnosis'])
        self.assertEqual(timeout['action'], 'repair')

    def test_measurements_without_threshold_and_noisy_samples_are_not_pass(self):
        samples = {'kind': 'measurement', 'workload': 'core', 'samples_ns': [10, 2000, 11],
                   'reference_version': None, 'noise': 'high'}
        profile = dict(self.real_test(), purpose='benchmark',
            argv=[sys.executable, '-c', 'print(' + repr(json.dumps(samples)) + ')'])
        profile.pop('success_pattern')
        result = verify_profile(self.root, profile, self.outside/'bench')
        self.assertEqual(result['result'], 'MEASURED')
        self.assertEqual(result['performance_verdict'], 'INCONCLUSIVE')
        self.assertEqual(result['measurements'], [samples])
        result = verify_profile(self.root, dict(profile, argv=[sys.executable, '-c', 'print("no data")']), self.outside/'missing-bench')
        self.assertEqual(result['result'], 'FAIL')
        for index, sample in enumerate(({'kind': 'measurement'},
                dict(samples, samples_ns=[float('nan'), 1]), dict(samples, samples_ns=[]))):
            result = verify_profile(self.root, dict(profile,
                argv=[sys.executable, '-c', 'print(' + repr(json.dumps(sample)) + ')']), self.outside/f'empty-data-{index}')
            self.assertEqual(result['result'], 'FAIL')

    def test_task_binding_ignores_unrelated_prose_but_tracks_code_and_checker(self):
        self.config['tasks'][self.task['id']]['context'] = ['src/lib.rs']
        task = load_tasks(self.root, self.config)[0]
        before = task_fingerprint(self.root, self.config, task)
        (self.root/'README.md').write_text('Unrelated explanation.')
        self.assertEqual(before, task_fingerprint(self.root, self.config, load_tasks(self.root, self.config)[0]))
        (self.root/'src/lib.rs').write_text('fn candidate_changed() {}')
        self.assertNotEqual(before, task_fingerprint(self.root, self.config, task))
        after = task_fingerprint(self.root, self.config, task)
        (self.root/'orchestrator/checker.py').write_text('# changed acceptance policy')
        self.assertNotEqual(after, task_fingerprint(self.root, self.config, task))

    def test_ordinary_repair_cannot_weaken_checker(self):
        self.prompt(write_path='orchestrator/checker.py', write_text='# accept everything')
        result = self.execute()
        self.assertEqual(result['status'], 'REVIEW_REQUIRED')

    def test_planned_task_blocks_before_generation(self):
        self.config['tasks'][self.task['id']].update(status='planned', blocked_reason='Missing node state', checks=[])
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))
        self.assertEqual(result['reason'], 'Missing node state')

    def test_fixture_mode_does_not_authorize_an_arbitrary_executable(self):
        self.config['codex'].update(fixture=True, executable='/usr/bin/false')
        result = run_task(self.root, self.config, self.task['id'], True, fixture_mode=True)
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))

    def test_failed_correctness_check_defers_benchmark_and_keeps_repair_routing(self):
        marker = self.outside/'benchmark-was-started'
        failed = dict(self.real_test(), argv=[sys.executable, '-c', 'print("assertion failed"); raise SystemExit(1)'])
        benchmark = dict(self.real_test(), id='benchmark', purpose='benchmark',
            argv=[sys.executable, '-c', 'from pathlib import Path; Path(' + repr(str(marker)) + ').write_text("started")'])
        self.use_checks(failed, benchmark)
        result = self.execute()
        self.assertEqual(result['status'], 'FAILED', result)
        self.assertEqual(result['agent_calls'], 2)
        self.assertFalse(marker.exists())
        self.assertEqual(result['deferred_checks'][0]['profile_id'], 'benchmark')

    def test_interrupted_plan_resumes_same_candidate_and_rechecks_changes(self):
        self.use_checks(self.real_test())
        self.config['codex']['fixture'] = True
        self.config['workflow']['allow_real_codex'] = False
        second = copy.deepcopy(self.config['tasks'][self.task['id']])
        second['after'] = [self.task['id']]
        self.config['tasks']['second'] = second
        (self.root/'input/second.md').write_text('Check the preserved candidate after the first task.')
        event = threading.Event()
        import orchestrator.scheduler as scheduler
        original = scheduler.run_task
        def stop_after_first(*args, **kwargs):
            result = original(*args, **kwargs)
            event.set()
            return result
        with patch.object(scheduler, 'run_task', side_effect=stop_after_first):
            first = run_plan(self.root, self.config, ['second'], True, cancel_event=event, fixture_mode=True)
        self.assertEqual(first['status'], 'CANCELLED', first)
        candidate = Path(first['candidate_path'])
        (candidate/'src/lib.rs').write_text('fn candidate_changed_but_still_valid() {}')
        event.clear()
        second = run_plan(self.root, self.config, execute_codex=True, resume_id=first['run_id'], fixture_mode=True)
        self.assertEqual(second['status'], 'PASSED', second)
        self.assertEqual(second['candidate_path'], first['candidate_path'])
        self.assertEqual(second['agent_calls'], 2)
        self.assertIn('recheck_run_id', second['completed'][self.task['id']])
        self.assertIn('candidate_changed', (candidate/'src/lib.rs').read_text())
        self.assertEqual((self.root/'src/lib.rs').read_text(), 'fn candidate() {}')
        (candidate/'src/lib.rs').write_text('broken source')
        third = run_plan(self.root, self.config, execute_codex=True, resume_id=first['run_id'], fixture_mode=True)
        self.assertEqual(third['status'], 'FAILED', third)
        self.assertEqual(third['agent_calls'], 4)
        self.assertEqual(third['completed'][self.task['id']]['status'], 'INVALIDATED')

    def test_resume_rejects_mutation_between_independent_checks(self):
        self.use_checks(self.real_test())
        self.config['codex']['fixture'] = True
        first = run_plan(self.root, self.config, execute_codex=True, fixture_mode=True)
        self.assertEqual(first['status'], 'PASSED', first)
        candidate = Path(first['candidate_path'])
        (candidate/'src/lib.rs').write_text('fn candidate_changed() {}')
        import orchestrator.scheduler as scheduler
        original = scheduler.verify_profile
        def mutate_after_check(*args, **kwargs):
            result = original(*args, **kwargs)
            (candidate/'src/lib.rs').write_text('fn candidate_mutated_after_check() {}')
            return result
        with patch.object(scheduler, 'verify_profile', side_effect=mutate_after_check):
            resumed = run_plan(self.root, self.config, execute_codex=True, resume_id=first['run_id'], fixture_mode=True)
        self.assertEqual(resumed['status'], 'BLOCKED', resumed)
        self.assertIn('changed during', resumed['reason'])
        self.assertIsNone(resumed['completed'][self.task['id']]['current_fingerprint'])

    def test_typecheck_plus_installed_real_verus_proof_and_current_evidence(self):
        installed = load_config(REPO)
        executable = Path(installed['toolchain']['verus_executable'])
        if not executable.is_file():
            self.skipTest('Installed real Verus missing; no simulated proof substituted')
        (self.root/'src/lib.rs').write_text('use vstd::prelude::*;\nverus! { pub proof fn local_proof() ensures 1int + 1 == 2 {} }\n')
        proof = dict(id='local-proof', purpose='proof', adapter='verus-calibrated-text-v1',
            argv=[str(executable), '--crate-type', 'lib', '--no-cheating', '--output-json', '--time-expanded', 'src/lib.rs'],
            entrypoints=['src/lib.rs'], proof_entrypoints=['src/lib.rs'], expected_targets=['local_proof'],
            expected_obligation_ids=['LOCAL-UNIT'], proof_targets={'LOCAL-UNIT': 'lib::local_proof'},
            calibration=installed['verus_calibration'], tool_env=installed['toolchain']['tool_env'],
            evidence_kind='REAL_LOCAL', timeout_seconds=30)
        typecheck = dict(proof, id='typecheck', purpose='typecheck', expected_obligation_ids=[], proof_targets={})
        self.use_checks(typecheck, proof, obligations=['LOCAL-UNIT'])
        self.config['budget']['task_timeout_seconds'] = 90
        result = self.execute()
        self.assertEqual(result['status'], 'PASSED', result)
        self.assertEqual(result['obligation_ids_checked'], ['LOCAL-UNIT'])
        self.assertFalse(result['full_acceptance'])
        candidate = Path(result['candidate_path'])
        # Task checks live under the outer evidence tree; copy a report into the
        # candidate's evidence index to exercise the ordinary CHECKED lookup.
        actual = self.root/'evidence/runs'/result['run_id']/'attempt-1/local-proof.check.json'
        target = candidate/'evidence/runs/copied-real-check'
        target.mkdir(parents=True)
        (target/'local-proof.check.json').write_bytes(actual.read_bytes())
        effective = dict(proof, timeout_seconds=min(proof['timeout_seconds'], 90))
        self.assertTrue(current_proof_evidence(candidate, effective, 'LOCAL-UNIT'))
        (candidate/'src/lib.rs').write_text((candidate/'src/lib.rs').read_text() + '// changed proof input\n')
        self.assertFalse(current_proof_evidence(candidate, effective, 'LOCAL-UNIT'))

    def test_unrelated_real_smoke_cannot_close_named_source_obligation(self):
        installed = load_config(REPO)
        executable = Path(installed['toolchain']['verus_executable'])
        if not executable.is_file():
            self.skipTest('Installed real Verus missing; no simulated proof substituted')
        (self.root/'src/lib.rs').write_text('use vstd::prelude::*; verus! { pub proof fn unproved_target() ensures false {} }')
        (self.root/'src/smoke.rs').write_text('use vstd::prelude::*; verus! { pub proof fn smoke() ensures 1int == 1 {} }')
        profile = dict(id='smoke-not-target', purpose='proof', adapter='verus-calibrated-text-v1',
            argv=[str(executable), '--crate-type', 'lib', '--no-cheating', '--output-json', '--time-expanded', 'src/smoke.rs'],
            entrypoints=['src/lib.rs', 'src/smoke.rs'], proof_entrypoints=['src/smoke.rs'],
            expected_targets=['unproved_target'], expected_obligation_ids=['LOCAL-UNIT'],
            proof_targets={'LOCAL-UNIT': 'lib::unproved_target'}, calibration=installed['verus_calibration'],
            tool_env=installed['toolchain']['tool_env'], evidence_kind='REAL_LOCAL', timeout_seconds=30)
        result = verify_profile(self.root, profile, self.outside/'unrelated')
        self.assertEqual(result['result'], 'FAIL', result)
        self.assertEqual(result['obligation_ids_checked'], [])

    def test_runtime_cfg_binding_is_not_proof_source_but_proof_cfg_still_requires_review(self):
        installed = load_config(REPO)
        executable = Path(installed['toolchain']['verus_executable'])
        if not executable.is_file():
            self.skipTest('Installed real Verus missing; no simulated proof substituted')
        source = self.root/'src/lib.rs'
        source.write_text('use vstd::prelude::*; verus! { pub proof fn target() ensures 1int == 1 {} }')
        (self.root/'src/runtime.rs').write_text('#[cfg(test)] mod tests {}')
        profile = dict(id='proof-cfg', purpose='proof', adapter='verus-calibrated-text-v1',
            argv=[str(executable), '--crate-type', 'lib', '--no-cheating', '--output-json', '--time-expanded', 'src/lib.rs'],
            entrypoints=['src/lib.rs'], proof_entrypoints=['src/lib.rs'], source_inputs=['src/lib.rs', 'src/runtime.rs'],
            expected_targets=['target'], expected_obligation_ids=['LOCAL-UNIT'], proof_targets={'LOCAL-UNIT': 'lib::target'},
            calibration=installed['verus_calibration'], tool_env=installed['toolchain']['tool_env'],
            evidence_kind='REAL_LOCAL', timeout_seconds=30)
        result = verify_profile(self.root, profile, self.outside/'runtime-only-cfg')
        self.assertEqual(result['result'], 'PASS', result)
        source.write_text(source.read_text() + '\n#[cfg(test)] fn hidden() {}\n')
        result = verify_profile(self.root, profile, self.outside/'actual-proof-cfg')
        self.assertEqual(result['result'], 'ERROR', result)
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertTrue(any(finding.get('token') == 'cfg' for finding in result['policy_findings']))

    def test_real_slice_consumer_rejects_weakened_postcondition_and_stronger_precondition(self):
        installed = load_config(REPO)
        executable = Path(installed['toolchain']['verus_executable'])
        if not executable.is_file():
            self.skipTest('Installed real Verus missing; no simulated proof substituted')
        profile = next(check for check in installed['profiles'] if check['id'] == 'reconcile-proof')
        candidate = self.outside/'real-slice'
        shutil.copytree(REPO/'crates', candidate/'crates')
        for name in ('Cargo.toml', 'Cargo.lock'):
            shutil.copyfile(REPO/name, candidate/name)
        source = candidate/'crates/raft-verified/src/reconcile.rs'
        original = source.read_text()
        start = original.index('\n    ensures', original.index('pub fn reconcile_log'))
        end = original.index('\n{', start)
        cases = [original[:start] + '\n    ensures true,' + original[end:],
                 original[:start] + '\n    requires previous < old(log).len(),' + original[start:]]
        for index, changed in enumerate(cases):
            with self.subTest(change=('weaker postcondition', 'stronger precondition')[index]):
                source.write_text(changed)
                result = verify_profile(candidate, profile, self.outside/f'contract-change-{index}')
                self.assertEqual(result['result'], 'FAIL', result)
                self.assertEqual(result['obligation_ids_checked'], [])
                self.assertFalse(result.get('function_results', {}).get('reconcile::check_reconcile_contract', True))


if __name__ == '__main__':
    unittest.main()

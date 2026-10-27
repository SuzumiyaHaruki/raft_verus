"""Optional comparison routing; deterministic measurements are explicitly fixtures."""
import json
import sys
import unittest
from orchestrator.performance import compare
from orchestrator.scheduler import run_task
from orchestrator.verifier_runner import verify_profile
from orchestrator.tests import test_revision as revision

class PerformanceTests(unittest.TestCase):
    setUp = revision.RevisionTests.setUp
    tearDown = revision.RevisionTests.tearDown
    save_task = revision.RevisionTests.save_task
    prompt = revision.RevisionTests.prompt
    real_test = revision.RevisionTests.real_test
    use_checks = revision.RevisionTests.use_checks

    def measurement(self, samples):
        return dict(kind='measurement', workload='fixture', build_mode='release',
                    environment='SIMULATED fixed hardware', boundary='fixture operation', samples_ns=samples)

    def rule(self):
        path = self.outside/'reference.json'; path.write_text(json.dumps([self.measurement([10]*5)]))
        return dict(reference_path=str(path), reference_candidate='SIMULATED fixture reference v1',
                    max_regression_fraction=0.1, max_relative_dispersion=0.05)

    def test_measurement_noisy_and_confirmed_regression_are_distinct(self):
        self.assertEqual(compare([self.measurement([20]*5)], None, self.root)['performance_verdict'], 'INCONCLUSIVE')
        rule = self.rule()
        noisy = compare([self.measurement([10, 100, 10, 200, 10])], rule, self.root)
        self.assertEqual(noisy['performance_verdict'], 'INCONCLUSIVE')
        measured = compare([self.measurement([20]*5)], rule, self.root)
        self.assertEqual(measured['performance_verdict'], 'REGRESSION')
        self.assertEqual(measured['comparisons'][0]['ratio'], 2)
        self.assertEqual(compare([self.measurement([10]*5)], rule, self.root)['performance_verdict'], 'NO_REGRESSION')

    def test_incomparable_workload_units_and_configuration_rejected(self):
        rule = self.rule()
        for changed in (dict(self.measurement([20]*5), workload='different'),
                        dict(self.measurement([20]*5), build_mode='debug'),
                        dict(self.measurement([20]*5), environment='another machine'),
                        dict(kind='measurement', workload='fixture', build_mode='release',
                             environment='SIMULATED fixed hardware', boundary='fixture operation', samples=[20]*5, unit='seconds')):
            with self.assertRaises((ValueError, KeyError)):
                compare([changed], rule, self.root)
        with self.assertRaises(ValueError): compare([], rule, self.root)

    def test_optimization_patch_returns_through_correctness_before_measurement(self):
        test = self.real_test()
        script = 'from pathlib import Path; import json; s=Path("src/lib.rs").read_text(); r=' + repr(self.measurement([20]*5)) + '; r["samples_ns"]=[20 if "slow" in s else 5]*5; print(json.dumps(r))'
        benchmark = dict(id='measurement', purpose='benchmark', adapter='command-text-v1',
            argv=[sys.executable, '-c', script], entrypoints=['src/lib.rs'], expected_targets=['fixture'],
            expected_obligation_ids=[], comparison=self.rule(), evidence_kind='SIMULATED', timeout_seconds=3)
        self.use_checks(test, benchmark)
        self.config['codex'].update(fixture=True, fixture_edits=[
            dict(path='src/lib.rs', text='fn candidate_slow() {}'),
            dict(path='src/lib.rs', text='broken but fast'),
            dict(path='src/lib.rs', text='fn candidate_fast() {}')])
        result = run_task(self.root, self.config, self.task['id'], True, fixture_mode=True)
        self.assertEqual(result['status'], 'PASSED', result.get('reason'))
        self.assertEqual(result['agent_calls'], 3)
        run = self.root/'evidence/runs'/result['run_id']
        self.assertFalse((run/'attempt-2/measurement.check.json').exists())
        self.assertIn('BOUNDED OPTIMIZATION', (run/'attempt-2/prompt.txt').read_text())
        self.assertEqual(json.loads((run/'attempt-2/actual-test.check.json').read_text())['result'], 'FAIL')
        self.assertEqual(json.loads((run/'attempt-3/actual-test.check.json').read_text())['result'], 'PASS')
        self.assertEqual(json.loads((run/'attempt-3/measurement.check.json').read_text())['performance_verdict'], 'NO_REGRESSION')

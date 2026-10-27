import json
from pathlib import Path
import sys
import tempfile
import unittest

from orchestrator.codex_runner import discover_codex, run_codex
from orchestrator.config import validate_codex_settings
from orchestrator.verifier_runner import verify_profile

REPO = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).parent/'fixtures'
SCHEMA = REPO/'orchestrator/wire/agent_report.schema.json'


class AdapterTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.candidate = self.root/'candidate'
        self.candidate.mkdir()
        self.evidence = self.root/'evidence'
        self.evidence.mkdir()
        (self.candidate/'lib.rs').write_text('fn candidate() {}')

    def tearDown(self):
        self.tmp.cleanup()

    def codex(self, scenario='ok'):
        config = {'executable': str(FIXTURES/'fake_codex.py'), 'call_timeout_seconds': .15 if scenario == 'timeout' else 3}
        return run_codex(config, self.candidate, json.dumps({'scenario': scenario}), SCHEMA, self.evidence)

    def profile(self, scenario='ok', **overrides):
        p = dict(id='offline-fixture', adapter='fake-json-v1',
                 argv=[sys.executable, str(FIXTURES/'fake_verifier.py'), '--scenario', scenario, '--target', 'lemma'],
                 entrypoints=['lib.rs'], expected_targets=['lemma'], expected_obligation_ids=['PO-FIXTURE'],
                 timeout_seconds=.15 if scenario == 'timeout' else 3, max_log_bytes=1048576,
                 evidence_kind='SIMULATED')
        p.update(overrides)
        return p

    def test_cli_capability_discovery(self):
        cfg = {'executable': str(FIXTURES/'fake_codex.py'), 'supported_versions': ['codex-cli FIXTURE']}
        self.assertEqual(discover_codex(cfg, self.candidate, self.evidence)['status'], 'READY')

    def test_codex_success_is_candidate_only(self):
        result = self.codex()
        self.assertEqual(result['status'], 'CANDIDATE')
        self.assertEqual(result['agent_report']['claimed_outcome'], 'candidate_ready')
        self.assertEqual(result['thread_ids'], ['fixture-thread'])
        self.assertIsNotNone(result['usage'])
        self.assertNotIn('result', result['agent_report'])

    def test_explicit_model_and_effort_reach_the_subprocess(self):
        settings = {'executable': str(FIXTURES/'fake_codex.py'),
                    'supported_versions': ['codex-cli FIXTURE'],
                    'model': 'gpt-6.1-sol', 'model_reasoning_effort': 'high'}
        self.assertEqual(discover_codex(settings, self.candidate, self.evidence)['status'], 'READY')
        result = run_codex(settings, self.candidate,
            json.dumps({'expect_model': 'gpt-6.1-sol', 'expect_effort': 'high'}), SCHEMA, self.evidence)
        self.assertEqual(result['status'], 'CANDIDATE', result)
        self.assertEqual(result['model_requested'], 'gpt-6.1-sol')
        self.assertEqual(result['reasoning_effort_requested'], 'high')
        self.assertIsNone(result['model_resolved'])

    def test_unset_model_and_effort_do_not_override_codex_defaults(self):
        result = self.codex()
        process = json.loads((self.evidence/'codex.process.json').read_text())
        self.assertNotIn('--model', process['argv'])
        self.assertNotIn('--config', process['argv'])
        self.assertIsNone(result['model_requested'])
        self.assertIsNone(result['reasoning_effort_requested'])

    def test_invalid_model_settings_are_rejected(self):
        for key in ('model', 'model_reasoning_effort'):
            for value in ('', 'two words', '\nhigh', 1, True, None):
                with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                    validate_codex_settings({key: value})

    def test_unknown_event_preserved_with_warning(self):
        result = self.codex('unknown')
        self.assertTrue(result['warnings'])
        self.assertTrue(any(e['type'] == 'future.event' for e in result['events']))
        self.assertNotEqual(result['status'], 'PASS')

    def test_agent_local_build_outputs_do_not_mutate_candidate_sources(self):
        result = run_codex({'executable': str(FIXTURES/'fake_codex.py')}, self.candidate,
            json.dumps({'build_fixture': True}), SCHEMA, self.evidence)
        self.assertEqual(result['status'], 'CANDIDATE')
        self.assertTrue((self.evidence/'agent-build/fixture-build-output').is_file())
        self.assertFalse((self.candidate/'target').exists())

    def test_protocol_errors(self):
        for scenario in ('broken_jsonl', 'missing', 'empty', 'bad_schema', 'symlink_report_dir'):
            with self.subTest(scenario=scenario):
                case = self.candidate/scenario
                case.mkdir()
                out = self.evidence/scenario
                out.mkdir()
                result = run_codex({'executable': str(FIXTURES/'fake_codex.py')}, case,
                                   json.dumps({'scenario': scenario}), SCHEMA, out)
                self.assertEqual(result['status'], 'PROTOCOL_ERROR')

    def test_real_schema_rejects_trusted_result_in_agent_report(self):
        from orchestrator.schemas import validate_schema, ValidationError
        report = self.codex()['agent_report']
        report['result'] = 'PASS'
        with self.assertRaises(ValidationError):
            validate_schema(report, SCHEMA)

    def test_fake_verifier_pass_is_simulated(self):
        report = verify_profile(self.candidate, self.profile(), self.evidence)
        self.assertEqual(report['result'], 'PASS')
        self.assertEqual(report['evidence_kind'], 'SIMULATED')

    def test_verifier_failure_modes_never_pass(self):
        for scenario in ('fail', 'timeout', 'crash', 'unsupported', 'no_targets', 'empty'):
            with self.subTest(scenario=scenario):
                out = self.evidence/scenario
                out.mkdir()
                result = verify_profile(self.candidate, self.profile(scenario), out)
                self.assertNotEqual(result['result'], 'PASS')

    def test_missing_verifier(self):
        report = verify_profile(self.candidate, self.profile(argv=['/no/raft-verifier']), self.evidence)
        self.assertEqual(report['result'], 'BLOCKED')

    def test_checked_targets_cannot_be_unrelated(self):
        report = verify_profile(self.candidate, self.profile(expected_targets=['unproved']), self.evidence)
        self.assertNotEqual(report['result'], 'PASS')

    def test_changes_during_check_invalidate_result(self):
        profile = self.profile()
        profile['argv'] += ['--touch', str(self.candidate/'lib.rs')]
        report = verify_profile(self.candidate, profile, self.evidence)
        self.assertNotEqual(report['result'], 'PASS')

    def test_agent_claim_does_not_overrule_independent_failure(self):
        agent = self.codex()
        report = verify_profile(self.candidate, self.profile('fail'), self.evidence)
        self.assertEqual(agent['agent_report']['claimed_outcome'], 'candidate_ready')
        self.assertEqual(report['result'], 'FAIL')

    def calibrated_profile(self, verified=1, errors=0, **options):
        profile = self.profile(adapter='verus-calibrated-text-v1', expected_obligation_ids=[],
            argv=[str(FIXTURES/'fake_verus.py'), '--verified', str(verified), '--errors', str(errors)],
            calibration={'tool_version': 'verus FIXTURE 1',
                         'summary_pattern': r'fixture summary: (?P<verified>\d+) verified; (?P<errors>\d+) errors',
                         'positive_evidence': 'SIMULATED_POSITIVE', 'negative_evidence': 'SIMULATED_NEGATIVE'},
            rejection_pattern=r'postcondition not satisfied: wrong_increment')
        profile.update(options)
        return profile

    def test_zero_verification_is_not_a_proof_pass(self):
        result = verify_profile(self.candidate, self.calibrated_profile(verified=0), self.evidence)
        self.assertEqual(result['result'], 'FAIL')

    def test_typecheck_is_separate_from_proof_pass(self):
        result = verify_profile(self.candidate, self.calibrated_profile(verified=0, purpose='typecheck'), self.evidence)
        self.assertEqual(result['result'], 'TYPECHECKED')
        self.assertEqual(result['evidence_kind'], 'SIMULATED')
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertEqual(result['proof_result'], 'NOT_RUN')

    def test_negative_smoke_requires_actual_rejection(self):
        rejected = self.evidence/'rejected'
        rejected.mkdir()
        result = verify_profile(self.candidate,
            self.calibrated_profile(verified=0, errors=1, expect_rejection=True), rejected)
        self.assertEqual(result['result'], 'PASS')
        self.assertEqual(result['obligation_ids_checked'], [])
        accepted = self.evidence/'accepted'
        accepted.mkdir()
        result = verify_profile(self.candidate, self.calibrated_profile(expect_rejection=True), accepted)
        self.assertEqual(result['result'], 'FAIL')


if __name__ == '__main__':
    unittest.main()

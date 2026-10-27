"""Fixture approvals below exist only in temporary offline test directories."""
import copy
import datetime
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from orchestrator.evidence import create_run, write_json
from orchestrator.scheduler import (approval_bindings, dependency_ready, plan, run_task,
                                    task_fingerprint, task_order, validate_approval, validate_tasks, load_tasks, render_prompt)
from orchestrator.workspace import digest_bytes

REPO = Path(__file__).resolve().parents[2]
FIXTURES = Path(__file__).parent/'fixtures'


class SchedulerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='raft-scheduler-test-')
        self.outside = Path(self.tmp.name)
        self.root = self.outside/'repo'
        for path in ('input', 'orchestrator', 'src'):
            (self.root/path).mkdir(parents=True, exist_ok=True)
        (self.root/'input/requirements.md').write_text('Status: DRAFT. Offline fixture contract and TCB.')
        (self.root/'input/proofs.md').write_text('Status: DRAFT. Offline fixture proof target.')
        (self.root/'input/trust.md').write_text('Status: DRAFT. Offline fixture TCB.')
        shutil.copyfile(REPO/'input/instructions.md', self.root/'input/instructions.md')
        (self.root/'orchestrator/checker.py').write_text('# fixture controller')
        (self.root/'src/lib.rs').write_text('fn candidate() {}')
        self.budget = dict(max_agent_calls=3, max_repair_rounds=2,
                           call_timeout_seconds=3, task_timeout_seconds=15, max_log_bytes=1048576)
        self.config = {
            'workflow': {'allow_real_codex': True, 'isolation': 'local-development',
                         'protected_paths': ['input', 'orchestrator'],
                         'context': ['input', 'src']},
            'codex': {'executable': str(FIXTURES/'fake_codex.py'), 'supported_versions': ['codex-cli FIXTURE']},
            'budget': dict(self.budget), 'toolchain': {'status': 'TEST_ONLY'},
            'profiles': [dict(id='fake', adapter='fake-json-v1',
                argv=[str(FIXTURES/'fake_verifier.py'), '--target', 'lemma'],
                entrypoints=['src/lib.rs'], expected_targets=['lemma'], expected_obligation_ids=[],
                evidence_kind='SIMULATED', timeout_seconds=3)]}
        self.task = dict(id='fixture-task', task_type='safety_lemma', goal='offline fixture',
                         dependency_ids=[], contract_digest=digest_bytes((self.root/'input/requirements.md').read_bytes()),
                         input_paths=['src/lib.rs'], permitted_change_paths=['src'], forbidden_change_paths=['input'],
                         expected_outputs=['src/lib.rs'], target_obligation_ids=['PO-FIXTURE'], check_profile_ids=['fake'],
                         prompt_template='input/fixture-task.md', budget=dict(self.budget),
                         human_review_triggers=['specification change'])
        self.save_task()
        self.prompt()
        self.task = load_tasks(self.root, self.config)[0]

    def tearDown(self):
        self.tmp.cleanup()

    def save_task(self):
        self.config.setdefault('tasks', {})[self.task['id']] = dict(
            after=self.task['dependency_ids'], edit=self.task['permitted_change_paths'],
            outputs=self.task['expected_outputs'], obligations=self.task['target_obligation_ids'],
            checks=self.task['check_profile_ids'])

    def prompt(self, **options):
        payload = dict(task_id=self.task['id'])
        payload.update(options)
        (self.root/'input/fixture-task.md').write_text('FIXTURE_JSON='+json.dumps(payload)+'\n')

    def approve_fixture(self, **changes):
        # Never used with a real CLI: config points exclusively to executable fixtures.
        approval = dict(approval_id='offline-test-record', approved_by='Offline unit test fixture',
                        approved_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                        lifecycle_status='HUMAN_APPROVED', is_template=False,
                        approved_task_scope=[self.task['id']], **approval_bindings(self.root, self.config))
        approval.update(changes)
        path = self.outside/'fixture-approval.json'
        path.write_text(json.dumps(approval))
        return path

    def execute(self):
        return run_task(self.root, self.config, self.task['id'], execute_codex=True,
                        approval_file=self.approve_fixture())

    def test_prompt_is_plain_markdown_and_includes_real_diagnostics(self):
        log = self.outside/'actual.stderr'
        log.write_text('postcondition failed at candidate.rs:12')
        prompt = render_prompt(self.root, self.task, {'checks': [dict(profile_id='fake',
            result='FAIL', reason='verification failed', evidence_paths=[str(log)])]})
        self.assertIn('Task: fixture-task', prompt)
        self.assertIn('postcondition failed at candidate.rs:12', prompt)
        self.assertNotIn('CONTROLLER_TASK_JSON', prompt)
        self.assertNotIn('contract_digest', prompt)
        self.assertIn('Use English', prompt)

    def test_duplicate_unknown_and_cycle_dependencies(self):
        for tasks in ([self.task, self.task],
                      [dict(self.task, dependency_ids=['unknown'])],
                      [dict(self.task, dependency_ids=[self.task['id']])]):
            with self.assertRaises(ValueError):
                task_order(tasks)

    def test_missing_input_and_invalid_profile(self):
        for change in ({'input_paths': ['missing']}, {'check_profile_ids': ['missing']}):
            task = dict(self.task, **change)
            self.assertTrue(validate_tasks(self.root, [task], self.config))

    def test_plan_validate_and_dry_run_never_launch_codex(self):
        self.config['codex']['executable'] = '/no/real/codex'
        self.assertEqual(validate_tasks(self.root, [self.task], self.config), [])
        self.assertEqual(plan(self.root, self.config)['status'], 'DRAFT_NOT_APPROVED')
        result = run_task(self.root, self.config, self.task['id'])
        self.assertEqual((result['status'], result['agent_calls']), ('DRY_RUN', 0))

    def test_missing_example_inside_stale_and_wrong_scope_approvals(self):
        with self.assertRaises(ValueError):
            validate_approval(self.root, self.config, self.task, None)
        for change in ({'is_template': True}, {'approval_id': 'EXAMPLE'},
                       {'contract_digest': '0'*64}, {'approved_task_scope': ['different-task']}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                validate_approval(self.root, self.config, self.task, self.approve_fixture(**change))
        inside = self.root/'approval.json'
        shutil.copyfile(self.approve_fixture(), inside)
        with self.assertRaises(ValueError):
            validate_approval(self.root, self.config, self.task, inside)

    def test_disabled_bootstrap_and_strict_isolation_block(self):
        self.config['workflow']['allow_real_codex'] = False
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))
        self.config['workflow'].update(allow_real_codex=True, isolation='strict')
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))

    def test_dependency_not_complete_blocks(self):
        dependency = dict(self.task, id='dependency')
        self.config['tasks']['dependency'] = dict(self.config['tasks']['fixture-task'])
        (self.root/'input/dependency.md').write_text('Offline dependency fixture.')
        self.task['dependency_ids'] = ['dependency']
        self.save_task()
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))

    def test_fake_end_to_end_evidence_and_repeated_run_unique(self):
        first = self.execute()
        second = self.execute()
        self.assertEqual(first['status'], 'PASSED', first)
        self.assertEqual(first['evidence_kind'], 'SIMULATED')
        self.assertFalse(first['full_acceptance'])
        self.assertNotEqual(first['run_id'], second['run_id'])
        run_dir = self.root/'evidence/runs'/first['run_id']
        self.assertTrue((run_dir/'attempt-1/prompt.txt').exists())
        self.assertTrue((run_dir/'attempt-1/candidate.patch').exists())
        self.assertTrue((run_dir/'attempt-1/agent_report.json').exists())
        self.assertTrue((run_dir/'attempt-1/fake.check.json').exists())
        self.assertFalse(dependency_ready(self.root, self.config, self.task['id']))

    def test_claimed_success_fails_and_identical_failure_stops_early(self):
        self.config['profiles'][0]['argv'] += ['--scenario', 'fail']
        result = self.execute()
        self.assertEqual(result['status'], 'FAILED', result)
        self.assertEqual(result['agent_calls'], 2)
        self.assertIn('repeated', result['reason'])

    def test_one_call_budget_and_zero_budget(self):
        self.config['budget']['max_agent_calls'] = 0
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('BLOCKED', 0))
        self.config['budget']['max_agent_calls'] = 1
        self.config['profiles'][0]['argv'] += ['--scenario', 'fail']
        result = self.execute()
        self.assertEqual((result['status'], result['agent_calls']), ('FAILED', 1))

    def test_protected_candidate_mutation_requires_review(self):
        self.prompt(write_path='input/requirements.md', write_text='changed')
        result = self.execute()
        self.assertEqual(result['status'], 'REVIEW_REQUIRED', result)
        self.assertEqual((self.root/'input/requirements.md').read_text(), 'Status: DRAFT. Offline fixture contract and TCB.')

    def test_build_only_check_cannot_close_proof_task(self):
        self.config['profiles'][0].update(adapter='command-text-v1',
            argv=[sys.executable, '-c', 'print("fixture build complete")'],
            success_pattern='fixture build complete', expected_obligation_ids=['PO-FIXTURE'],
            evidence_kind='REAL_LOCAL')
        result = self.execute()
        self.assertEqual(result['status'], 'BLOCKED', result)
        self.assertFalse(result['full_acceptance'])

    def test_old_controller_evidence_invalidated_by_input_change(self):
        run_id, directory = create_run(self.root, 'fixture-record-only')
        # A synthetic controller record tests cache invalidation, not proof validity.
        write_json(directory/'conclusion.json', {'task_id': self.task['id'], 'status': 'PASSED',
            'evidence_kind': 'REAL_LOCAL', 'current_fingerprint': task_fingerprint(self.root, self.config, self.task)})
        self.assertTrue(dependency_ready(self.root, self.config, self.task['id']))
        (self.root/'src/lib.rs').write_text('different input')
        self.assertFalse(dependency_ready(self.root, self.config, self.task['id']))


if __name__ == '__main__':
    unittest.main()

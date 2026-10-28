"""Continuation scenarios use the real subprocess fixture, never a live model."""
import copy
import json
from pathlib import Path
import threading
import unittest
from unittest.mock import patch
from orchestrator.scheduler import run_plan, edit_saved_plan
from orchestrator.tests import test_revision as revision


class ContinuationTests(unittest.TestCase):
    setUp = revision.RevisionTests.setUp
    tearDown = revision.RevisionTests.tearDown
    save_task = revision.RevisionTests.save_task
    prompt = revision.RevisionTests.prompt
    real_test = revision.RevisionTests.real_test
    use_checks = revision.RevisionTests.use_checks

    def configure(self, edits, calls=3, redesigns=1):
        self.use_checks(self.real_test())
        self.config['codex'].update(fixture=True, fixture_edits=[dict(path='src/lib.rs', text=s) for s in edits])
        self.config['budget'].update(max_agent_calls=calls, max_repair_rounds=calls-1,
                                    max_redesigns=redesigns, stalled_attempts=2)

    def run_plan(self, resume=None, **kw):
        return run_plan(self.root, self.config, execute_codex=True, fixture_mode=True, resume_id=resume, **kw)

    def test_each_new_experiment_generates_from_the_supplied_foundation(self):
        self.configure(['fn candidate_generated() {}'])
        import orchestrator.scheduler as scheduler
        original = scheduler.run_codex
        initial_sources = []
        def inspect_start(config, candidate, *args, **kwargs):
            initial_sources.append((candidate/'src/lib.rs').read_text())
            return original(config, candidate, *args, **kwargs)
        with patch.object(scheduler, 'run_codex', side_effect=inspect_start):
            first = self.run_plan()
            first_source = Path(first['candidate_path'])/'src/lib.rs'
            first_source.write_text('fn previous_experiment_only() {}')
            second = self.run_plan()
        self.assertEqual((first['status'], second['status']), ('PASSED', 'PASSED'))
        self.assertNotEqual(first['run_id'], second['run_id'])
        self.assertNotEqual(first['candidate_path'], second['candidate_path'])
        foundation = (self.root/'src/lib.rs').read_text()
        self.assertEqual(initial_sources, [foundation, foundation])
        self.assertEqual(first_source.read_text(), 'fn previous_experiment_only() {}')
        self.assertEqual((first['agent_calls'], second['agent_calls']), (1, 1))
        self.assertEqual((first['resume_count'], second['resume_count']), (0, 0))

    def test_exhaustion_resume_and_explicit_extension(self):
        self.configure(['broken', 'still broken', 'fn candidate() {}'], calls=2)
        first = self.run_plan()
        self.assertEqual((first['status'], first['agent_calls']), ('FAILED', 2))
        resumed = self.run_plan(first['run_id'])
        self.assertEqual(resumed['agent_calls'], 2)
        self.assertEqual(resumed['usage']['call_limit'], 2)
        edit_saved_plan(self.root, self.config, first['run_id'], self.task['id'],
                        extend=dict(call_limit=1, repair_limit=1, seconds_limit=5, redesign_limit=0))
        repaired = self.run_plan(first['run_id'])
        self.assertEqual((repaired['status'], repaired['agent_calls']), ('PASSED', 3), repaired['reason'])
        record = repaired['task_usage'][self.task['id']]
        self.assertEqual(record['redesigns'], 1)
        prompt = Path(record['attempts'][-1]['artifacts'], 'prompt.txt').read_text()
        self.assertIn('BOUNDED LOCAL REDESIGN', prompt)
        self.assertIn('Attempt 1', prompt)
        self.assertIn('actual-test=FAIL', prompt)
        self.assertEqual(len(repaired['extensions']), 1)

    def test_interrupt_after_launch_is_charged_and_candidate_retained(self):
        self.configure(['fn candidate_first() {}', 'fn candidate_second() {}'])
        import orchestrator.scheduler as scheduler
        original = scheduler.run_codex
        def interrupt(*args, **kwargs):
            original(*args, **kwargs)
            raise KeyboardInterrupt()
        with patch.object(scheduler, 'run_codex', side_effect=interrupt):
            with self.assertRaises(KeyboardInterrupt): self.run_plan()
        path = next((self.root/'evidence/runs').glob('*/plan-state.json'))
        saved = json.loads(path.read_text())
        self.assertEqual(saved['usage']['calls'], 1)
        self.assertGreaterEqual(saved['usage']['active_seconds'], 3)
        self.assertIn('candidate_first', Path(saved['candidate_path'], 'src/lib.rs').read_text())
        resumed = self.run_plan(saved['run_id'])
        self.assertEqual((resumed['status'], resumed['agent_calls']), ('PASSED', 2), resumed.get('reason'))
        self.assertEqual(len(resumed['task_usage'][self.task['id']]['attempts']), 2)
        self.assertIn('candidate_second', Path(resumed['candidate_path'], 'src/lib.rs').read_text())

    def test_active_time_not_paused_wall_time_and_plan_cap(self):
        self.configure(['broken'], calls=3)
        self.config['plan_budget'] = dict(max_agent_calls=1, active_seconds=30)
        first = self.run_plan()
        used = first['usage']['active_seconds']
        with patch('orchestrator.scheduler.time.time', return_value=10**12):
            resumed = self.run_plan(first['run_id'])
        self.assertEqual(resumed['agent_calls'], 1)
        self.assertEqual(resumed['usage']['active_seconds'], used)
        self.assertLess(used, 30)

    def test_cross_task_regression_repaired_in_current_task_scope(self):
        self.configure(['fn candidate() {}'])
        first_id = self.task['id']
        second = copy.deepcopy(self.config['tasks'][first_id]); second['after'] = [first_id]
        self.config['tasks']['second'] = second
        (self.root/'input/second.md').write_text('Develop the next behavior without discarding the existing candidate.')
        self.config['codex']['fixture_task_edits'] = {'second': [dict(path='src/lib.rs', text='broken B'),
            dict(path='src/lib.rs', text='fn candidate_with_B() {}')]}
        result = self.run_plan()
        self.assertEqual((result['status'], result['agent_calls']), ('PASSED', 3), result.get('reason'))
        record = result['task_usage']['second']
        prompt = Path(record['attempts'][1]['artifacts'], 'prompt.txt').read_text()
        self.assertIn('actual-test=FAIL', prompt)
        self.assertIn('Repair the current change', prompt)
        change = Path(record['attempts'][1]['artifacts'], 'attempt.patch').read_text()
        self.assertIn('-broken B', change)
        self.assertIn('+fn candidate_with_B', change)
        self.assertTrue(all(v['status'] == 'PASSED' for v in result['completed'].values()))
        self.assertIn('candidate_with_B', Path(result['candidate_path'], 'src/lib.rs').read_text())
        self.assertEqual((self.root/'src/lib.rs').read_text(), 'fn candidate() {}')

    def test_regression_after_interruption_repairs_later_task(self):
        self.configure(['fn candidate() {}'])
        second = copy.deepcopy(self.config['tasks'][self.task['id']]); second['after'] = [self.task['id']]
        self.config['tasks']['second'] = second
        (self.root/'input/second.md').write_text('Keep earlier behavior while adding B.')
        self.config['codex']['fixture_task_edits'] = {'second': [dict(path='src/lib.rs', text='broken B'),
            dict(path='src/lib.rs', text='fn candidate_with_B() {}')]}
        import orchestrator.scheduler as scheduler
        original = scheduler.run_codex
        def interrupt(config, candidate, prompt, *args, **kw):
            result = original(config, candidate, prompt, *args, **kw)
            if 'Task: second' in prompt: raise KeyboardInterrupt()
            return result
        with patch.object(scheduler, 'run_codex', side_effect=interrupt):
            with self.assertRaises(KeyboardInterrupt): self.run_plan()
        path = next((self.root/'evidence/runs').glob('*/plan-state.json'))
        saved = json.loads(path.read_text())
        resumed = self.run_plan(saved['run_id'])
        self.assertEqual((resumed['status'], resumed['agent_calls']), ('PASSED', 3), resumed.get('reason'))
        self.assertEqual(resumed['task_usage'][self.task['id']]['calls'], 1)
        self.assertIn('candidate_with_B', Path(resumed['candidate_path'], 'src/lib.rs').read_text())

    def test_rewind_preserves_abandoned_patch_and_rechecks(self):
        self.configure(['fn candidate() {}'])
        first = self.run_plan()
        candidate = Path(first['candidate_path'])
        (candidate/'src/lib.rs').write_text('broken abandoned design')
        result = edit_saved_plan(self.root, self.config, first['run_id'], self.task['id'], rewind='actual-test')
        self.assertEqual(result['status'], 'REWOUND')
        self.assertEqual(result['checks'][0]['result'], 'PASS')
        patch_text = (self.root/'evidence/runs'/result['run_id']/'candidate.patch').read_text()
        self.assertIn('broken abandoned design', patch_text)
        self.assertEqual((candidate/'src/lib.rs').read_text(), 'fn candidate() {}')
        self.assertEqual((self.root/'src/lib.rs').read_text(), 'fn candidate() {}')

    def test_resume_keeps_mixed_historical_generation_labels(self):
        self.configure(['broken', 'fn candidate() {}'], calls=2)
        import orchestrator.scheduler as scheduler
        original = scheduler.run_codex
        def interrupt(*args, **kwargs):
            original(*args, **kwargs)
            raise KeyboardInterrupt()
        with patch.object(scheduler, 'run_codex', side_effect=interrupt):
            with self.assertRaises(KeyboardInterrupt): self.run_plan()
        path = next((self.root/'evidence/runs').glob('*/plan-state.json'))
        saved = json.loads(path.read_text())
        # Synthetic historical metadata only: no REAL_CODEX process is called.
        saved['task_usage'][self.task['id']]['generation_kinds'] = ['REAL_CODEX']
        saved['task_usage'][self.task['id']]['attempts'][0]['generation_kind'] = 'REAL_CODEX'
        path.write_text(json.dumps(saved))
        result = self.run_plan(saved['run_id'])
        self.assertEqual(result['status'], 'PASSED', result.get('reason'))
        self.assertEqual(result['generation_kind'], 'MIXED')
        self.assertEqual(result['generation_kinds'], ['REAL_CODEX', 'SIMULATED'])
        self.assertEqual(result['completed'][self.task['id']]['check_evidence_kind'], 'REAL_LOCAL')

    def test_tests_only_checkpoint_does_not_claim_open_proof_passed(self):
        self.configure(['fn candidate() {}'], calls=1)
        proof = dict(self.real_test(), id='open-proof', purpose='proof', adapter='fake-json-v1',
            argv=[str(revision.FIXTURES/'fake_verifier.py'), '--target', 'lemma', '--scenario', 'fail'],
            expected_targets=['lemma'], evidence_kind='SIMULATED')
        self.use_checks(self.real_test(), proof)
        result = self.run_plan()
        self.assertEqual(result['status'], 'FAILED')
        saved = result['task_usage'][self.task['id']]['checkpoints']
        self.assertIn('actual-test', saved)
        self.assertNotIn('open-proof', saved)
        self.assertEqual(saved['actual-test']['checked_scope'], 'actual-test')
        self.assertFalse(result['completed'])

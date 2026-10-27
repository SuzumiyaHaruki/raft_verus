"""Module context, selected-step scope and completion boundaries."""
import copy
import io
import json
import re
import subprocess
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

from orchestrator.cli import main
from orchestrator.config import load_config
from orchestrator.materials import module_assignment
from orchestrator.scheduler import load_tasks, plan, render_prompt, run_plan, run_task, task_order
from orchestrator.tests import test_scheduler as scheduler_cases

REPO = scheduler_cases.REPO


class ModuleTests(unittest.TestCase):
    setUp = scheduler_cases.SchedulerTests.setUp
    tearDown = scheduler_cases.SchedulerTests.tearDown
    save_task = scheduler_cases.SchedulerTests.save_task
    prompt = scheduler_cases.SchedulerTests.prompt

    def use_module(self):
        self.path = self.root/'input/replication.md'
        self.path.write_text(
            '# Replication\n\nStatus: DRAFT. Module: INCOMPLETE.\n'
            'Complete leader and follower replication over repeated events.\n'
            '```text\n## Step: Example, not a task\n```\n'
            '## Step: Repair reconciliation\n\n'
            'Preserve matching tails.\n### Checks\nUse the independent checker.\n'
            'FIXTURE_JSON={"task_id":"fixture-task"}\n'
            '## Step: Implement peer responses\n\n'
            'UNSELECTED: implement response backtracking.\n')
        self.config['tasks']['fixture-task'].update(
            module='input/replication.md', step='Repair reconciliation', context=['src'])
        self.config['codex']['fixture'] = True
        self.task = load_tasks(self.root, self.config)[0]

    def test_prompt_contains_module_and_exactly_selected_step(self):
        self.use_module()
        prompt = render_prompt(self.root, self.task)
        self.assertIn('Complete leader and follower replication', prompt)
        self.assertIn('Preserve matching tails', prompt)
        self.assertIn('### Checks', prompt)
        self.assertNotIn('UNSELECTED', prompt)
        self.assertNotIn('## Step: Implement peer responses', prompt)
        self.assertIn('Passing this step does not complete the module', prompt)
        self.assertIn('input/replication.md', self.task['input_paths'])

    def test_misspelled_duplicate_and_empty_steps_fail_closed(self):
        self.use_module()
        original = self.path.read_text()
        cases = [('No such step', original),
                 ('Repair reconciliation', original + '## Step: Repair reconciliation\nDuplicate.\n'),
                 ('Repair reconciliation', '# Module\n## Step: Repair reconciliation\n\n'),
                 ('Repair reconciliation', '## Step: Repair reconciliation\nNo module context.\n')]
        for title, text in cases:
            with self.subTest(title=title, text=text):
                self.path.write_text(text)
                with self.assertRaises(ValueError):
                    module_assignment(self.path, title)

    def test_invalid_module_configuration_cannot_fall_back_to_legacy_task(self):
        self.use_module()
        original = copy.deepcopy(self.config['tasks']['fixture-task'])
        for change in ({'module': '../outside.md'}, {'module': 'workflow.toml'},
                       {'module': 'input/missing.md'}, {'step': None},
                       {'module': None, 'step': 'Repair reconciliation'}):
            with self.subTest(change=change):
                self.config['tasks']['fixture-task'] = dict(original, **change)
                with self.assertRaises((ValueError, OSError)):
                    load_tasks(self.root, self.config)

    def test_planned_module_step_exposes_blocker_without_agent_calls(self):
        self.use_module()
        self.config['tasks']['fixture-task'].update(status='planned', blocked_reason='Peer state is missing.')
        task = load_tasks(self.root, self.config)[0]
        self.assertIn('Peer state is missing.', render_prompt(self.root, task))
        result = run_task(self.root, self.config, task['id'], execute_codex=True, fixture_mode=True)
        self.assertEqual(result['status'], 'BLOCKED')
        self.assertEqual(result['agent_calls'], 0)

    def test_fixture_plan_passes_only_selected_step(self):
        self.use_module()
        result = run_plan(self.root, self.config, execute_codex=True, fixture_mode=True)
        self.assertEqual(result['status'], 'PASSED', result)
        self.assertEqual(result['completion_scope'], 'SELECTED_STEPS_ONLY')
        self.assertEqual(result['module_completion'], 'NOT_ASSESSED')
        self.assertFalse(result['full_acceptance'])
        record = result['completed']['fixture-task']
        self.assertEqual(record['completion_scope'], 'STEP_ONLY')
        self.assertEqual(record['module_document'], 'input/replication.md')
        self.assertEqual(record['generation_kind'], 'SIMULATED')
        self.assertEqual(record['check_evidence_kind'], 'SIMULATED')

    def test_module_requirement_edit_still_requires_review(self):
        self.use_module()
        body = self.path.read_text()
        self.config['codex']['fixture_edits'] = [
            {'path': 'input/replication.md', 'text': 'Weakened module.'}]
        result = run_task(self.root, self.config, 'fixture-task', execute_codex=True, fixture_mode=True)
        self.assertEqual(result['status'], 'REVIEW_REQUIRED', result)
        self.assertEqual(self.path.read_text(), body)

    def test_text_plan_cli_keeps_json_interface_and_marks_scope(self):
        self.use_module()
        for output_format in ('text', 'json'):
            output = io.StringIO()
            with patch('orchestrator.cli.load_config', return_value=self.config), redirect_stdout(output):
                result = main(['--root', str(self.root), 'plan', '--format', output_format])
            self.assertEqual(result, 0)
            if output_format == 'text':
                self.assertIn('input/replication.md', output.getvalue())
                self.assertIn('[ready] fixture-task: Repair reconciliation', output.getvalue())
                self.assertIn('NOT_ASSESSED', output.getvalue())
            else:
                self.assertEqual(json.loads(output.getvalue())['module_completion'], 'NOT_ASSESSED')

    def test_cli_model_override_is_visible_bound_and_does_not_enable_execution(self):
        self.use_module()
        self.config['workflow']['allow_real_codex'] = False
        self.config['codex'].update(model='old-model', model_reasoning_effort='low')
        original = copy.deepcopy(self.config)
        before = plan(self.root, original)['input_bindings']
        output = io.StringIO()
        with patch('orchestrator.cli.load_config', return_value=self.config), redirect_stdout(output):
            result = main(['--root', str(self.root), '--model', 'gpt-6.1-sol',
                           '--reasoning-effort', 'high', 'plan'])
        self.assertEqual(result, 0)
        record = json.loads(output.getvalue())
        self.assertEqual(record['codex_request'], {'model': 'gpt-6.1-sol', 'model_reasoning_effort': 'high'})
        self.assertNotEqual(before['checker_policy_digest'], record['input_bindings']['checker_policy_digest'])
        self.assertFalse(self.config['workflow']['allow_real_codex'])
        with patch('orchestrator.scheduler.run_codex', side_effect=AssertionError('must remain a dry run')):
            preview = run_task(self.root, self.config, 'fixture-task')
        self.assertEqual(preview['codex_request'], record['codex_request'])
        self.assertEqual(preview['agent_calls'], 0)

    def test_repository_plan_maps_all_module_steps_and_keeps_global_work(self):
        config = load_config(REPO)
        tasks = load_tasks(REPO, config)
        task_order(tasks)
        declared = {(t['module_document'], t['step_title']) for t in tasks}
        self.assertEqual(len(declared), len(tasks))
        for document in {t['module_document'] for t in tasks}:
            headings = re.findall(r'^## Step: (.+)$', (REPO/document).read_text(), re.M)
            self.assertEqual({(document, heading) for heading in headings},
                             {pair for pair in declared if pair[0] == document})
        indexed = {t['id']: t for t in tasks}
        self.assertEqual(indexed['STATE-INIT']['status'], 'ready')
        self.assertEqual(set(indexed['STATE-INIT']['permitted_change_paths']), {
            'crates/raft-verified/src/node.rs', 'crates/raft-verified/src/node_state.rs',
            'crates/raft-verified/src/contracts.rs'})
        self.assertIn('node-proof', indexed['STATE-INIT']['check_profile_ids'])
        self.assertIn('REPLICATION-PROOF', indexed['INTEGRATION-MEASURE']['dependency_ids'])
        self.assertIn('PROGRESS-REFINEMENT', indexed['INTEGRATION-FULL']['dependency_ids'])
        self.assertIn('STATE-INIT', indexed['RUNTIME-PERSIST']['dependency_ids'])
        with patch('orchestrator.scheduler.run_codex', side_effect=AssertionError('preview must not generate')):
            preview = run_plan(REPO, config)
        self.assertEqual(preview['status'], 'DRY_RUN')
        self.assertEqual(preview['agent_calls'], 0)
        self.assertEqual(preview['codex_request'], {'model': 'gpt-6.1-sol', 'model_reasoning_effort': 'high'})
        for task in tasks:
            prompt = render_prompt(REPO, task)
            self.assertEqual(len(re.findall(r'^## Step: ', prompt, re.M)), 1)
            self.assertNotRegex(prompt, r'[\u3400-\u9fff]')
        self.assertFalse(plan(REPO, config)['full_acceptance'])

    def test_launcher_defaults_to_missing_node_work_with_no_generation(self):
        result = subprocess.run([str(REPO/'experiment.sh')], cwd='/tmp',
                                text=True, capture_output=True, check=True)
        preview = json.loads(result.stdout)
        self.assertEqual(preview['tasks'], ['STATE-INIT'])
        self.assertEqual(preview['agent_calls'], 0)

    def test_repair_prompt_preserves_prior_approaches_and_independent_results(self):
        self.use_module()
        diagnostics = {'attempts': [dict(number=1, summary='Tried a prefix-only invariant',
                        changed_paths=['src/lib.rs'], results='proof=FAIL, tests=PASS', artifacts='/tmp/attempt-1'),
                       dict(number=2, summary='Added duplicate scanning', changed_paths=['src/lib.rs'],
                            results='proof=FAIL, tests=PASS', artifacts='/tmp/attempt-2')]}
        prompt = render_prompt(self.root, self.task, diagnostics)
        for text in ('prefix-only invariant', 'duplicate scanning', 'proof=FAIL, tests=PASS', '/tmp/attempt-1'):
            self.assertIn(text, prompt)


if __name__ == '__main__':
    unittest.main()

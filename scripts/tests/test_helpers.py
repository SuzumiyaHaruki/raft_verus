"""Offline system regressions. Agent calls/check reports here are SIMULATED."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run as controller
import check as checker
from inputs import copy_files, input_files, save_json
from process import ProcessResult, run_process
from tools import ROOT


def simulated_report(output, revision, *, passed=(), missing_reason='missing_entry_or_dependency', final=False, mode='feedback'):
    output.mkdir(parents=True)
    log = output / 'SIMULATED.log'
    log.write_text('SIMULATED checker transport; no Rust or Verus was executed.\n')
    names = ('build', 'inventory', 'tests_seed_1', 'proof', *checker.OBLIGATIONS, 'linkage_review', 'safety')
    result = dict(status='MACHINE_CHECKS_PASSED' if final and mode == 'final' else 'INCOMPLETE', mode=mode, candidate_revision=revision,
        checks={n: dict(status='PASS' if final or n in passed or n in ('build','inventory','tests_seed_1','proof') else 'OPEN',
                        reason='checked' if final or n in passed or n in ('build','inventory','tests_seed_1','proof') else missing_reason,
                        log=str(log), command=['SIMULATED-check', n], diagnostic='SIMULATED missing dependency',
                        passed_tests=['SIMULATED test'] if n=='tests_seed_1' else []) for n in names})
    save_json(output / 'results.json', result)
    return result


class LoopTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / 'prototype'
        self.root.mkdir()
        copy_files(ROOT, self.root, input_files(ROOT) + ['scripts/run.py'] + [f'scripts/prompts/{n}.md' for n in controller.PROMPTS])
        self.workspace = self.root / 'runs/new-experiment'
        cfg = controller.settings(self.root)
        cfg['agent'].update(total_seconds='30', slice_seconds='1', check_seconds='1', max_rounds='3',
                            stall_rounds='2', max_redesigns='1', min_work_seconds='.1', handoff_seconds='.1', final_seconds='1')
        with (self.root / 'tools.ini').open('w') as f: cfg.write(f)
        self.home = Path(self.tmp.name) / 'personal'
        self.home.mkdir()
        (self.home / 'auth.json').write_text('{"SIMULATED":"not a credential"}')
        (self.home / 'AGENTS.md').write_text('SIMULATED personal instructions')
        (self.home / 'config.toml').write_text('model = "SIMULATED-personal-model"')
        (self.home / 'sessions').mkdir()
        (self.home / 'sessions/old-answer').write_text('SIMULATED old answer')
        (self.home / 'rules').mkdir()
        self.fake = Path(self.tmp.name) / 'simulated-agent'
        self.fake.write_text('#!' + sys.executable + '\n' + '''
import json, pathlib, sys, time
prompt=sys.stdin.read()
print(json.dumps({'type':'thread.started','thread_id':'SIMULATED-session'}), flush=True)
with pathlib.Path('SIMULATED-calls.jsonl').open('a') as f: f.write(json.dumps(sys.argv[1:])+'\\n')
pathlib.Path('madraft/SIMULATED-edit.txt').write_text('SIMULATED generation; no protocol implementation')
pathlib.Path('PROGRESS.md').write_text('SIMULATED approach: tried a toy decomposition; actual global dependencies remain open.')
if pathlib.Path('SIMULATED-sleep').exists(): time.sleep(5)
pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1]).write_text('SIMULATED claim: all proved')
''')
        self.fake.chmod(0o755)
        self.calls = []
        self.plan = []
        self.patches = [patch.object(controller, 'ROOT', self.root),
                        patch.object(controller, 'probe_tools', return_value={'status':'PASS','evidence':{'SIMULATED':'capability-only'}}),
                        patch.object(controller, 'evaluate', side_effect=self.evaluate),
                        patch.dict(os.environ, {'CODEX_HOME':str(self.home)})]
        for p in self.patches: p.start(); self.addCleanup(p.stop)

    def evaluate(self, workspace, *, output, revision, initial=False, **kwargs):
        self.calls.append((workspace, initial, revision))
        item = self.plan.pop(0) if self.plan else {}
        return simulated_report(output, revision, mode='initial' if initial else kwargs.get('mode','feedback'), **item)

    def invoke(self, action='new', *args):
        argv = ['run.py', action, str(self.workspace), '--codex', str(self.fake), *args]
        with patch.object(sys, 'argv', argv), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            try: return controller.main()
            except SystemExit as exc: return exc.code

    def state(self):
        return json.loads((controller.control_path(self.root, self.workspace) / 'state.json').read_text())

    def test_preview_has_no_side_effect_or_call(self):
        self.assertEqual(self.invoke(), 0)
        self.assertFalse(self.workspace.exists())
        self.assertFalse((self.root/'runs/.controller').exists())
        self.assertFalse(self.calls)

    def test_exit_zero_and_local_pass_continue_same_session(self):
        self.assertEqual(self.invoke('new', '--execute'), 1)
        state = self.state()
        self.assertEqual(state['attempts'], 3)
        self.assertEqual(state['status'], 'INCOMPLETE')
        argv = [json.loads(s) for s in (self.workspace/'SIMULATED-calls.jsonl').read_text().splitlines()]
        self.assertNotIn('resume', argv[0])
        self.assertIn('resume', argv[1])
        self.assertIn('SIMULATED-session', argv[1])
        self.assertIn('--ignore-user-config', argv[0])
        self.assertIn('--no-daemon', argv[0])
        prompt = (self.workspace/'runs/round-2/prompt.md').read_text()
        self.assertIn('missing_entry_or_dependency', prompt)
        self.assertIn('SIMULATED approach', prompt)
        self.assertIn('UNVERIFIED', prompt)
        self.assertIn('execution_link', prompt)
        self.assertEqual(state['history'][0]['exit'], 0)

    def test_fresh_runs_and_personal_home_do_not_import_prior_answers(self):
        (self.root/'runs/old/madraft').mkdir(parents=True)
        (self.root/'runs/old/madraft/SOLUTION.rs').write_text('SIMULATED prior answer')
        (self.root/'RESULT.md').write_text('SIMULATED operator historical answer')
        (self.root/'madraft/runs').mkdir()
        (self.root/'madraft/runs/SOLUTION.rs').write_text('SIMULATED hidden prior answer')
        original = (self.root/'madraft/src/raft/raft.rs').read_bytes()
        self.invoke('new', '--execute', '--max-rounds', '1')
        first = self.workspace
        self.workspace = first.with_name('another-fresh-experiment')
        self.invoke('new', '--execute', '--max-rounds', '1')
        self.assertEqual((self.root/'madraft/src/raft/raft.rs').read_bytes(), original)
        self.assertFalse((self.workspace/'madraft/runs').exists())
        self.assertFalse((self.workspace/'RESULT.md').exists())
        self.assertFalse((self.workspace/'scripts/run.py').exists())
        self.assertFalse((self.workspace/'README.md').exists())
        self.assertFalse((self.workspace/'runs/codex-home/sessions').exists())
        self.assertFalse((self.workspace/'runs/codex-home/AGENTS.md').exists())
        self.assertTrue((self.workspace/'runs/codex-home/auth.json').samefile(self.home/'auth.json'))
        self.assertFalse((first/'madraft/src/raft/raft.rs').samefile(self.workspace/'madraft/src/raft/raft.rs'))
        self.assertNotEqual(json.loads((controller.control_path(self.root, first)/'state.json').read_text())['run_id'], self.state()['run_id'])

    def test_resume_keeps_budget_history_and_rechecks_before_agent(self):
        self.invoke('new','--execute','--max-rounds','1')
        before = self.state()
        self.invoke('resume','--execute','--max-rounds','2')
        after = self.state()
        self.assertEqual(after['attempts'], 2)
        self.assertEqual(after['run_id'], before['run_id'])
        self.assertGreater(after['seconds_spent'], before['seconds_spent'])
        self.assertEqual(len(after['history']), 2)
        self.assertFalse(self.calls[2][1])
        self.assertIn('SIMULATED approach', (self.workspace/'runs/round-2/prompt.md').read_text())

    def test_legacy_state_is_not_a_resume_origin(self):
        (self.workspace/'runs').mkdir(parents=True)
        (self.workspace/'runs/session.json').write_text('{"thread_id":"old","attempts":1}')
        self.assertNotEqual(self.invoke('resume','--execute'), 0)
        self.assertFalse(self.calls)

    def test_exhausted_budget_cannot_be_reset_by_resume(self):
        self.invoke('new','--execute','--max-rounds','1')
        control = controller.control_path(self.root,self.workspace)
        state=self.state();state['seconds_spent']=state['total_seconds'];save_json(control/'state.json',state)
        count=len(self.calls)
        self.assertEqual(self.invoke('resume','--execute'),1)
        self.assertEqual(len(self.calls),count)
        self.assertEqual(self.state()['attempts'],1)

    def test_one_top_level_pass_does_not_finish(self):
        self.plan=[{}, {'passed':['election_safety']}]
        self.assertEqual(self.invoke('new','--execute','--max-rounds','1'),1)
        self.assertEqual(self.state()['status'],'INCOMPLETE')

    def test_five_safety_passes_do_not_replace_refinement(self):
        self.plan=[{}, {'passed':list(checker.SAFETY)}]
        self.assertEqual(self.invoke('new','--execute','--max-rounds','1'),1)
        self.assertIn('execution_link', (self.workspace/'runs/EXPERIMENT.md').read_text())

    def test_safety_and_execution_passes_do_not_replace_snapshot_coverage(self):
        self.plan = [{}, {'passed': [*checker.SAFETY, 'execution_link']}]
        self.assertEqual(self.invoke('new', '--execute', '--max-rounds', '1'), 1)
        self.assertIn('snapshot_observation | OPEN', (self.workspace / 'runs/EXPERIMENT.md').read_text())

    def test_simulated_full_acceptance_stops_only_after_independent_check(self):
        self.plan=[{}, {'final':True}, {'final':True}]
        self.assertEqual(self.invoke('new','--execute'),0)
        self.assertEqual(self.state()['attempts'],1)
        self.assertEqual(self.state()['status'],'MACHINE_CHECKS_PASSED')
        final = json.loads(Path(self.state()['last_check']).read_text())
        self.assertEqual(final['mode'], 'final')
        # This is solely a SIMULATED state-machine regression, not a Raft result.
        self.assertIn('SIMULATED', (self.workspace/'runs/round-1/agent.md').read_text())

    def test_previous_pass_is_invalidated_on_resume(self):
        self.plan=[{}, {'final':True}, {'final':True}]
        self.invoke('new','--execute','--max-rounds','1')
        (self.workspace/'madraft/SIMULATED-edit.txt').write_text('SIMULATED regression')
        self.plan=[{}]
        self.assertEqual(self.invoke('resume','--execute'),1)
        self.assertEqual(self.state()['status'],'INCOMPLETE')

    def test_stagnation_triggers_bounded_redesign(self):
        self.invoke('new','--execute','--max-rounds','8')
        state=self.state()
        self.assertEqual(state['redesigns'],1)
        self.assertLess(state['attempts'],8)
        self.assertIn('finite redesign',state['reason'])
        self.assertTrue(any(h['mode']=='redesign' for h in state['history']))
        self.assertIn('SIMULATED approach',(self.workspace/'runs/round-3/prompt.md').read_text())

    def test_changed_fixed_inputs_require_review(self):
        self.invoke('new','--execute','--max-rounds','1')
        (self.root/'requirements/safety.md').write_text('SIMULATED changed target')
        self.assertEqual(self.invoke('resume','--execute','--max-rounds','2'),2)
        self.assertEqual(self.state()['status'],'NEEDS_REVIEW')
        self.assertEqual(self.state()['attempts'],1)

    def test_missing_tool_stops_before_agent(self):
        with patch.object(controller,'probe_tools',return_value={'status':'BLOCKED','reason':'SIMULATED missing binary'}):
            self.assertEqual(self.invoke('new','--execute'),2)
        self.assertEqual(self.state()['attempts'],0)
        self.assertFalse((self.workspace/'SIMULATED-calls.jsonl').exists())

    def test_interrupted_reservation_is_not_refunded(self):
        self.invoke('new','--execute','--max-rounds','1')
        state=self.state();state['seconds_spent']=10;state['in_flight']={'kind':'agent','pid':None,'allowance':5}
        save_json(controller.control_path(self.root,self.workspace)/'state.json',state)
        self.invoke('resume','--execute')
        self.assertGreaterEqual(self.state()['seconds_spent'],10)
        self.assertIsNone(self.state()['in_flight'])

    def test_interrupted_history_is_saved_before_independent_check(self):
        original = self.evaluate
        def interrupted(workspace, **kwargs):
            if not kwargs.get('initial'):
                raise KeyboardInterrupt()
            return original(workspace, **kwargs)
        with patch.object(controller, 'evaluate', side_effect=interrupted):
            self.assertEqual(self.invoke('new', '--execute', '--max-rounds', '1'), 1)
        before = self.state()
        self.assertEqual(before['thread_id'], 'SIMULATED-session')
        self.assertEqual(len(before['history']), 1)
        self.assertIn('SIMULATED approach', before['history'][0]['progress'])
        self.assertEqual(before['history'][0]['check_status'], 'INCOMPLETE')
        self.invoke('resume', '--execute', '--max-rounds', '2')
        self.assertEqual(len(self.state()['history']), 2)

    def test_changed_real_diagnostics_are_not_stagnation_from_local_counts(self):
        first = {'checks': {'election_safety': {'status': 'OPEN', 'reason': 'verification_failure',
                                                'diagnostic': 'SIMULATED: initialization postcondition'}}}
        second = {'checks': {'election_safety': {'status': 'OPEN', 'reason': 'verification_failure',
                                                 'diagnostic': 'SIMULATED: transition postcondition'}}}
        self.assertNotEqual(controller.unresolved(first), controller.unresolved(second))
        second['checks']['election_safety']['diagnostic'] = first['checks']['election_safety']['diagnostic']
        second['checks']['proof'] = {'status': 'PASS', 'reason': 'checked', 'diagnostic': '999 verified SIMULATED'}
        self.assertEqual(controller.unresolved(first), controller.unresolved(second))

    def test_timeout_keeps_candidate_logs_and_same_session(self):
        self.invoke('new','--execute','--max-rounds','1')
        (self.workspace/'SIMULATED-sleep').touch()
        self.assertEqual(self.invoke('resume','--execute','--max-rounds','2'),1)
        state=self.state()
        self.assertEqual(state['last_process'],'TIMEOUT')
        self.assertEqual(state['history'][-1]['handoff'], 'hard_timeout')
        self.assertTrue((self.workspace/'madraft/SIMULATED-edit.txt').exists())
        self.assertIn('SIMULATED-session',(self.workspace/'runs/round-2/agent.jsonl').read_text())

    def test_insufficient_useful_round_budget_never_calls_model(self):
        self.assertEqual(self.invoke('new', '--execute', '--total-seconds', '1.1'), 1)
        self.assertEqual(self.state()['attempts'], 0)
        self.assertFalse((self.workspace/'SIMULATED-calls.jsonl').exists())
        self.assertIn('Insufficient', self.state()['reason'])

    def test_soft_handoff_and_hard_timeout_are_distinct(self):
        self.invoke('new', '--execute', '--max-rounds', '1')
        state = self.state()
        record = state['history'][0]
        self.assertLess(record['soft_seconds'], record['hard_seconds'])
        self.assertEqual(record['handoff'], 'normal')
        prompt = (self.workspace/'runs/round-1/prompt.md').read_text()
        self.assertIn('Soft handoff deadline:', prompt)
        self.assertIn('return normally', prompt)

    def test_test_progress_does_not_reset_individual_proof_stagnation(self):
        state = {'proof_gaps': {}}
        report = {'checks': {'execution_link': {'status':'OPEN', 'reason':'verification_failure',
                                              'diagnostic':'SIMULATED same init lemma'},
                             'election_safety': {'status':'OPEN', 'reason':'verification_failure',
                                                'diagnostic':'SIMULATED election'},
                             'tests_seed_1': {'status':'OPEN', 'reason':'test_failure', 'passed_tests':[]}}}
        controller.track_proofs(state, report)
        for n in range(3):
            report['checks']['tests_seed_1']['passed_tests'].append(str(n))
            report['checks']['election_safety']['diagnostic'] += ' changed'
            controller.track_proofs(state, report)
        self.assertEqual(state['proof_gaps']['execution_link']['repeats'], 3)
        self.assertEqual(state['proof_gaps']['election_safety']['repeats'], 0)

    def test_common_dependency_remains_stalled_when_other_errors_change(self):
        state = {'proof_gaps': {}}
        error = dict(file='madraft/src/core.rs', function='actual_step', line=10, column=1,
                     message='postcondition not satisfied', category='dependency_failure')
        report = {'checks': {'execution_link': {'status':'OPEN','reason':'verification_failure','errors':[error]}}}
        controller.track_proofs(state, report)
        error['line'] = 20  # Moving code is not mathematical progress.
        report['checks']['execution_link']['errors'].append(dict(error, function='another_lemma'))
        controller.track_proofs(state, report)
        self.assertEqual(state['proof_gaps']['dependency:madraft/src/core.rs::actual_step']['repeats'], 1)

    def test_resume_recheck_does_not_count_as_another_generation_round(self):
        state = {'proof_gaps': {}}
        report = {'checks': {'execution_link': {'status':'OPEN','reason':'verification_failure','diagnostic':'SIMULATED'}}}
        controller.track_proofs(state, report)
        controller.track_proofs(state, report)
        before = state['stale_rounds']
        controller.track_proofs(state, report, advance=False)
        self.assertEqual(state['stale_rounds'], before)

    def test_unrun_proof_is_a_scheduling_problem_not_redesign(self):
        state = {'proof_gaps': {}}
        report = {'checks': {'execution_link': {'status':'OPEN', 'reason':'check_budget_exhausted'}}}
        for _ in range(3): controller.track_proofs(state, report)
        self.assertEqual(state['stale_rounds'], 0)
        self.assertEqual(state['proof_gaps']['execution_link']['missed'], 3)

    def test_lock_prevents_two_writers(self):
        self.invoke('new','--execute','--max-rounds','1')
        with (controller.control_path(self.root,self.workspace)/'session.lock').open('w') as lock:
            controller.fcntl.flock(lock,controller.fcntl.LOCK_EX | controller.fcntl.LOCK_NB)
            with self.assertRaises(BlockingIOError): self.invoke('resume','--execute')
        self.assertEqual(self.state()['attempts'],1)


class ProcessTests(unittest.TestCase):
    def test_log_limit_and_nonzero_exit(self):
        result=run_process([sys.executable,'-c','print("x"*100000)'],ROOT,max_log_bytes=100)
        self.assertEqual(result.status,'LOG_LIMIT')
        self.assertEqual(len(result.stdout),100)
        self.assertEqual(run_process([sys.executable,'-c','raise SystemExit(7)'],ROOT).exit_code,7)

    def test_controller_sigkill_cleans_worker(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);pidfile=root/'pid';marker=root/'leaked'
            code='import pathlib,time; time.sleep(2); pathlib.Path('+repr(str(marker))+').touch()'
            parent='import sys,pathlib; sys.path.insert(0,'+repr(str(ROOT/'scripts'))+'); from process import run_process; run_process([sys.executable,"-c",'+repr(code)+'], '+repr(str(root))+', timeout=10, on_start=lambda p:pathlib.Path('+repr(str(pidfile))+').write_text(str(p)))'
            proc=subprocess.Popen([sys.executable,'-c',parent])
            try:
                deadline=time.monotonic()+5
                while not pidfile.exists() and time.monotonic()<deadline: time.sleep(.02)
                self.assertTrue(pidfile.exists())
                proc.kill();proc.wait()
                time.sleep(2.2)
                self.assertFalse(marker.exists())
            finally:
                if proc.poll() is None: proc.kill();proc.wait()


if __name__=='__main__': unittest.main()

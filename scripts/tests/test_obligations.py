"""SIMULATED processes/check outcomes for P0/P1/P2; no experimental agent."""
from pathlib import Path
import copy
import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check
import run as controller
from inputs import copy_files, input_files, save_json
from process import ProcessResult
from tools import ROOT


def note(**changes):
    plan = dict(id='counter-call', statement='Actual counter preserves the requested update.', goal='execution_link',
                kind='local', functions='madraft/src/counter.rs::update, madraft/src/outer.rs::handle',
                entry='madraft/proof/counter.rs', parent='madraft/proof/parent.rs', dependencies='',
                premises='Outer async path and history invariant remain open.', tests='')
    plan.update(changes)
    return '<!-- obligation -->\n```ini\n[obligation]\n' + ''.join(f'{k} = {v}\n' for k,v in plan.items()) + '```\n<!-- /obligation -->\n'


def fake_result(ok=True):
    return ProcessResult(['SIMULATED verifier'], 'COMPLETED', 0 if ok else 1, 'verification results:: 1 verified, 0 errors\n' if ok else '',
                         '' if ok else 'error: postcondition not satisfied\n', elapsed_seconds=.001)


class IncrementTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)/'prototype'; self.root.mkdir()
        copy_files(ROOT, self.root, input_files(ROOT)+['scripts/run.py']+[f'scripts/prompts/{n}.md' for n in controller.PROMPTS])
        self.workspace = self.root/'runs/fresh'
        cfg = controller.settings(self.root)['agent']
        self.state = dict(thread_id='SIMULATED-session', seconds_spent=17., attempts=0, model=cfg['model'],
                          effort=cfg['reasoning_effort'], codex='SIMULATED-never-called', seeds=[1])
        for k in ('total_seconds','slice_seconds','check_seconds','final_seconds','min_work_seconds','handoff_seconds'):
            self.state[k] = float(cfg[k])
        for k in ('max_rounds','stall_attempts','max_obligation_redesigns','max_redesigns','max_log_bytes'):
            self.state[k] = int(cfg[k])
        with patch.dict(os.environ, {'CODEX_HOME':self.tmp.name}):
            self.control = controller.prepare(self.root, self.workspace, self.state)
        self.state['attempts'] = 1
        self.state['history'] = [dict(changed_files=['src/counter.rs'], round=1)]
        self.loop = controller.Loop(self.root, self.workspace, self.control, self.state)
        (self.workspace/'madraft/src/counter.rs').write_text('pub fn update() {} // SIMULATED source')
        (self.workspace/'madraft/src/outer.rs').write_text('pub async fn handle() {} // SIMULATED outer path')
        (self.workspace/'madraft/proof/counter.rs').write_text('#[path = "../src/counter.rs"] mod counter;')
        (self.workspace/'PROGRESS.md').write_text(note())
        self.n = 0

    def checked(self, ok=True, output=None):
        self.n += 1
        output = output or self.control/f'unit-{self.n}'
        c = check.Checks(self.workspace, self.control/'inputs', self.root, output, 30, f'unit-{self.n}')
        with patch.object(check, 'run_process', return_value=fake_result(ok)):
            c.cargo('build'); c.obligation()
        r = c.finish('obligation', list(c.results))
        self.state['last_check'] = str(output/'results.json')
        return r

    def test_local_contract_survives_missing_outer_path_and_parent(self):
        table = '<!-- connections -->\n| transitions | madraft/src/counter.rs::update, madraft/src/outer.rs::handle | madraft/proof/counter.rs | caller open |\n<!-- /connections -->'
        p=self.workspace/'PROGRESS.md';p.write_text(note()+table)
        c=check.Checks(self.workspace,self.control/'inputs',self.root,self.control/'connections',30,'SIMULATED')
        with patch.object(check,'run_process',return_value=fake_result()) as proc:
            c.obligation();c.connections()
        self.assertEqual(proc.call_count,1)
        self.assertEqual(c.results['current_entry']['status'],'PASS')
        self.assertEqual(c.results['current_parent']['reason'],'missing_entry_or_dependency')
        self.assertEqual(c.results['connection_transitions_unit']['status'],'PASS')
        self.assertEqual(c.results['connection_transitions']['status'],'OPEN')
        self.assertEqual(c.results['connection_transitions']['open_paths'],['madraft/src/outer.rs::handle'])
        self.assertEqual(c.current['layer'],'LOCAL_UNIT_CHECKED')
        self.assertEqual(c.finish('final',list(c.results))['status'],'INCOMPLETE')

    def test_parent_nomination_is_checked_but_never_semantic_closure(self):
        (self.workspace/'madraft/proof/parent.rs').write_text('mod counter; // SIMULATED actual use still unreviewed')
        r=self.checked();c=r['current_obligation']
        self.assertTrue(c['parent_imports_entry'])
        self.assertEqual(c['layer'],'PARENT_UNIT_CHECKED_PREMISES_UNREVIEWED')
        self.assertEqual(c['first_blocker']['reason'],'outer_paths_open')
        self.assertEqual(r['implementation_acceptance'],'NOT_ESTABLISHED')

    def test_untrusted_test_commands_and_outside_proof_paths_are_rejected(self):
        for changes in [dict(tests='raft::tests::; touch /tmp/unsafe'),dict(entry='../old/proof.rs'),dict(rollback='../old/cp-1')]:
            (self.workspace/'PROGRESS.md').write_text(note(**changes))
            with patch.object(check,'run_process',side_effect=AssertionError('invalid plan must not execute')):
                c=check.Checks(self.workspace,self.control/'inputs',self.root,self.control/('invalid-'+str(self.n)),30,'SIMULATED')
                self.n+=1;c.obligation()
            self.assertEqual(c.current['plan']['error'],'invalid_current_obligation')

    def test_failures_are_per_obligation_and_renaming_does_not_reset(self):
        r=self.checked(False)
        for _ in range(3):controller.record_attempt(self.state,r)
        key=self.state['current_key']; record=self.state['obligations'][key]
        self.assertEqual(record['stalled'],3)
        self.assertEqual(controller.select_mode(self.state,r),'redesign')
        renamed=copy.deepcopy(r);renamed['current_obligation']['plan']['id']='renamed'
        for _ in range(3):controller.record_attempt(self.state,renamed)
        self.assertEqual(self.state['current_key'],key)
        self.assertEqual(controller.select_mode(self.state,renamed),'redesign')
        for _ in range(3):controller.record_attempt(self.state,r)
        self.assertEqual(controller.select_mode(self.state,r),'continue')
        self.assertTrue(record['blocked']);self.assertEqual(record['redesigns'],2)
        self.assertEqual(self.state['redesigns'],2)
        self.assertIn('counter-call',record['aliases']);self.assertIn('renamed',record['aliases'])

    def test_related_dependency_success_clears_stall_but_helpers_elsewhere_do_not(self):
        r=self.checked(False)
        controller.record_attempt(self.state,r);controller.record_attempt(self.state,r)
        record=self.state['obligations'][self.state['current_key']]
        r['checks']['unrelated']=dict(status='PASS',reason='checked',diagnostic='999 verified SIMULATED')
        controller.record_attempt(self.state,r);self.assertEqual(record['stalled'],3)
        r['current_obligation']['units']['current_dependency_1']='madraft/proof/premise.rs'
        r['checks']['current_dependency_1']=dict(status='PASS',reason='checked',command=['SIMULATED'])
        attempts=record['attempts']
        controller.record_attempt(self.state,r,advance=False)
        self.assertEqual(record['attempts'],attempts)  # Resume recheck is not an agent attempt.
        self.assertEqual(record['stalled'],0)
        self.assertEqual(record['newly_checked'],['madraft/proof/premise.rs'])
        self.assertEqual(r['checks']['current_entry']['status'],'OPEN')

    def test_renamed_or_rechecked_old_success_is_not_new_progress(self):
        first=self.checked();controller.record_attempt(self.state,first)
        record=self.state['obligations'][self.state['current_key']]
        failed=self.checked(False);controller.record_attempt(self.state,failed)
        controller.record_attempt(self.state,self.checked())
        self.assertEqual(record['newly_checked'],[])
        self.assertEqual(record['stalled'],1)
        (self.workspace/'madraft/proof/renamed.rs').write_bytes((self.workspace/'madraft/proof/counter.rs').read_bytes())
        (self.workspace/'PROGRESS.md').write_text(note(id='new-name',entry='madraft/proof/renamed.rs'))
        controller.record_attempt(self.state,self.checked())
        self.assertEqual(len(self.state['obligations']),1)
        self.assertEqual(record['newly_checked'],[])
        self.assertEqual(record['stalled'],1)

    def test_run_redesign_cap_blocks_another_obligation_without_resetting_costs(self):
        report=self.checked(False)
        self.state['redesigns']=6
        spent=self.state['seconds_spent']
        for _ in range(3):controller.record_attempt(self.state,report)
        self.assertEqual(controller.select_mode(self.state,report),'continue')
        record=self.state['obligations'][self.state['current_key']]
        self.assertTrue(record['blocked']);self.assertEqual(record['redesigns'],0)
        self.assertEqual(self.state['redesigns'],6);self.assertEqual(self.state['seconds_spent'],spent)

    def test_missing_timeout_and_compilation_do_not_consume_proof_redesign(self):
        for reason in ['missing_entry_or_dependency','verification_timeout','tool_unavailable','dependency_or_toolchain_unavailable','verifier_error','compilation_error']:
            r=self.checked(False);r['checks']['current_entry']['reason']=reason
            for _ in range(3):controller.record_attempt(self.state,r)
            self.assertEqual(controller.select_mode(self.state,r),'continue')
            self.assertEqual(self.state['obligations'][self.state['current_key']]['stalled'],0)
        self.assertEqual(self.state['redesigns'],0)

    def test_timeout_or_missing_plan_does_not_trigger_a_pending_redesign(self):
        report=self.checked(False)
        for _ in range(3):controller.record_attempt(self.state,report)
        key=self.state['current_key']
        report['checks']['current_entry']['reason']='verification_timeout'
        controller.record_attempt(self.state,report)
        self.assertEqual(controller.select_mode(self.state,report),'continue')
        self.assertEqual(self.state['obligations'][key]['stalled'],3)
        report['current_obligation']['plan']={'error':'missing_current_obligation'}
        controller.record_attempt(self.state,report)
        self.assertIsNone(self.state['current_key'])
        self.assertEqual(controller.select_mode(self.state,report),'continue')
        self.assertEqual(self.state['redesigns'],0)

    def test_after_two_local_handoffs_python_prioritizes_execution_connection(self):
        with patch.object(controller,'evaluate',side_effect=lambda *a,**kw: self.checked(output=kw['output'])):
            self.loop.inspect();self.loop.inspect()
        self.assertEqual(self.state['integration_wait'],2)
        report=json.loads(Path(self.state['last_check']).read_text())
        self.assertIn('ESTABLISH_EXECUTION_CONNECTION',controller.next_obligation(self.state,report))
        self.assertEqual(self.state['redesigns'],0)

    def test_current_test_selection_preserves_oldest_family_rotation(self):
        (self.workspace/'PROGRESS.md').write_text(note(tests='raft::tests::initial_election_2a'))
        with patch.object(check,'run_process',return_value=fake_result()):
            report=check.evaluate(self.workspace,authority=self.control/'inputs',tool_root=self.root,
                output=self.control/'rotation',mode='feedback',timeout=30,
                test_history={'raft::tests::':{'last_attempt':4}})
        self.assertEqual(report['functional_plan'][:2],['kvraft::tests::','raft::tests::initial_election_2a'])

    def test_rollback_restores_code_and_proof_keeps_unrelated_work_and_history(self):
        good=self.checked();self.loop.remember(good)
        cp=self.state['checkpoints'][0]
        original=(self.workspace/'madraft/src/counter.rs').read_bytes()
        (self.workspace/'madraft/src/counter.rs').write_text('SIMULATED broken code')
        (self.workspace/'madraft/proof/counter.rs').write_text('#[path = "../src/counter.rs"] mod counter; // broken proof')
        (self.workspace/'madraft/src/unrelated.rs').write_text('SIMULATED independent progress')
        (self.workspace/'PROGRESS.md').write_text(note(rollback=cp['id'],restore='src/counter.rs, proof/counter.rs'))
        bad=self.checked(False);self.loop.remember(bad)
        spent=self.state['seconds_spent'];run_id=self.state['run_id'];key=self.state['current_key']
        attempts=self.state['obligations'][key]['attempts']
        self.assertTrue(self.loop.rollback(bad))
        self.assertEqual((self.workspace/'madraft/src/counter.rs').read_bytes(),original)
        self.assertNotIn('broken',(self.workspace/'madraft/proof/counter.rs').read_text())
        self.assertTrue((self.workspace/'madraft/src/unrelated.rs').exists())
        self.assertEqual(self.state['seconds_spent'],spent);self.assertEqual(self.state['run_id'],run_id)
        self.assertEqual(self.state['obligations'][key]['attempts'],attempts)
        self.assertIsNone(self.state['last_check'])
        self.assertIn('PROOF_FAILURE',(self.control/'attempts.jsonl').read_text())
        self.assertTrue((self.control/'rollback-1/failed-madraft/src/counter.rs').is_file())
        with patch.object(controller,'evaluate',side_effect=lambda *a,**kw: self.checked(output=kw['output'])) as replay:
            restored=self.loop.inspect()
        self.assertEqual(replay.call_count,1);self.assertEqual(restored['checks']['current_entry']['status'],'PASS')

    def test_rollback_rejects_other_run_and_traversal_before_mutation(self):
        good=self.checked();self.loop.remember(good);cp=self.state['checkpoints'][0]
        before=(self.workspace/'madraft/src/counter.rs').read_bytes()
        for fields in [dict(rollback=cp['id'],restore='../elsewhere.rs'),dict(rollback='cp-999',restore='src/counter.rs')]:
            (self.workspace/'PROGRESS.md').write_text(note(**fields));bad=self.checked(False)
            with self.assertRaises(ValueError):self.loop.rollback(bad)
            self.assertEqual((self.workspace/'madraft/src/counter.rs').read_bytes(),before)
        cp['run_id']='another-experiment'
        (self.workspace/'PROGRESS.md').write_text(note(rollback=cp['id']))
        with self.assertRaises(ValueError):self.loop.rollback(self.checked(False))

    def test_microattempt_claim_is_replayed_and_cannot_forge_success(self):
        self.checked(output=self.workspace/'runs/checks/1')
        # Candidate claims PASS, but the independent replay really receives FAIL.
        with patch.object(check,'run_process',return_value=fake_result(False)) as proc:
            self.loop.replay_attempts();self.loop.replay_attempts()
        self.assertEqual(proc.call_count,1)
        self.assertEqual(self.state['replayed'],['1'])
        self.assertEqual(self.state['obligations'][self.state['current_key']]['last_outcome'],'PROOF_FAILURE')
        self.assertIn('attempt-replay',[c['kind'] for c in self.state['costs']])
        self.assertEqual(self.state['thread_id'],'SIMULATED-session')

    def test_two_hour_defaults_do_not_stop_at_eight_rounds(self):
        self.assertEqual(self.state['total_seconds'],7200);self.assertEqual(self.state['max_rounds'],64)
        state=dict(self.state,attempts=8,seconds_spent=4000)
        self.assertEqual(controller.round_plan(state),(600,'feedback'))
        state['seconds_spent']=6500
        self.assertEqual(controller.round_plan(state),(0,'final'))
        state['seconds_spent']=6250
        self.assertEqual(controller.round_plan(state),(350,'final'))


if __name__=='__main__':unittest.main()

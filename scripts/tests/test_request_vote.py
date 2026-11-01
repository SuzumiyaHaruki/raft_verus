"""Offline task/controller regressions. Mock check results below are SIMULATED."""
import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check
import run
import request_vote
from inputs import copy_files, differences, input_files, input_version, protected_files, save_json
from tools import ROOT

class RequestVoteTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/'source'; self.root.mkdir()
        copy_files(ROOT,self.root,input_files(ROOT,'request-vote')+['scripts/run.py']+
            ['scripts/prompts/'+p+'.md' for p in run.PROMPTS])
        self.workspace=self.root/'runs/new-receiver'
        cfg=run.settings(self.root)
        self.state=dict(task='request-vote',thread_id=None,seconds_spent=0.,attempts=0,
            model=cfg['agent']['model'],effort=cfg['agent']['reasoning_effort'],codex='NOT_CALLED',seeds=[1,2])
        for key in ('total_seconds','slice_seconds','check_seconds','final_seconds','min_work_seconds','handoff_seconds'):
            self.state[key]=float(cfg['agent'][key])
        for key in ('max_rounds','stall_attempts','max_redesigns','max_obligation_redesigns','max_log_bytes'):
            self.state[key]=int(cfg['agent'][key])
        with patch.dict(os.environ,{'CODEX_HOME':str(Path(self.tmp.name)/'empty-personal')}):
            self.control=run.prepare(self.root,self.workspace,self.state)
        self.authority=self.control/'inputs'

    def test_fresh_preparation_preserves_root_and_excludes_answers(self):
        self.assertFalse((self.root/'madraft/src/raft/vote_core.rs').exists())
        self.assertIn('todo!("handle RequestVote RPC")',(self.root/'madraft/src/raft/raft.rs').read_text())
        self.assertIn('unreached()',(self.workspace/'madraft/src/raft/vote_core.rs').read_text())
        self.assertIn('self.vote.receive(', (self.workspace/'madraft/src/raft/raft.rs').read_text())
        self.assertIn('task_scope', (self.workspace/'scripts/request_vote.py').read_text())
        self.assertFalse(any('fixtures' in n or 'bad_receiver' in n or n.startswith(('runs/','labs/')) for n in self.state['input_files']))
        self.assertEqual(run.origin_changes(self.root,self.control,self.state),[])
        self.assertFalse(differences(self.root/'requirements',self.workspace/'requirements'))
        self.assertEqual(self.state['total_seconds'],7200)
        self.assertEqual(self.state['model'],run.settings(ROOT)['agent']['model'])

    def test_protection_covers_task_and_actual_host_but_not_student_core(self):
        protected=protected_files(self.authority)
        for name in ('api.rs','caller.rs','host.rs','rpc_tests.rs','TASK.md'):
            self.assertIn('tasks/request_vote/'+name,protected)
        self.assertIn('madraft/src/raft/raft.rs',protected)
        self.assertNotIn('madraft/src/raft/vote_core.rs',protected)
        self.assertNotIn('madraft/proof/vote_lemmas.rs',protected)
        self.assertFalse(any(n.startswith('tasks/') for n in input_files(ROOT)))
        self.assertNotIn('madraft/src/raft/raft.rs',protected_files(ROOT))

    def test_raw_source_changes_are_detected_separately_from_preparation(self):
        p=self.root/'tasks/request_vote/api.rs'; p.write_text(p.read_text()+'\n// changed\n')
        self.assertIn('tasks/request_vote/api.rs',run.origin_changes(self.root,self.control,self.state))

    def invoke(self, argv):
        with patch.object(run,'ROOT',self.root),patch.object(sys,'argv',['run.py']+argv),contextlib.redirect_stdout(io.StringIO()),contextlib.redirect_stderr(io.StringIO()):
            try:return run.main()
            except SystemExit as e:return e.code

    def test_task_is_bound_across_resume_even_in_preview(self):
        self.assertEqual(self.state['input_version'],input_version('request-vote'))
        self.assertEqual(self.invoke(['resume',str(self.workspace),'--task','full']),2)
        self.assertEqual(self.invoke(['resume',str(self.workspace)]),0)
        self.assertEqual(self.invoke(['new',str(self.root/'runs/preview'),'--task','request-vote']),0)
        self.assertFalse((self.root/'runs/preview').exists())

    def simulated_check(self, *, mode='final', fail=None):
        output=self.root/('SIMULATED-check-'+str(len(list(self.root.glob('SIMULATED-check-*')))))
        def fake_run(checks,name,argv,**kwargs):
            return checks.record(name,'OPEN' if name==fail else 'PASS','SIMULATED',diagnostic='No real tool ran')
        def fake_cargo(checks,kind,seed=1,selection=''):
            return checks.record(kind+str(seed),'PASS','SIMULATED',diagnostic='No real tool ran')
        with patch.object(check.Checks,'run',fake_run),patch.object(check.Checks,'cargo',fake_cargo):
            return check.evaluate(self.workspace,authority=self.authority,tool_root=self.root,
                output=output,task='request-vote',mode=mode,seeds=[1,2],timeout=30)

    def test_simulated_local_pass_never_claims_global_completion(self):
        report=self.simulated_check()
        self.assertEqual(report['status'],request_vote.LABEL)
        self.assertEqual(set(report['global_safety'].values()),{'OPEN'})
        self.assertEqual(report['full_madraft'],'INCOMPLETE')
        self.assertNotEqual(run.success_label(self.state),run.success_label({'task':'full'}))
        self.assertEqual(self.simulated_check(fail='vote_pending')['status'],'INCOMPLETE')
        self.assertEqual(self.simulated_check(mode='initial')['status'],'NEEDS_REVIEW')

    def test_simulated_success_cannot_hide_modified_fixed_test(self):
        p=self.workspace/'tasks/request_vote/rpc_tests.rs';p.write_text('// deleted tests')
        report=self.simulated_check()
        self.assertEqual(report['status'],'NEEDS_REVIEW')
        self.assertIn('tasks/request_vote/rpc_tests.rs',report['changed_standards'])

    def test_feedback_is_narrow_chinese_and_local_history_has_distinct_goals(self):
        report=self.simulated_check(fail='vote_history')
        text=run.feedback(self.authority,self.state,report,'start',self.workspace,600)
        self.assertIn('累计剩余预算',text)
        self.assertNotIn('ESTABLISH_EXECUTION_CONNECTION',text)
        self.assertNotIn('connection_snapshots',text)
        a=report['current_obligation']['plan']
        b=dict(a,id='request-vote-confirmed',goal='request-vote-confirmed')
        self.assertNotEqual(run.obligation_key(self.state,a),run.obligation_key(self.state,b))
        self.assertFalse(run.ready_for_final(self.state,report))

if __name__=='__main__':unittest.main()

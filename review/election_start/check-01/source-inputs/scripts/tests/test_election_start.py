"""Offline controller tests. Mock tool outcomes are SIMULATED, never proof evidence."""
import contextlib
import io
import os
from pathlib import Path
import re
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check
import run
import election_start as task
import request_vote
from inputs import copy_files,differences,input_files,input_version,protected_files
from tools import ROOT


class ElectionStartTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/'source';self.root.mkdir()
        copy_files(ROOT,self.root,input_files(ROOT,'election-start')+['scripts/run.py']+
            ['scripts/prompts/'+p+'.md' for p in run.PROMPTS])
        cfg=run.settings(self.root)['agent']
        self.state=dict(task='election-start',thread_id=None,seconds_spent=0.,attempts=0,
            model=cfg['model'],effort=cfg['reasoning_effort'],codex='NOT_CALLED',seeds=[1,2,3,4,5])
        for k in ('total_seconds','slice_seconds','check_seconds','final_seconds','min_work_seconds','handoff_seconds'):
            self.state[k]=float(cfg[k])
        for k in ('max_rounds','stall_attempts','max_redesigns','max_obligation_redesigns','max_log_bytes'):
            self.state[k]=int(cfg[k])
        self.workspace=self.root/'runs/new-campaign'
        with patch.dict(os.environ,{'CODEX_HOME':str(Path(self.tmp.name)/'empty-personal')}):
            self.control=run.prepare(self.root,self.workspace,self.state)
        self.authority=self.control/'inputs'

    def test_fresh_unfinished_one_authority_and_input_isolation(self):
        self.assertIn('todo!("handle RequestVote RPC")',(self.root/'madraft/src/raft/raft.rs').read_text())
        actual=(self.workspace/'madraft/src/raft/raft.rs').read_text()
        self.assertNotIn('state: State',actual)
        self.assertNotIn('self.state.',actual);self.assertNotIn('raft.state.',actual)
        for text in ('vote.start_election()','vote.complete(','self.vote.receive(',
                     'self.send_vote_request(send).await','net.add_rpc_handler'):
            self.assertIn(text,actual)
        self.assertGreater((self.workspace/'madraft/src/raft/vote_core.rs').read_text().count('unreached()'),4)
        self.assertIn('assert(false)',(self.workspace/'madraft/proof/vote_lemmas.rs').read_text())
        self.assertFalse(any(n.startswith(('runs/','target/','labs/')) or any(x in n for x in ('bad_core','fixtures','maintain.py')) for n in self.state['input_files']))
        self.assertFalse(any(n.startswith('tasks/request_vote/') for n in self.state['input_files']))
        self.assertEqual(run.origin_changes(self.root,self.control,self.state),[])
        self.assertFalse(differences(self.root/'requirements',self.workspace/'requirements'))

    def test_fixed_path_protected_core_editable(self):
        names=protected_files(self.authority)
        for n in ('api.rs','caller.rs','host.rs','driver.rs','rpc_tests.rs','election_tests.rs'):
            self.assertIn('tasks/election_start/'+n,names)
        self.assertIn('madraft/src/raft/raft.rs',names)
        self.assertNotIn('madraft/src/raft/vote_core.rs',names)
        self.assertNotIn('madraft/proof/vote_lemmas.rs',names)

    def test_full_and_receiver_scopes_and_budget_preserved(self):
        self.assertEqual(self.state['total_seconds'],7200)
        cfg=run.settings(ROOT)['agent']
        self.assertEqual((self.state['model'],self.state['effort']),(cfg['model'],cfg['reasoning_effort']))
        self.assertEqual(input_version('full'),'madraft-joint-proof-5')
        self.assertEqual(input_version('request-vote'),'madraft-request-vote-1')
        self.assertEqual(request_vote.GOALS,('rules','history','confirmed','pending'))
        self.assertFalse(any(n.startswith('tasks/') for n in input_files(ROOT,'full')))

    def test_original_eight_receiver_tests_preserved(self):
        pattern=r'#\[(?:madsim::)?test\]\s*(?:async\s+)?fn\s+(\w+)'
        old=(ROOT/'tasks/request_vote/rpc_tests.rs').read_text()
        new=(ROOT/'tasks/election_start/rpc_tests.rs').read_text()
        self.assertEqual(re.findall(pattern,old),re.findall(pattern,new))
        self.assertEqual(len(re.findall(pattern,old)),8)
        # Original behavioral assertion messages survive interface adaptation.
        messages=re.findall(r'"([^"\n]+)"',old)
        for message in messages:self.assertIn('"'+message+'"',new)
        self.assertNotIn('#[ignore]',new)

    def simulated(self,mode='final',fail=None):
        output=self.root/('SIMULATED-'+str(len(list(self.root.glob('SIMULATED-*')))))
        self.commands=[];self.seeds=[]
        def fake_run(checks,name,argv,**kw):
            self.commands.append(argv)
            return checks.record(name,'OPEN' if name==fail else 'PASS','SIMULATED',diagnostic='No real tool ran')
        def fake_cargo(checks,kind,seed=1,selection=''):
            if kind=='test':self.seeds.append(seed)
            return checks.record(kind+str(seed),'PASS','SIMULATED',diagnostic='No real tool ran')
        with patch.object(check.Checks,'run',fake_run),patch.object(check.Checks,'cargo',fake_cargo):
            return check.evaluate(self.workspace,authority=self.authority,tool_root=self.root,
                output=output,task='election-start',mode=mode,seeds=[1,2,3,4,5],timeout=30)

    def test_simulated_final_requires_all_groups_and_seeds(self):
        report=self.simulated();self.assertEqual(report['status'],task.LABEL)
        self.assertEqual(self.seeds,[1,2,3,4,5]);self.assertEqual(len(self.commands),8)
        for cmd in self.commands:
            self.assertIn('--no-cheating',cmd);self.assertNotIn('--no-verify',cmd);self.assertNotIn('--verify-root',cmd)
        self.assertEqual(set(report['global_safety'].values()),{'OPEN'})
        self.assertEqual(report['full_madraft'],'INCOMPLETE')
        self.assertEqual(report['host_verification'],'UNVERIFIED')
        for group in ('campaign','authorization','history','receive_confirmed','receive_pending','self_confirmed','self_pending'):
            report=self.simulated(fail='election_'+group)
            self.assertEqual(report['status'],'INCOMPLETE');self.assertFalse(run.ready_for_final(self.state,report))
        self.assertEqual(self.simulated(mode='initial')['status'],'NEEDS_REVIEW')

    def test_safety_does_not_run_functional_tests(self):
        self.simulated(mode='safety');self.assertEqual(self.seeds,[]);self.assertEqual(len(self.commands),8)

    def test_tampered_observer_or_test_cannot_pass(self):
        for f in ('caller.rs','election_tests.rs'):
            p=self.workspace/'tasks/election_start'/f;old=p.read_text();p.write_text('// removed')
            report=self.simulated();self.assertEqual(report['status'],'NEEDS_REVIEW')
            self.assertIn('tasks/election_start/'+f,report['changed_standards']);p.write_text(old)

    def test_preview_and_resume_task_binding(self):
        def invoke(argv):
            with patch.object(run,'ROOT',self.root),patch.object(sys,'argv',['run.py']+argv),contextlib.redirect_stdout(io.StringIO()),contextlib.redirect_stderr(io.StringIO()):
                try:return run.main()
                except SystemExit as e:return e.code
        path=self.root/'runs/preview'
        self.assertEqual(invoke(['new',str(path),'--task','election-start']),0);self.assertFalse(path.exists())
        self.assertEqual(invoke(['resume',str(self.workspace),'--task','full']),2)
        self.assertEqual(invoke(['resume',str(self.workspace),'--task','request-vote']),2)
        self.assertEqual(invoke(['resume',str(self.workspace)]),0)

    def test_all_authored_agent_inputs_and_feedback_are_english(self):
        for scope in ('full','request-vote','election-start'):
            for n in input_files(ROOT,scope):
                self.assertIsNone(re.search(r'[\u3400-\u9fff]',(ROOT/n).read_text()),(scope,n))
        report=self.simulated(fail='election_history')
        for mode in ('start','continue','redesign'):
            text=run.feedback(self.authority,self.state,report,mode,self.workspace,600)
            self.assertIn('Cumulative budget remaining',text);self.assertIsNone(re.search(r'[\u3400-\u9fff]',text))
        a=report['current_obligation']['plan'];b=dict(a,goal='election-start-self_pending')
        self.assertNotEqual(run.obligation_key(self.state,a),run.obligation_key(self.state,b))


if __name__=='__main__':unittest.main()

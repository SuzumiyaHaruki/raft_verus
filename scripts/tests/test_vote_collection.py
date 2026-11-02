"""Offline dispatch regressions; mocked tool results are explicitly SIMULATED."""
from pathlib import Path
import contextlib
import io
import re
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check
import run
import vote_collection as task
from inputs import copy_files,input_files,protected_files,scoped_task
from tools import ROOT,settings,environment

class VoteCollectionTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)
        self.input=self.root/'input';self.workspace=self.root/'candidate'
        copy_files(ROOT,self.input,input_files(ROOT,'vote-collection'))
        task.prepare(self.input)
        copy_files(self.input,self.workspace,protected_files(self.input)+[
            'madraft/src/raft/vote_core.rs','madraft/proof/vote_lemmas.rs'])
        # The independent checker also needs the untouched student source inventory.
        from inputs import files_under
        copy_files(self.input/'madraft',self.workspace/'madraft',files_under(self.input/'madraft'))
        cfg, _=environment(ROOT); local=settings(self.workspace)
        for k in ('verus','rust_bin','rustup_home','cargo_home'):local['tools'][k]=cfg[k]
        with (self.workspace/'tools.ini').open('w') as f:local.write(f)
        self.calls=[];self.seeds=[]

    def simulated(self,fail=None,mode='final'):
        def fake_run(c,name,argv,**kw):
            self.calls.append(argv)
            return c.record(name,'OPEN' if name==fail else 'PASS','SIMULATED')
        def cargo(c,kind,seed=1,selection=''):
            if kind=='test':self.seeds.append(seed)
            return c.record(kind+str(seed),'PASS','SIMULATED')
        with patch.object(check.Checks,'run',fake_run),patch.object(check.Checks,'cargo',cargo):
            return check.evaluate(self.workspace,authority=self.input,tool_root=ROOT,
                output=self.root/('result-'+str(len(list(self.root.glob('result-*'))))),timeout=30,
                seeds=[1,2,3,4,5],task='vote-collection',mode=mode)

    def test_unfinished_same_core_and_no_old_answer(self):
        names=input_files(ROOT,'vote-collection')
        self.assertFalse(any(n.startswith(('runs/','review/','labs/','target/')) for n in names))
        self.assertFalse(any(x in n for n in names for x in ('fixture','bad_core','maintain.py','tasks/election_start/')))
        actual=(self.input/'madraft/src/raft/raft.rs').read_text()
        for text in ['vote.start_election()','vote.collect(context,result)','vote.complete(','self.vote.receive(',
                     'net.add_rpc_handler','self.deliver_vote_result(context,raw).await']:
            self.assertIn(text,actual)
        self.assertNotIn('state: State',actual)
        self.assertNotIn('vote_core::Core', (ROOT/'madraft/src/raft/raft.rs').read_text())
        self.assertGreater((self.input/'madraft/src/raft/vote_core.rs').read_text().count('unreached()'),7)
        self.assertIn('network_inv(t: Trace) -> bool { false }',(self.input/'madraft/proof/vote_lemmas.rs').read_text())
        for n in names:self.assertIsNone(re.search(r'[\u3400-\u9fff]',(ROOT/n).read_text()),n)
        self.assertEqual(settings(ROOT).getint('agent','total_seconds'),7200)

    def test_protected_inputs_and_editable_candidate(self):
        protected=protected_files(self.input)
        for n in ['network.rs','caller.rs','driver.rs','host.rs','collection_tests.rs','observer_tests.rs']:
            self.assertIn(task.DIRECTORY+'/'+n,protected)
        self.assertIn('madraft/src/raft/raft.rs',protected)
        self.assertNotIn('madraft/src/raft/vote_core.rs',protected)
        self.assertNotIn('madraft/proof/vote_lemmas.rs',protected)

    def test_old_behavior_assertions_and_observer_retained(self):
        for f in ['rpc_tests.rs','election_tests.rs','observer_tests.rs']:
            old=(ROOT/'tasks/election_start'/f).read_text();new=(ROOT/task.DIRECTORY/f).read_text()
            pattern=r'#\[(?:madsim::)?test\]\s*(?:async\s+)?fn\s+(\w+)'
            self.assertEqual(re.findall(pattern,old),re.findall(pattern,new))
            for message in re.findall(r'"([^"\n]+)"',old):self.assertIn('"'+message+'"',new)
        self.assertEqual((ROOT/'tasks/election_start/observer_tests.rs').read_bytes(),(ROOT/task.DIRECTORY/'observer_tests.rs').read_bytes())

    def test_final_needs_every_source_witness_recovery_and_seed(self):
        report=self.simulated();self.assertEqual(report['status'],task.LABEL)
        self.assertEqual(self.seeds,[1,2,3,4,5]);self.assertEqual(len(self.calls),len(task.GOALS))
        for command in self.calls:
            self.assertIn('--no-cheating',command);self.assertNotIn('--verify-root',command);self.assertNotIn('--no-verify',command)
        self.assertEqual(report['host_verification'],'UNVERIFIED');self.assertEqual(set(report['global_safety'].values()),{'OPEN'})
        state={'task':'vote-collection'}
        for goal in task.GOALS:
            failed=self.simulated(fail=task.PREFIX+goal)
            self.assertEqual(failed['status'],'INCOMPLETE');self.assertFalse(run.ready_for_final(state,failed))
        self.calls.clear();self.seeds.clear();self.simulated(mode='safety');self.assertEqual(self.seeds,[])

    def test_source_tamper_needs_review(self):
        p=self.workspace/task.DIRECTORY/'network.rs';p.write_text('// Removed source target')
        report=self.simulated();self.assertEqual(report['status'],'NEEDS_REVIEW')
        self.assertIn(task.DIRECTORY+'/network.rs',report['changed_standards'])

    def test_preview_and_scopes(self):
        path=ROOT/'runs/collection-PREVIEW-ONLY'
        self.assertFalse(path.exists())
        with patch.object(sys,'argv',['run.py','new',str(path),'--task','vote-collection']),contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(run.main(),0)
        self.assertIn('vote-collection',out.getvalue());self.assertFalse(path.exists())
        self.assertEqual(scoped_task('election-start').VERSION,'madraft-election-start-2')
        self.assertEqual(scoped_task('request-vote').VERSION,'madraft-request-vote-1')
        self.assertIsNone(scoped_task('full'))

if __name__=='__main__':unittest.main()

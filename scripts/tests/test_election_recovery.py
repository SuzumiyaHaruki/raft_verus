"""Offline dispatch/CLI regressions. Mocked check outcomes are SIMULATED."""
from pathlib import Path
import contextlib
import io
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check
import run
import election_recovery as task
from inputs import SCOPED_TASKS,copy_files,input_files,protected_files,success_label
from tools import ROOT,environment,settings

class RecoveryTests(unittest.TestCase):
    def test_inputs_unfinished_and_old_semantics_preserved(self):
        names=input_files(ROOT,'election-recovery')
        self.assertFalse(any(n.startswith(('runs/','review/','labs/','target/')) for n in names))
        self.assertFalse(any('fixture' in n or 'maintain.py' in n for n in names))
        for n in names:self.assertIsNone(re.search(r'[\u3400-\u9fff]',(ROOT/n).read_text()),n)
        old=ROOT/'tasks/election_safety';new=ROOT/task.DIRECTORY
        for name in ('api.rs','core.rs','lemmas.rs','network.rs','observation.rs','election_tests.rs','collection_tests.rs','observer_tests.rs','promotion_tests.rs','driver.rs'):
            self.assertEqual((old/name).read_bytes(),(new/name).read_bytes(),name)
        self.assertEqual((new/'rpc_tests.rs').read_text(),(old/'rpc_tests.rs').read_text()+'\ninclude!("recovery_tests.rs");\n')
        self.assertIn('recovery_inv(t:Trace) -> bool { false }',(new/'recovery_lemmas.rs').read_text())
        self.assertEqual(settings(ROOT).getint('agent','total_seconds'),7200)
        self.assertEqual(settings(ROOT)['agent']['model'],'gpt-6.1-sol')

    def test_preparation_and_fixed_source_boundary(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);copy_files(ROOT,root,input_files(ROOT,'election-recovery'));task.prepare(root)
            text=(root/'madraft/src/raft/raft.rs').read_text()
            for token in ('new_receiver_observed','vote_host::load','c.try_become_leader()','c.collect(context,result)','net.add_rpc_handler'):
                self.assertIn(token,text)
            protected=protected_files(root)
            for n in ('recovery_network.rs','recovery_tests.rs','caller.rs','host.rs'):
                self.assertIn(task.DIRECTORY+'/'+n,protected)
            self.assertNotIn('madraft/proof/recovery_lemmas.rs',protected)
            self.assertGreater((root/'madraft/src/raft/vote_core.rs').read_text().count('unreached()'),7)
            self.assertTrue((root/'madraft/proof/recovery_lemmas.rs').is_file())
        # Exercise the launcher's returned-file list, not just the task preparer.
        # An isolated empty CODEX_HOME prevents touching personal authentication.
        with tempfile.TemporaryDirectory() as d:
            source=Path(d)/'source';workspace=source/'runs/fresh'
            copy_files(ROOT,source,input_files(ROOT,'election-recovery')+['scripts/run.py']+
                       ['scripts/prompts/'+n+'.md' for n in run.PROMPTS])
            state={'task':'election-recovery'}
            with patch.dict(os.environ,{'CODEX_HOME':str(Path(d)/'empty-home')}):
                run.prepare(source,workspace,state)
            self.assertIn('madraft/proof/recovery_lemmas.rs',state['input_files'])
            self.assertEqual((workspace/'madraft/proof/recovery_lemmas.rs').read_bytes(),
                             (ROOT/task.DIRECTORY/'recovery_lemmas.rs').read_bytes())
            self.assertIn('unreached()', (workspace/'madraft/src/raft/vote_core.rs').read_text())

    def test_final_all_groups_seeds_and_failure(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);copy_files(ROOT,root,input_files(ROOT,'election-recovery'));task.prepare(root)
            cfg,_=environment(ROOT);local=settings(root)
            for k in ('verus','rust_bin','rustup_home','cargo_home'):local['tools'][k]=cfg[k]
            with (root/'tools.ini').open('w') as f:local.write(f)
            commands=[];seeds=[]
            def fake_run(c,name,argv,**kw):
                commands.append(argv);return c.record(name,'PASS','SIMULATED')
            def fake_cargo(c,kind,seed=1,selection=''):
                if kind=='test':seeds.append(seed)
                return c.record(kind+str(seed),'PASS','SIMULATED')
            with patch.object(check.Checks,'run',fake_run),patch.object(check.Checks,'cargo',fake_cargo):
                report=check.evaluate(root,authority=root,tool_root=ROOT,output=root/'result',timeout=30,task='election-recovery',mode='final')
            self.assertEqual(report['status'],task.LABEL, (report['changed_standards'],report['checks'].get('source_boundary')));self.assertEqual(seeds,[1,2,3,4,5])
            self.assertEqual(len(commands),21)
            for cmd in commands:
                self.assertIn('--no-cheating',cmd);self.assertNotIn('--verify-root',cmd);self.assertNotIn('--no-verify',cmd)
            self.assertEqual(set(report['global_safety'].values()),{'OPEN'});self.assertEqual(report['arbitrary_crash_recovery'],'OPEN')
            for goal in task.GOALS:
                failed=json.loads(json.dumps(report));failed['checks'][task.PREFIX+goal]['status']='OPEN'
                self.assertFalse(run.ready_for_final({'task':'election-recovery'},failed))

    def test_preview_no_agent_or_workspace(self):
        path=ROOT/'runs/election-recovery-PREVIEW-ONLY'
        with patch.object(sys,'argv',['run.py','new',str(path),'--task','election-recovery']),contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(run.main(),0)
        self.assertIn('election-recovery',out.getvalue());self.assertFalse(path.exists())

class ExitTests(unittest.TestCase):
    def test_registered_labels_and_controller_agree(self):
        for task_name in ('full',*SCOPED_TASKS):
            label=success_label(task_name)
            self.assertEqual(run.success_label({'task':task_name}),label)
            self.assertEqual(check.exit_status(label,task_name),0)
            self.assertEqual(check.exit_status('PASS',task_name),0)
            for status in ('INCOMPLETE','FAIL','MADE_UP_PASSED','TYPECHECKED'):
                self.assertEqual(check.exit_status(status,task_name),1)
            for status in ('BLOCKED','NEEDS_REVIEW'):self.assertEqual(check.exit_status(status,task_name),2)
            if task_name!='full':self.assertEqual(check.exit_status('MACHINE_CHECKS_PASSED',task_name),1)

    def test_real_cli_process_exit_codes_with_simulated_reports(self):
        # A real Python CLI process; ONLY its evaluator is mocked, not real proof evidence.
        program='''import sys
from unittest.mock import patch
sys.path.insert(0,sys.argv[1])
import check
status,task=sys.argv[2:4]
sys.argv=['check.py','final','--task',task]
with patch.object(check,'evaluate',return_value={'checks':{},'status':status}):
    raise SystemExit(check.main())
'''
        pairs=[(success_label(t),t,0) for t in ('full',*SCOPED_TASKS)]
        pairs += [(s,'election-recovery',code) for s,code in [('INCOMPLETE',1),('FAIL',1),('UNKNOWN_PASSED',1),('BLOCKED',2),('NEEDS_REVIEW',2),('ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED',1)]]
        for status,t,code in pairs:
            r=subprocess.run([sys.executable,'-c',program,str(ROOT/'scripts'),status,t],capture_output=True,text=True)
            self.assertEqual(r.returncode,code,(status,r.stdout,r.stderr))

if __name__=='__main__':unittest.main()

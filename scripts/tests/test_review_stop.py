"""SIMULATED controller regressions: no model, Rust or Verus is run here."""
import json
from pathlib import Path
import unittest
from unittest.mock import patch
import test_election_start as fixtures
import check
import run
from inputs import save_json
from process import ProcessResult

FAILED='raft::raft::vote_tests::restart_written_pending'

class ReviewStopTests(unittest.TestCase):
    setUp=fixtures.ElectionStartTests.setUp

    def simulated(self,output,revision,mode='feedback'):
        def cargo(c,kind,seed=1,selection=''):
            name='tests_seed_'+str(seed) if kind=='test' else kind
            log=c.output/(name+'.log');log.write_text('SIMULATED assertion failed: observer closed\n')
            item=c.record(name,'OPEN' if kind=='test' else 'PASS','test_failure' if kind=='test' else 'checked',
                category='functional_failure' if kind=='test' else 'pass',seed=seed,process='COMPLETED',
                failed_tests=[FAILED] if kind=='test' else [],diagnostic='assertion failed: observer closed')
            item['log']=str(log);return item
        def proof(c,name,argv,**kw):return c.record(name,'PASS','SIMULATED')
        with patch.object(check.Checks,'cargo',cargo),patch.object(check.Checks,'run',proof):
            return check.evaluate(self.workspace,authority=self.authority,tool_root=self.root,output=output,
                revision=revision,task='election-start',mode=mode,seeds=[1,2,3,4,5],timeout=30)

    def objection(self,report,**updates):
        value=dict(file='tasks/election_start/rpc_tests.rs',test=FAILED,conflict='Closed channel is not elapsed timeout',
            evidence=report['checks']['tests_seed_1']['log'],needs_fixed_change=True,recommendation='Review observer lifetime; unapplied',
            candidate_revision=report['candidate_revision'],unresolved='Fixed observation needs maintenance review')
        value.update(updates)
        run.progress_path(self.workspace,self.state).write_text('<!-- fixed-input-objection -->\n'+json.dumps(value)+'\n<!-- /fixed-input-objection -->\n')

    def test_actual_functional_blocker_and_no_final_in_both_scopes(self):
        report=self.simulated(self.root/'report','r')
        first=report['current_obligation']['first_blocker']
        self.assertEqual(first['failed_tests'],[FAILED]);self.assertEqual(first['seed'],1)
        self.assertNotEqual(first['reason'],'run_all_local_tests_and_seeds')
        self.assertEqual(set(report['local_goals'].values()),{'PASS'})
        for scope in ('full','request-vote','election-start'):
            module=run.scoped_task(scope)
            names=[module.PREFIX+g for g in module.GOALS] if module else [*run.OBLIGATIONS,'safety','linkage_review',*('connection_'+a for a in run.CONNECTIONS)]
            other=dict(report,checks={**{n:dict(status='PASS') for n in names},'tests_seed_1':report['checks']['tests_seed_1']})
            self.assertFalse(run.ready_for_final(dict(task=scope),other))
        # Exercise the actual shared evaluate function with the receiver descriptor.
        from inputs import copy_files,input_files
        import request_vote
        a=self.root/'receiver-authority';w=self.root/'receiver-workspace'
        copy_files(run.ROOT,a,input_files(run.ROOT,'request-vote'));request_vote.prepare(a)
        copy_files(a,w,run.files_under(a))
        # Same parser and blocker selection, four receiver proofs all green.
        def cargo(c,kind,seed=1,selection=''):
            return c.record(kind,'OPEN' if kind=='test' else 'PASS','SIMULATED',failed_tests=[FAILED],seed=seed)
        with patch.object(check.Checks,'cargo',cargo),patch.object(check.Checks,'run',lambda c,n,a,**k:c.record(n,'PASS','SIMULATED')):
            r=check.evaluate(w,authority=a,tool_root=run.ROOT,output=self.root/'receiver-report',task='request-vote',timeout=30)
        self.assertEqual(r['current_obligation']['first_blocker']['check'],'test')

    def test_identity_ignores_reports_and_unreferenced_repro_but_tracks_real_dependencies(self):
        identity=lambda:run.relevant_identity(self.workspace,self.authority,self.state,{})
        old=identity()
        for name in ('madraft/PROGRESS.md','madraft/RESULT.md','madraft/repro/unreferenced.rs'):
            p=self.workspace/name;p.parent.mkdir(exist_ok=True);p.write_text('diagnostic only')
        self.assertEqual(old,identity())
        for name in ('madraft/src/raft/vote_core.rs','madraft/proof/vote_lemmas.rs','tasks/election_start/rpc_tests.rs','scripts/check.py','madraft/Cargo.toml'):
            p=self.workspace/name;b=p.read_bytes();p.write_bytes(b+b'\n// changed\n');self.assertNotEqual(old,identity(),name);p.write_bytes(b)
        self.state['seeds']=[6];self.assertNotEqual(old,identity())

    def test_failure_signature_normalizes_paths_not_assertions_or_seeds(self):
        r=self.simulated(self.root/'report','r')
        a=r['checks']['tests_seed_1'];a['diagnostic']="thread 'test' panicked at /tmp/old/check-1-77/file.rs:3:4:\nassertion failed: no old reply"
        first=run.failure_signature(r)
        a['diagnostic']=a['diagnostic'].replace('/tmp/old/check-1-77','/elsewhere/check-9-88').replace(':3:4',':9:2')
        self.assertEqual(first,run.failure_signature(r))
        a['seed']=5;self.assertNotEqual(first,run.failure_signature(r))
        a['seed']=1;a['diagnostic']=a['diagnostic'].replace('no old reply','wrong restored vote');self.assertNotEqual(first,run.failure_signature(r))

    def test_shared_dependency_scan_tracks_data_and_conservatively_handles_unknown_modules(self):
        root=self.workspace/'madraft'
        (root/'src/helper.rs').write_text('const DATA: &[u8] = include_bytes!("data.bin");')
        (root/'src/data.bin').write_bytes(b'\xff\xfe')
        entry=root/'src/lib.rs';entry.write_text('mod helper;')
        identity=lambda:run.relevant_identity(self.workspace,self.authority,self.state,{})
        old=identity();(root/'src/data.bin').write_bytes(b'changed')
        self.assertNotEqual(old,identity())
        for uncertain in ('mod unresolved;', 'include!(concat!("generated", ".rs"));'):
            entry.write_text(uncertain)
            before=identity();(root/'unresolved-input.txt').write_text(uncertain)
            self.assertNotEqual(before,identity())

    def test_scoped_proof_dispatch_is_fixed_and_rejects_unknown_targets(self):
        c=check.Checks(self.workspace,self.authority,self.root,self.root/'proof-dispatch',30,'SIMULATED',task='election-start')
        with patch.object(c,'run',return_value={'reason':'verification_failure'}) as execute:
            c.verus('scoped',obligation='campaign')
            name,argv=execute.call_args.args
            self.assertEqual(name,'election_campaign')
            self.assertEqual(Path(argv[1]),c.snapshot/'tasks/election_start/caller.rs')
            self.assertIn('madsim',argv);self.assertIn('es_campaign',argv)
            self.assertIn('--no-cheating',argv)
            self.assertNotIn('--verify-root',argv);self.assertNotIn('--no-verify',argv)
            with self.assertRaises(ValueError):c.verus('scoped',obligation='arbitrary.rs')
            self.assertEqual(execute.call_count,1)

    def drive(self,edit,*,final=False):
        reports=[];calls=[];diagnostics=[]
        def evaluate(workspace,**kw):
            report=self.simulated(kw['output'],kw['revision'],'final' if final else kw['mode']);reports.append(report);return report
        def agent(argv,cwd,**kw):
            calls.append(argv);edit(reports[-1],len(calls))
            kw['output_paths'][0].write_text(json.dumps(dict(type='thread.started',thread_id='SIMULATED'))+'\n')
            return ProcessResult(argv,'COMPLETED',0,'','',elapsed_seconds=.01)
        def cargo(c,kind,seed=1,selection=''):
            diagnostics.append((kind,selection,c.deadline-c.started))
            return c.record(kind,'OPEN' if kind=='test' else 'PASS','test_failure' if kind=='test' else 'checked',
                failed_tests=[FAILED] if kind=='test' else [],process='COMPLETED',seed=seed)
        self.state['max_rounds']=10
        loop=run.Loop(self.root,self.workspace,self.control,self.state)
        with patch.object(run,'probe_tools',return_value=dict(status='PASS',evidence={'SIMULATED':'tools'})),patch.object(run,'evaluate',evaluate),patch.object(run,'run_process',agent),patch.object(check.Checks,'cargo',cargo):
            result=loop.run()
        return result,reports,calls,diagnostics

    def test_objection_one_bounded_diagnostic_then_review_no_shell(self):
        def edit(r,n):
            p=self.workspace/'madraft/repro/closed.rs';p.parent.mkdir(exist_ok=True)
            p.write_text('// SIMULATED reproducer; no verifier evidence\nfn main() {}\n')
            self.objection(r,evidence='madraft/repro/closed.rs',reproduce='DO NOT EXECUTE; arbitrary shell')
        result,reports,calls,diagnostics=self.drive(edit)
        self.assertEqual(result,2);self.assertEqual(self.state['reason'],'suspected_fixed_input_defect')
        self.assertEqual(len(calls),1);self.assertEqual(len(reports),1)
        self.assertEqual([x[:2] for x in diagnostics],[('build',''),('test',FAILED)])
        self.assertTrue(all(0<x[2]<=60 for x in diagnostics))
        self.assertEqual(self.state['fixed_input_diagnostic']['limit_seconds'],60)
        self.assertGreater(self.state['seconds_spent'],0)
        loop=run.Loop(self.root,self.workspace,self.control,self.state)
        with patch.object(run,'run_process',side_effect=AssertionError('must not execute again')):
            self.assertEqual(loop.review_objection(reports[-1]),2)
        self.assertEqual(self.state['reason'],'fixed_input_diagnostic_already_used')

    def test_diagnostic_timeout_is_inconclusive_and_budget_is_not_renewed(self):
        report=self.simulated(self.root/'report','r');self.objection(report)
        self.state['seconds_spent']=self.state['total_seconds']-2
        loop=run.Loop(self.root,self.workspace,self.control,self.state)
        limits=[]
        def timeout(c,kind,seed=1,selection=''):
            limits.append(c.deadline-c.started)
            return c.record(kind,'OPEN','check_budget_exhausted',process='TIMEOUT',seed=seed)
        with patch.object(check.Checks,'cargo',timeout):
            self.assertEqual(loop.review_objection(report),2)
        self.assertEqual(self.state['reason'],'fixed_input_diagnostic_inconclusive')
        self.assertTrue(all(0<x<=2 for x in limits))
        self.assertEqual(self.state['total_seconds'],7200)
        self.assertGreater(self.state['seconds_spent'],7198)

    def test_review_stop_survives_tool_failure_and_removal_of_the_objection_note(self):
        self.drive(lambda r,n:self.objection(r))
        with patch.object(run,'probe_tools',return_value=dict(status='BLOCKED',reason='SIMULATED missing tool')):
            self.assertEqual(run.Loop(self.root,self.workspace,self.control,self.state).run(),2)
        self.assertEqual(self.state['status'],'BLOCKED')
        run.progress_path(self.workspace,self.state).write_text('Report changed; checked candidate unchanged.')
        with patch.object(run,'probe_tools',return_value=dict(status='PASS',evidence={'SIMULATED':'tools'})),patch.object(run,'run_process',side_effect=AssertionError('must not call an agent')):
            self.assertEqual(run.Loop(self.root,self.workspace,self.control,self.state).run(),2)
        self.assertEqual(self.state['reason'],'unchanged_candidate_waiting_for_review')

    def test_unsubstantiated_claim_gets_one_gap_then_stops(self):
        result,reports,calls,diagnostics=self.drive(lambda r,n:self.objection(r,evidence=''))
        self.assertEqual(result,2);self.assertIn('no_progress',self.state['reason'])
        self.assertEqual(len(calls),2);self.assertEqual(len(reports),1);self.assertEqual(diagnostics,[])
        self.assertIn('evidence gaps', (self.workspace/'runs/round-2/prompt.md').read_text())

    def test_reports_and_unused_reproducer_cannot_repeat_failed_final(self):
        def edit(r,n):
            run.progress_path(self.workspace,self.state).write_text('report updated '+str(n))
            p=self.workspace/'madraft/repro/unused.rs';p.parent.mkdir(exist_ok=True);p.write_text('unused '+str(n))
        result,reports,calls,diagnostics=self.drive(edit,final=True)
        self.assertEqual(result,2);self.assertEqual(len(reports),1);self.assertEqual(len(calls),2)
        self.assertTrue(all(h['no_source_change'] for h in self.state['history']))
        self.assertEqual(len(self.state['evidence_reuse']),2);self.assertEqual(diagnostics,[])

    def test_core_change_requires_recheck_and_protected_change_stops(self):
        def edit(r,n):
            if n==1:
                p=self.workspace/'madraft/src/raft/vote_core.rs';p.write_text(p.read_text()+'\n// SIMULATED changed checked input\n')
            else:self.objection(r)
        result,reports,calls,diag=self.drive(edit)
        self.assertEqual(result,2);self.assertEqual(len(reports),2)
        self.assertTrue(self.state['history'][0]['changed_checked_inputs'])
        p=self.workspace/'tasks/election_start/rpc_tests.rs';p.write_text('// unauthorized change')
        c=check.Checks(self.workspace,self.authority,self.root,self.root/'protection',30,'tamper',task='election-start')
        c.record('SIMULATED','PASS','SIMULATED');self.assertEqual(c.finish('final',['SIMULATED'])['status'],'NEEDS_REVIEW')

if __name__=='__main__':unittest.main()

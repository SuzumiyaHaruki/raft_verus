"""Offline checker regressions; all subprocess output fixtures are SIMULATED."""
from pathlib import Path
import contextlib
import io
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import check
from inputs import copy_files, input_files
from process import ProcessResult
from tools import ROOT, environment, settings


def simulated_output(text='', code=0, status='COMPLETED'):
    return ProcessResult(['SIMULATED tool'],status,code,text,'')


class CheckerTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.directory=Path(self.tmp.name)
        self.workspace=self.directory/'workspace'
        copy_files(ROOT,self.workspace,input_files(ROOT))
        cfg=settings(ROOT);resolved,_=environment(ROOT)
        for k in ('verus','rust_bin','rustup_home','cargo_home'):cfg['tools'][k]=resolved[k]
        with (self.workspace/'tools.ini').open('w') as file:cfg.write(file)
        self.serial=0

    def checker(self):
        self.serial+=1
        return check.Checks(self.workspace,ROOT,ROOT,self.directory/f'check-{self.serial}',30,'SIMULATED-revision')

    def test_expected_inventory_has_all_families_and_host_tests(self):
        names=check.required_tests(ROOT)
        self.assertEqual(len(names),60)
        self.assertIn('test_support::restart_preserves_node_and_simulated_file',names)

    def test_empty_filter_and_missing_family_are_not_passes(self):
        c=self.checker()
        with patch.object(check,'run_process',return_value=simulated_output('running 0 tests\ntest result: ok. 0 passed; 0 failed;')):
            self.assertEqual(c.cargo('test',1,'nonexistent')['reason'],'empty_test_selection')
        with patch.object(check,'run_process',return_value=simulated_output('raft::tests::initial_election_2a: test\n')):
            self.assertEqual(c.cargo('inventory')['reason'],'missing_test_coverage')

    def test_one_passing_test_does_not_hide_missing_coverage(self):
        c=self.checker()
        with patch.object(check,'run_process',return_value=simulated_output('test raft::tests::initial_election_2a ... ok\n')):
            result=c.cargo('test')
        self.assertEqual(result['status'],'OPEN')
        self.assertIn('kvraft::tests::basic_3a',result['uncovered_tests'])

    def test_newly_ignored_original_test_is_missing_coverage(self):
        c=self.checker()
        text='\n'.join(f'test {n} ... '+('ignored' if n in check.IGNORED or n=='raft::tests::initial_election_2a' else 'ok') for n in c.expected)
        with patch.object(check,'run_process',return_value=simulated_output(text)):
            result=c.cargo('test')
        self.assertEqual(result['uncovered_tests'],['raft::tests::initial_election_2a'])

    def test_original_ignored_placeholder_is_not_counted_as_pass(self):
        c=self.checker()
        text='\n'.join(f'test {n} ... '+('ignored' if n in check.IGNORED else 'ok') for n in c.expected)
        with patch.object(check,'run_process',return_value=simulated_output(text)):
            result=c.cargo('test')
        self.assertEqual(result['status'],'PASS')
        self.assertEqual(len(result['passed_tests']),59)
        self.assertEqual(result['ignored_tests'],list(check.IGNORED))

    def test_protected_changes_use_original_tests_and_targets(self):
        for name in ('requirements/targets.rs','scripts/check.py','madraft/src/raft/tests.rs'):
            (self.workspace/name).write_text('SIMULATED weakened check')
        c=self.checker()
        self.assertEqual((c.snapshot/'requirements/targets.rs').read_bytes(),(ROOT/'requirements/targets.rs').read_bytes())
        self.assertEqual((c.snapshot/'madraft/src/raft/tests.rs').read_bytes(),(ROOT/'madraft/src/raft/tests.rs').read_bytes())
        self.assertIn('scripts/check.py',c.changed_standards)
        self.assertEqual(c.finish('final',[])['status'],'NEEDS_REVIEW')

    def test_default_final_seeds_are_all_checked_on_one_snapshot(self):
        observed=[]
        def cargo(self,kind,seed=1,selection=''):
            if kind=='test':observed.append((seed,self.snapshot))
            return self.record(kind if kind!='test' else f'tests_seed_{seed}','PASS','SIMULATED')
        def verus(self,kind,entry=None,obligation=None):return self.record(obligation or kind,'OPEN','SIMULATED missing proof')
        with patch.object(check.Checks,'cargo',cargo),patch.object(check.Checks,'verus',verus):
            report=check.evaluate(self.workspace,output=self.directory/'suite',timeout=20)
        self.assertEqual([s for s,p in observed],[1,2,3,4,5])
        self.assertEqual(len({p for s,p in observed}),1)
        self.assertEqual(report['status'],'INCOMPLETE')

    def test_edit_during_checks_invalidates_results(self):
        c=self.checker();c.record('SIMULATED','PASS','SIMULATED')
        (self.workspace/'madraft/src/raft/state.rs').write_text('SIMULATED concurrent edit')
        self.assertEqual(c.finish('final',['SIMULATED'])['status'],'INCOMPLETE')

    def test_mutation_of_checked_snapshot_is_also_detected(self):
        c=self.checker();c.record('SIMULATED','PASS','SIMULATED')
        (c.snapshot/'madraft/src/raft/state.rs').write_text('SIMULATED build script edit')
        self.assertEqual(c.finish('final',['SIMULATED'])['status'],'INCOMPLETE')

    def test_standards_changed_during_checks_are_not_accepted(self):
        for name, checked in [('scripts/check.py', False), ('requirements/targets.rs', True)]:
            with self.subTest(name=name):
                c = self.checker()
                c.record('SIMULATED', 'PASS', 'SIMULATED')
                path = (c.snapshot if checked else self.workspace) / name
                original = path.read_bytes()
                path.write_text('SIMULATED modification during checks')
                self.assertEqual(c.finish('final', ['SIMULATED'])['status'], 'NEEDS_REVIEW')
                path.write_bytes(original)

    def test_top_caller_uses_the_copied_fixed_requirements(self):
        c = self.checker()
        with patch.object(check, 'run_process', return_value=simulated_output('verification results:: 1 verified, 0 errors')):
            c.verus('top', obligation='election_safety')
        caller = (c.output / 'election_safety-caller.rs').read_text()
        self.assertIn(str(c.snapshot / 'requirements/targets.rs'), caller)
        self.assertNotIn(str(ROOT / 'requirements/targets.rs'), caller)

    def test_safety_cli_includes_full_tests_and_proof_acceptance(self):
        with patch.object(sys, 'argv', ['check.py', 'safety', '--json']), \
             patch.object(check, 'evaluate', return_value={'status': 'INCOMPLETE'}) as evaluate, \
             contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(check.main(), 1)
        evaluate.assert_called_once()

    def test_single_caller_keeps_fixed_target_and_no_function_filter(self):
        path=self.workspace/'madraft/proof/implementation.rs'
        text=check.caller_text(ROOT,path,'election_safety')
        self.assertIn(str(ROOT/'requirements/targets.rs'),text)
        self.assertIn('prove_election_safety',text)
        self.assertNotIn('prove_log_matching',text)
        full=check.caller_text(ROOT,path)
        for goal in check.OBLIGATIONS:self.assertIn('// obligation: '+goal,full)
        c=self.checker()
        with patch.object(check,'run_process',return_value=simulated_output('verification results:: 1 verified, 0 errors')) as run:
            c.verus('top',obligation='election_safety')
        argv=run.call_args.args[0]
        self.assertIn('--no-cheating',argv)
        self.assertFalse(any('verify-function' in a or 'verify-only' in a for a in argv))

    def test_typechecking_is_not_verification(self):
        self.assertEqual(check.classify(simulated_output(''),proof=True),('OPEN','no_verified_obligation'))

    def test_diagnostic_categories_are_distinct(self):
        cases=[(simulated_output("error: couldn't read x: No such file",1),'missing_entry_or_dependency'),
               (simulated_output('error[E0308]: mismatched types',1),'compilation_error'),
               (simulated_output('postcondition not satisfied',1),'verification_failure'),
               (simulated_output('',None,'TIMEOUT'),'verification_timeout'),
               (simulated_output('',None,'BLOCKED'),'tool_unavailable')]
        for result,reason in cases:self.assertEqual(check.classify(result,proof=True)[1],reason)

    def test_known_vacuity_forms_are_flagged_not_declared_semantically_complete(self):
        self.assertTrue(check.vacuity_findings('pub open spec fn execution(x:int)->bool { false }'))
        self.assertTrue(check.vacuity_findings('pub open spec fn refines_executable(x:int)->bool { true }'))
        self.assertTrue(check.vacuity_findings('pub open spec fn observe(x:int)->History { History { events: Seq::empty() } }'))
        self.assertFalse(check.vacuity_findings('pub open spec fn observe(x:int)->History { actual(x) } fn initial(){ History {events: Seq::empty()} }'))
        self.assertIn('do not decide',check.SEMANTIC_BOUNDARY)

    def test_source_links_reject_an_external_answer_path(self):
        p=self.workspace/'madraft/proof/implementation.rs'
        outside=self.directory/'not-this-experiment.rs';outside.write_text('// SIMULATED unrelated source')
        p.write_text('#[path = "'+str(outside)+'"] mod answer;')
        self.assertTrue(check.source_links(p,self.workspace)[1])

    def test_copied_tools_configuration_is_accepted_without_trusting_overrides(self):
        self.assertFalse(self.checker().changed_standards)
        cfg=settings(self.workspace);cfg['tools']['verus']='/SIMULATED/fake-verifier'
        with (self.workspace/'tools.ini').open('w') as f:cfg.write(f)
        self.assertIn('tools.ini',self.checker().changed_standards)


if __name__=='__main__': unittest.main()

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

    def test_nonfinite_call_cap_is_rejected_before_starting_a_process(self):
        cfg = settings(ROOT)
        cfg['checks']['proof_case_seconds'] = 'nan'
        with patch.object(check, 'settings', return_value=cfg), self.assertRaises(ValueError):
            self.checker()

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
        self.assertEqual([s for s,p in observed],[s for s in range(1,6) for _ in check.FAMILIES])
        self.assertTrue(set(check.OBLIGATIONS).issubset(report['required']))
        self.assertIn('foundation', report['required'])
        self.assertIn('safety', report['required'])
        self.assertEqual(len({p for s,p in observed}),1)
        self.assertEqual(report['status'],'INCOMPLETE')

    def test_safety_never_calls_cargo(self):
        def verus(self, kind, entry=None, obligation=None):
            return self.record(obligation or kind, 'OPEN', 'SIMULATED')
        with patch.object(check.Checks, 'cargo', side_effect=AssertionError('Cargo must not run')), patch.object(check.Checks,'verus',verus):
            report = check.evaluate(self.workspace, output=self.directory/'formal-only', mode='safety', timeout=10)
        self.assertTrue(set(check.OBLIGATIONS).issubset(report['required']))

    def test_hung_test_cannot_consume_proof_reservation(self):
        clock = [0.0]
        calls = []
        def tool(argv, cwd, **kwargs):
            calls.append((argv, kwargs['timeout']))
            if '--no-run' in argv: return simulated_output()
            if '--list' in argv:
                return simulated_output('\n'.join(n+': test' for n in check.required_tests(ROOT)))
            if 'test' in argv:
                clock[0] += kwargs['timeout']
                return simulated_output('', None, 'TIMEOUT')
            return simulated_output('verification results:: 1 verified, 0 errors')
        with patch.object(check.time, 'monotonic', side_effect=lambda:clock[0]), patch.object(check,'run_process',side_effect=tool):
            report = check.evaluate(self.workspace, output=self.directory/'hung-test', timeout=10, mode='feedback')
        test = next(v for k,v in report['checks'].items() if k.startswith('tests_seed'))
        self.assertEqual(test['reason'],'test_timeout')
        self.assertEqual(report['checks']['foundation']['status'],'PASS')
        self.assertTrue(any('--no-cheating' in argv and seconds >= 6 for argv,seconds in calls))

    def test_shared_errors_are_grouped_by_function_and_location(self):
        c = self.checker()
        path=c.snapshot/'madraft/src/toy.rs'
        path.write_text('proof fn failing_dependency() {\n    assert(false);\n}\n')
        raw=f'error: assertion failed\n  --> {path}:2:5\n'
        errors=check.error_locations(raw,c.snapshot,'election_safety')
        self.assertEqual(errors[0]['function'],'failing_dependency')
        items={n:dict(status='OPEN',reason='verification_failure',errors=errors,log=n+'.log') for n in check.SAFETY}
        groups=check.group_diagnostics(items)
        self.assertEqual(len(groups),1)
        self.assertEqual(groups[0]['affected'],list(check.SAFETY))
        self.assertEqual(len(groups[0]['logs']),5)

    def test_unavailable_verifier_is_reported_once_with_unrun_dependents(self):
        c=self.checker()
        with patch.object(check,'run_process',return_value=simulated_output('SIMULATED missing verifier',None,'BLOCKED')) as process:
            c.verus('foundation')
            result=c.verus('definitions')
        self.assertEqual(process.call_count,1)
        self.assertEqual(result['status'],'BLOCKED')
        self.assertEqual(result['category'],'not_run')
        self.assertEqual(result['dependency'],'foundation')

    def test_shared_dependency_compile_failure_is_not_rerun_or_passed(self):
        c=self.checker()
        entry=c.snapshot/'madraft/proof/implementation.rs'
        entry.write_text('pub fn bad() { unknown(); }')
        raw=f'error[E0425]: cannot find function unknown\n  --> {entry}:1:16\n'
        with patch.object(check,'run_process',return_value=simulated_output(raw,1)) as process:
            c.verus('top',obligation='election_safety')
            item=c.verus('top',obligation='log_matching')
        self.assertEqual(process.call_count,1)
        self.assertEqual(item['status'],'OPEN')
        self.assertEqual(item['dependency'],'election_safety')

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
        (c.snapshot/'madraft/proof/implementation.rs').write_text('// SIMULATED candidate interface')
        with patch.object(check, 'run_process', return_value=simulated_output('verification results:: 1 verified, 0 errors')):
            c.verus('top', obligation='election_safety')
        caller = (c.output / 'election_safety-caller.rs').read_text()
        self.assertIn(str(c.snapshot / 'requirements/targets.rs'), caller)
        self.assertNotIn(str(ROOT / 'requirements/targets.rs'), caller)

    def test_safety_cli_requests_formal_only(self):
        with patch.object(sys, 'argv', ['check.py', 'safety', '--json']), \
             patch.object(check, 'evaluate', return_value={'status': 'INCOMPLETE'}) as evaluate, \
             contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(check.main(), 1)
        evaluate.assert_called_once()
        self.assertEqual(evaluate.call_args.kwargs['mode'], 'safety')

    def test_single_caller_keeps_fixed_target_and_no_function_filter(self):
        path=self.workspace/'madraft/proof/implementation.rs'
        text=check.caller_text(ROOT,path,'election_safety')
        self.assertIn(str(ROOT/'requirements/targets.rs'),text)
        self.assertIn('prove_election_safety',text)
        self.assertNotIn('prove_log_matching',text)
        full=check.caller_text(ROOT,path)
        for goal in check.OBLIGATIONS:self.assertIn('// obligation: '+goal,full)
        c=self.checker()
        (c.snapshot/'madraft/proof/implementation.rs').write_text('// SIMULATED candidate interface')
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
               (simulated_output('error: unsupported construct',1),'verifier_error'),
               (simulated_output('failed to download required dependency',1),'dependency_or_toolchain_unavailable'),
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

    def test_model_pass_and_vacuous_snapshot_keep_actual_connections_open(self):
        c = self.checker()
        for goal in check.OBLIGATIONS: c.record(goal, 'PASS', 'SIMULATED model statement')
        c.connections()
        report = c.finish('final', list(c.results))
        self.assertEqual(report['checks']['election_safety']['status'], 'PASS')
        self.assertEqual(report['assessments']['election_safety']['implementation_task'], 'OPEN')
        self.assertEqual(report['assessments']['snapshot_observation']['formula_status'], 'PASS')
        self.assertIn('madraft/src/raft/raft.rs::snapshot', report['placeholders'])
        self.assertIn('recovery', report['assessments']['election_safety']['open_connections'])
        self.assertEqual(report['status'], 'INCOMPLETE')

    def nominate_connections(self):
        (self.workspace/'madraft/src/connection_toy.rs').write_text('pub fn actual() {}')
        (self.workspace/'madraft/proof/calls.rs').write_text('#[path = "../src/connection_toy.rs"] mod toy;')
        rows = ['<!-- connections -->', '| Behavior | Actual functions | Proof entry | Open premises / next step |']
        rows += [f'| {area} | madraft/src/connection_toy.rs::actual | madraft/proof/calls.rs | Claimed complete |' for area in check.CONNECTIONS]
        (self.workspace/'PROGRESS.md').write_text('\n'.join(rows+['<!-- /connections -->']))

    def test_verified_nominated_units_and_table_are_not_semantic_acceptance(self):
        self.nominate_connections()
        c = self.checker()
        for goal in check.OBLIGATIONS: c.record(goal, 'PASS', 'SIMULATED')
        with patch.object(check, 'run_process', return_value=simulated_output('verification results:: 1 verified, 0 errors')) as proc:
            c.connections()
        self.assertEqual(proc.call_count, 1)  # Same snapshot/unit, one real invocation.
        report = c.finish('final', list(c.results))
        self.assertEqual(report['status'], 'MACHINE_CHECKS_PASSED')  # Only a SIMULATED aggregation test.
        self.assertEqual(report['semantic_review'], 'NOT_PERFORMED')
        self.assertEqual(report['implementation_acceptance'], 'NOT_ESTABLISHED')
        self.assertEqual(report['assessments']['election_safety']['implementation_task'], 'SEMANTIC_REVIEW_REQUIRED')
        self.assertEqual(report['checks']['connection_recovery']['claim_status'], 'UNVERIFIED_DECLARATION')

    def test_missing_actual_function_reference_never_becomes_checked_coverage(self):
        self.nominate_connections()
        path = self.workspace/'PROGRESS.md'
        path.write_text(path.read_text().replace('::actual', '::invented'))
        c = self.checker()
        with patch.object(check, 'run_process', return_value=simulated_output('verification results:: 1 verified, 0 errors')):
            c.connections()
        self.assertTrue(all(c.results['connection_'+a]['reason']=='outer_paths_open' for a in check.CONNECTIONS))
        self.assertTrue(all(c.results['connection_'+a+'_unit']['status']=='PASS' for a in check.CONNECTIONS))

    def test_same_content_is_rechecked_and_changed_source_or_seed_never_reuses_pass(self):
        with patch.object(check, 'run_process', return_value=simulated_output('verification results:: 1 verified, 0 errors')) as proc:
            a=self.checker();a.verus('proof')
            b=self.checker();b.verus('proof')
            self.assertEqual(a.input_fingerprint,b.input_fingerprint)
            path=self.workspace/'madraft/src/raft/state.rs';path.write_text(path.read_text()+'\n// changed\n')
            c=self.checker();c.verus('proof')
            self.assertNotEqual(a.input_fingerprint,c.input_fingerprint)
            c.cargo('test',seed=1);c.cargo('test',seed=2)
        self.assertEqual(proc.call_count,5)
        self.assertEqual(c.results['tests_seed_2']['seed'],2)
        self.assertNotEqual(a.results['proof']['log'],b.results['proof']['log'])

    def test_cross_candidate_results_cannot_be_assembled(self):
        c=self.checker();c.record('model','PASS','SIMULATED')
        c.results['model']['candidate_revision']='another-candidate'
        report=c.finish('final',['model'])
        self.assertEqual(report['status'],'INCOMPLETE')
        self.assertEqual(report['evidence_mismatches'],['model'])

    def test_rotation_is_deterministic_and_reaches_every_family(self):
        history={}; seen=[]
        for round in range(len(check.FAMILIES)):
            plan=check.feedback_families(['src/raft/raft.rs'],history,1)
            seen.append(plan[0])
            # A hung first test exhausted functional time: later checks NOT RUN.
            history[plan[0]]={'last_attempt':round}
            self.assertIn('raft::tests::',plan)
        self.assertEqual(seen,list(check.FAMILIES))
        self.assertEqual(check.feedback_families(['unknown.rs'],{},1),list(check.FAMILIES))
        with self.assertRaises(ValueError):check.feedback_families([],{},0)


if __name__=='__main__': unittest.main()

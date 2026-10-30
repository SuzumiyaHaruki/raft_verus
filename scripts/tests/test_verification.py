"""Real Rust/Verus micro-examples. No simulated verifier or complete Raft fixture."""
from pathlib import Path
import shlex
import sys
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check
from process import run_process
from tools import ROOT, environment
from inputs import save_json


class RealVerusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg, cls.env = environment()
        if not Path(cls.cfg['verus']).is_file() or not (Path(cls.cfg['rust_bin'])/'rustc').is_file():
            path = ROOT / 'runs/maintenance-v4/real-tools-BLOCKED.json'
            save_json(path, dict(status='BLOCKED', reason='Actual Rust/Verus unavailable; real checks NOT RUN'))
            raise unittest.SkipTest('BLOCKED: actual Rust/Verus unavailable; real checks NOT RUN')
        cls.logs = ROOT / 'runs/maintenance-v4' / ('real-tools-' + str(time.time_ns()))
        cls.logs.mkdir(parents=True)

    def execute(self, name, argv, directory):
        result=run_process(argv,directory,env=self.env,timeout=60)
        (self.logs/(name+'.log')).write_text('$ '+shlex.join(argv)+'\n'+result.stdout+result.stderr+
                                            f'\nprocess={result.status} exit={result.exit_code}\n')
        save_json(self.logs/(name+'.json'), dict(command=argv, process=result.status,
            exit_code=result.exit_code, elapsed_seconds=result.elapsed_seconds,
            result_kind='REAL_TOOL_EXECUTION', log=str(self.logs/(name+'.log'))))
        self.assertEqual(result.status,'COMPLETED',result.stderr)
        return result

    def verus(self, name, source):
        path=self.logs/(name+'.rs');path.write_text(source)
        return self.execute(name,[self.cfg['verus'],str(path),'--crate-type','lib','--no-cheating','--triggers-mode','silent'],self.logs)

    def test_same_executable_counter_rejected_then_repaired_by_rust_and_verus(self):
        template='''
use vstd::prelude::*;
verus! {
pub fn increment(n: u64) -> (r: u64)
    requires n < u64::MAX,
    ensures r == n + 1,
{ BODY }
}
#[cfg(test)] mod tests {
    #[test] fn actual_increment() { assert_eq!(super::increment(2),3); }
}
'''
        for label,body,ok in [('counter-defect','n',False),('counter-repaired','n + 1',True)]:
            with self.subTest(label=label):
                proof=self.verus(label,template.replace('BODY',body))
                self.assertEqual(proof.exit_code==0,ok,proof.stdout+proof.stderr)
                if not ok:self.assertEqual(check.classify(proof,True)[1],'verification_failure')
                source=self.logs/(label+'.rs');binary=self.logs/(label+'-test')
                vstd=Path(self.cfg['verus']).parent
                build=self.execute(label+'-rustc',[str(Path(self.cfg['rust_bin'])/'rustc'),str(source),
                    '--crate-name','counter_example','--edition=2021','--test','--extern',f'vstd={vstd / "libvstd.rlib"}',
                    '-L',f'dependency={vstd}','-o',str(binary)],self.logs)
                self.assertEqual(build.exit_code,0,build.stderr)
                tests=self.execute(label+'-rust-test',[str(binary)],self.logs)
                self.assertEqual(tests.exit_code==0,ok,tests.stdout+tests.stderr)

    def test_wrong_existing_role_observer_is_rejected(self):
        original=(ROOT/'madraft/src/raft/state.rs').read_text()
        mutated=original.replace('matches!(self.role, Role::Leader)','false').replace('pub(super)','pub')
        result=self.verus('negative-accessor',mutated)
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('postcondition not satisfied',result.stderr)

    def test_independent_election_target_rejects_two_leaders(self):
        source='''
#[path = "TARGETS"] mod targets;
use targets::*;
use vstd::prelude::*;
verus! {
proof fn incorrect_claim() {
    let h = History { states: Seq::empty(), events: seq![
        Event::Elect { who: Key { group: 1, node: 0 }, term: 2 },
        Event::Elect { who: Key { group: 1, node: 1 }, term: 2 }] };
    assert(election_safety(h));
}
}
'''.replace('TARGETS',str(ROOT/'requirements/targets.rs'))
        result=self.verus('negative-election',source)
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('assertion failed',result.stderr)

    def toy_adapter(self, name, wrong=False):
        # Reuse only the new protocol-independent example, never an old Raft run.
        toy = self.logs/'toy.rs'
        source = (ROOT/'requirements/support_example.rs').read_text().replace(
            '#[path = "support.rs"]\nmod support;\nuse support::*;', 'use crate::support::*;')
        toy.write_text(source)
        module = self.logs/(name+'-module.rs')
        module.write_text('''#[path = "toy.rs"] mod toy;
use vstd::prelude::*;
use crate::{support::*, targets, Execution, execution};
pub use toy::{initial as runtime_initial, step as runtime_step, receive, emitted as emit,
              model_initial as initial, bounded as runtime_invariant};
verus! {
pub type RuntimeState = u64;
pub type State = nat;
pub type Input = Option<Delivery<u64>>;
pub type Output = (u64, u64);
pub type Message = u64;
pub open spec fn view(s: u64) -> nat { s as nat }
pub open spec fn step(s: nat, i: Input, t: nat, o: Output) -> bool {
    toy::model_step(s,i,t,o) WRONG
}
pub open spec fn refines_executable() -> bool { true } // Deliberate decoy, never consumed.
pub proof fn prove_runtime_initial(s: u64)
    requires runtime_initial(s),
    ensures runtime_invariant(s, Seq::empty()), initial(view(s)), {}
pub proof fn prove_runtime_step(s: u64, i: Input, t: u64, o: Output,
    a: Seq<Envelope<u64>>, b: Seq<Envelope<u64>>)
    requires runtime_invariant(s,a), runtime_step(s,i,t,o), exchange(a,receive(i),emit(o),b),
    ensures runtime_invariant(t,b), step(view(s),i,view(t),o), {}
// Empty observations intentionally fail nonempty coverage; this is not Raft.
pub open spec fn observe(h: Execution) -> targets::History {
    targets::History { states: seq![Map::empty()], events: Seq::empty() }
}
pub open spec fn applications() -> Map<nat, targets::Application> { Map::empty() }
pub proof fn prove_observation_and_snapshots(h: Execution)
    requires execution(h),
    ensures targets::shape(observe(h)), targets::snapshot_fidelity(observe(h), applications()), {}
}'''.replace('WRONG', '&& t == s + 2' if wrong else ''))
        for goal in check.SAFETY:
            with module.open('a') as file:
                file.write('verus! { pub proof fn prove_'+goal+'(h: Execution) requires execution(h), '
                           'ensures targets::'+goal+'(observe(h)), {} }\n')
        return module

    def test_foundation_and_actual_toy_history_verify(self):
        result = self.execute('foundation', [self.cfg['verus'], str(ROOT/'requirements/support_example.rs'),
            '--crate-type', 'lib', '--no-cheating', '--triggers-mode', 'silent'], self.logs)
        self.assertEqual(check.classify(result, True), ('PASS', 'checked'), result.stderr)

    def test_fixed_execution_caller_checks_relation_projection(self):
        for name, wrong in [('toy-link-correct', False), ('toy-link-incorrect', True)]:
            module = self.toy_adapter(name, wrong)
            result = self.verus(name, check.caller_text(ROOT, module, 'execution_link'))
            self.assertEqual(result.exit_code == 0, not wrong, result.stdout + result.stderr)
            if wrong:
                self.assertEqual(check.classify(result, True)[1], 'verification_failure')
                self.assertIn('postcondition not satisfied', result.stderr)

    def example_source(self):
        return (ROOT/'requirements/support_example.rs').read_text().replace(
            '"support.rs"', '"' + str(ROOT/'requirements/support.rs') + '"')

    def test_actual_function_mismatch_does_not_invalidate_the_model_theorem(self):
        source = self.example_source()
        # Remove only executable handler and its call examples for the separate
        # unchanged-model check. Never change the model to match the defect.
        model = source[:source.index('pub fn handle(')] + source[source.index('pub proof fn network_examples'):]
        model = model[:model.index('pub fn actual_bug_is_admitted')] + model[model.index('// A memory-sized'):]
        model_result = self.verus('model-still-passes', model)
        self.assertEqual(check.classify(model_result, True), ('PASS', 'checked'), model_result.stderr)
        wrong = source.replace('let t = if s < 100 { s + 1 } else { s };', 'let t = if s < 100 { s + 2 } else { s };')
        self.assertNotEqual(source, wrong)
        actual = self.verus('actual-handler-plus-two-rejected', wrong)
        self.assertEqual(check.classify(actual, True)[1], 'verification_failure')
        self.assertIn('postcondition not satisfied', actual.stderr)
        self.assertNotIn('error[E', actual.stderr)
        repaired = self.verus('actual-handler-plus-one-accepted', source)
        self.assertEqual(check.classify(repaired, True), ('PASS', 'checked'), repaired.stderr)

    def test_recovery_must_establish_actual_callers_precondition(self):
        source = self.example_source()
        wrong = source.replace('        consistent(*c), // recovery establishes the callee\'s precondition\n', '').replace(
            '    c.recorded = d.recorded; // derive ghost relation from the checkpoint\n', '')
        self.assertNotEqual(source, wrong)
        result = self.verus('recovery-caller-rejected', wrong)
        self.assertEqual(check.classify(result, True)[1], 'verification_failure')
        self.assertIn('precondition not satisfied', result.stderr)
        self.assertIn('update(&mut restarted)', result.stderr)
        self.assertNotIn('error[E', result.stderr)
        repaired = self.verus('recovery-caller-accepted', source)
        self.assertEqual(check.classify(repaired, True), ('PASS', 'checked'), repaired.stderr)

    def test_single_top_caller_does_not_skip_other_imported_bodies(self):
        module = self.toy_adapter('imported-dependency')
        with module.open('a') as file:
            file.write('verus! { pub proof fn broken_dependency() { assert(false); } }')
        result = self.verus('negative-imported-body',check.caller_text(ROOT,module,'election_safety'))
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('assertion failed',result.stderr)
        self.assertNotIn('error[E', result.stderr)

    def test_aggregate_keeps_nonempty_observation_requirement(self):
        module = self.toy_adapter('empty-observation')
        with module.open('a') as file:
            file.write('''verus! {
pub proof fn execution_witness() -> (h: crate::RuntimeExecution)
    ensures crate::runtime_execution(h), targets::shape(observe(crate::abstract_execution(h))),
{
    History { states: seq![0u64], inputs: Seq::empty(), outputs: Seq::empty(), sent: seq![Seq::empty()] }
}
}''')
        result = self.verus('negative-empty-composition',check.caller_text(ROOT,module))
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('assertion failed',result.stderr)
        self.assertNotIn('error[E',result.stderr)

    def test_unsent_receive_cannot_be_proved_valid(self):
        text='''#[path = "SUPPORT"] mod support;
use support::*;
use vstd::prelude::*;
verus! { proof fn wrong_network_claim() {
    let m = Envelope { group: 1, sender: 0, recipient: 1, body: 7u64 };
    assert(delivered(Seq::empty(), Delivery { send: 0, message: m }));
} }
'''.replace('SUPPORT',str(ROOT/'requirements/support.rs'))
        result=self.verus('negative-unsent-delivery',text)
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('assertion failed',result.stderr)

    def test_external_consensus_body_cannot_close_a_proof(self):
        result=self.verus('negative-external-body','''
use vstd::prelude::*;
verus! {
#[verifier::external_body]
pub fn incorrect(n:u64) -> (r:u64) ensures r == n, { 7 }
}
''')
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('external_body',result.stdout+result.stderr)


if __name__=='__main__': unittest.main()

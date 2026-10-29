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


class RealVerusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg, cls.env = environment()
        if not Path(cls.cfg['verus']).is_file() or not (Path(cls.cfg['rust_bin'])/'rustc').is_file():
            raise unittest.SkipTest('Actual Rust/Verus unavailable; real checks NOT RUN')
        cls.logs = ROOT / 'runs/maintenance-revision' / ('real-tools-' + str(time.time_ns()))
        cls.logs.mkdir(parents=True)

    def execute(self, name, argv, directory):
        result=run_process(argv,directory,env=self.env,timeout=60)
        (self.logs/(name+'.log')).write_text('$ '+shlex.join(argv)+'\n'+result.stdout+result.stderr+
                                            f'\nprocess={result.status} exit={result.exit_code}\n')
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

    def test_single_top_caller_does_not_skip_other_imported_bodies(self):
        # Intentionally vacuous ghost-only interface, never a Raft implementation.
        module=self.logs/'invalid-dependency.rs'
        module.write_text('''
use vstd::prelude::*;
use crate::targets;
verus! {
pub type Execution = nat;
pub open spec fn execution(run: Execution) -> bool { false }
pub open spec fn observe(run: Execution) -> targets::History {
    targets::History { states: Seq::empty(), events: Seq::empty() }
}
pub proof fn prove_election_safety(run: Execution)
    requires execution(run), ensures targets::election_safety(observe(run)), {}
pub proof fn broken_dependency() { assert(false); }
}
''')
        text=check.caller_text(ROOT,module,'election_safety')
        result=self.verus('negative-imported-body',text)
        self.assertNotEqual(result.exit_code,0)
        self.assertIn('assertion failed',result.stderr)

    def test_real_witness_caller_rejects_vacuity_and_accepts_an_observable_witness(self):
        # Ghost-only interface tests, not an executable model or a Raft proof.
        for name,execution,events,states,ok in [
            ('false-execution','false','Seq::empty()','seq![Map::empty()]',False),
            ('zero-step','true','Seq::empty()','seq![Map::empty()]',False),
            ('silent-observation','true','seq![targets::Event::Silent]','seq![Map::empty(), Map::empty()]',False),
            ('observable-witness','true',
             'seq![targets::Event::Apply { who: targets::Key { group: 0, node: 0 }, index: 1, entry: targets::Entry { term: 1, command: Seq::empty() } }]',
             'seq![Map::empty(), Map::empty()]',True)]:
            with self.subTest(name=name):
                module=self.logs/(name+'-module.rs')
                module.write_text('''
use vstd::prelude::*;
use crate::targets;
verus! {
pub type Execution = nat;
pub open spec fn execution(run: Execution) -> bool { EXECUTION }
pub open spec fn observe(run: Execution) -> targets::History {
    targets::History { states: STATES, events: EVENTS }
}
pub proof fn execution_witness() -> (run: Execution) ensures run == 0nat, { 0 }
}
'''.replace('EXECUTION',execution).replace('STATES',states).replace('EVENTS',events))
                if ok:
                    # Supply the required witness contract, not just a return
                    # value. This synthetic history still proves no executable linkage.
                    module.write_text(module.read_text().replace(
                        'ensures run == 0nat, { 0 }', '''
    ensures execution(run), targets::shape(observe(run)),
        exists|i: int| 0 <= i < observe(run).events.len()
            && !(observe(run).events[i] is Silent),
{
    assert(!(observe(0nat).events[0] is Silent));
    0
}'''))
                result=self.verus('witness-'+name,check.caller_text(ROOT,module,'nonempty_execution'))
                self.assertEqual(result.exit_code == 0, ok, result.stdout + result.stderr)
                if not ok:
                    self.assertEqual(check.classify(result,True)[1],'verification_failure',result.stderr)

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

"""Actual Verus negative checks; no simulated verifier or protocol generator."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from process import run_process
from tools import ROOT, environment


class RealVerusTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cfg, cls.env = environment()
        if not Path(cls.cfg['verus']).is_file():
            raise unittest.SkipTest('Actual Verus unavailable; negative checks NOT RUN')

    def reject(self, source, name, diagnostic):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / 'negative.rs'
            file.write_text(source)
            result = run_process([self.cfg['verus'], str(file), '--crate-type', 'lib',
                                  '--no-cheating', '--triggers-mode', 'silent'], ROOT,
                                 env=self.env, timeout=60)
        logs = ROOT / 'runs/preparation'
        logs.mkdir(parents=True, exist_ok=True)
        (logs / f'{name}.log').write_text(result.stdout + result.stderr)
        self.assertNotEqual(result.exit_code, 0)
        self.assertEqual(result.status, 'COMPLETED')
        self.assertIn(diagnostic, result.stderr)

    def test_wrong_executable_accessor_fails_its_real_contract(self):
        original = (ROOT / 'madraft/src/raft/state.rs').read_text()
        self.assertIn('matches!(self.role, Role::Leader)', original)
        mutated = original.replace('matches!(self.role, Role::Leader)', 'false')
        # A root module cannot use pub(super); visibility is irrelevant to this mutation.
        mutated = mutated.replace('pub(super)', 'pub')
        self.reject(mutated, 'negative-accessor', 'postcondition not satisfied')

    def test_independent_election_target_rejects_two_leaders(self):
        source = '''
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
'''.replace('TARGETS', str(ROOT / 'requirements/targets.rs'))
        self.reject(source, 'negative-election', 'assertion failed')


if __name__ == '__main__':
    unittest.main()

"""Actual Verus rejects vacuous implementations and altered local contracts."""
import tempfile
import unittest
from pathlib import Path

from orchestrator.config import load_config, profile_by_id
from orchestrator.verifier_runner import verify_profile
from orchestrator.workspace import create_candidate
from orchestrator.tests.real_repair_demo import contract_edit

ROOT = Path(__file__).resolve().parents[2]


class NodeContractTests(unittest.TestCase):
    def test_completion_contract_cannot_weaken_its_independent_target(self):
        config = load_config(ROOT)
        if not Path(config['toolchain']['verus_executable']).is_file():
            self.skipTest('Actual Verus missing; no simulated proof substituted')
        profile = profile_by_id(config, 'persist-proof')
        with tempfile.TemporaryDirectory() as location:
            outside = Path(location)
            candidate = create_candidate(ROOT, outside/'candidate')
            edit = contract_edit()
            (candidate/edit['path']).write_text(edit['text'])
            source = candidate/'crates/raft-verified/src/persistence.rs'
            # Scripted solution is explicit test setup, never a fresh candidate input.
            text = (ROOT/'orchestrator/tests/fixtures/solutions/persistence.rs').read_text()
            contract = 'ensures crate::contracts::complete_result(*old(self), *self, operation, kind, outcome, result),'
            for name, changed in [('weakened', text.replace(contract, 'ensures true,')),
                                  ('stronger', text.replace('requires old(self).wf(),', 'requires false,'))]:
                source.write_text(changed)
                result = verify_profile(candidate, profile, outside/name)
                self.assertEqual(result['result'], 'FAIL', result)
                self.assertFalse(result.get('function_results', {}).get('persistence::check_complete', True))
                self.assertEqual(result['obligation_ids_checked'], [])

    def test_seed_weaker_postcondition_and_stronger_precondition_are_rejected(self):
        config = load_config(ROOT)
        if not Path(config['toolchain']['verus_executable']).is_file():
            self.skipTest('Actual Verus missing; no simulated proof substituted')
        profile = profile_by_id(config, 'node-proof')
        solution = (ROOT/'orchestrator/tests/fixtures/node_init_solution.rs').read_text()
        cases = {
            'reject-all': (ROOT/'crates/raft-verified/src/node.rs').read_text(),
            'weaker-contract': solution.replace('ensures crate::contracts::constructed(result, id, members@),', 'ensures true,'),
            'stronger-precondition': solution.replace('ensures crate::contracts::constructed', 'requires members.len() > 1,\n        ensures crate::contracts::constructed'),
        }
        with tempfile.TemporaryDirectory() as location:
            outside = Path(location)
            candidate = create_candidate(ROOT, outside/'candidate')
            edit = contract_edit(True)
            (candidate/edit['path']).write_text(edit['text'])
            for label, text in cases.items():
                with self.subTest(case=label):
                    (candidate/'crates/raft-verified/src/node.rs').write_text(text)
                    result = verify_profile(candidate, profile, outside/label)
                    self.assertEqual(result['result'], 'FAIL', result)
                    self.assertEqual(result['obligation_ids_checked'], [])
                    self.assertGreater(result.get('verification_summary', {}).get('errors', 0), 0)


if __name__ == '__main__':
    unittest.main()

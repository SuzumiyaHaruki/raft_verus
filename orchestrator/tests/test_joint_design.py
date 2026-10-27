"""Joint design remains accountable to actual observations and independent targets."""
import tempfile
import unittest
from pathlib import Path

from orchestrator.config import load_config, profile_by_id
from orchestrator.feedback import classify
from orchestrator.workspace import create_candidate
from orchestrator.tests.real_repair_demo import contract_edit, persistence_solution, SOLUTIONS
from orchestrator.verifier_runner import verify_profile

ROOT = Path(__file__).resolve().parents[2]


class JointDesignTests(unittest.TestCase):
    def setUp(self):
        self.config = load_config(ROOT)
        if self.config['toolchain']['resolved']['missing']:
            self.skipTest('Actual toolchain missing; no simulated checking substituted')
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.outside = Path(self.tmp.name)
        self.candidate = create_candidate(ROOT, self.outside/'candidate')
        self.number = 0
        self.write('node_state.rs', (SOLUTIONS/'redesigned_state.rs').read_text())
        self.write('node.rs', (ROOT/'orchestrator/tests/fixtures/node_init_solution.rs').read_text())
        self.write('contracts.rs', contract_edit()['text'])
        self.write('persistence.rs', persistence_solution(True))

    def write(self, name, text):
        (self.candidate/'crates/raft-verified/src'/name).write_text(text)

    def check(self, name, expected):
        self.number += 1
        result = verify_profile(self.candidate, profile_by_id(self.config, name),
                                self.outside/str(self.number))
        self.assertEqual(result['result'], expected, result)
        self.assertEqual(result['evidence_kind'], 'REAL_LOCAL')
        return result

    def test_different_layout_and_nontrivial_invariant_pass_unchanged_observable_targets(self):
        for rel in ['crates/raft-spec/src/node_init.rs', 'crates/raft-spec/src/persistence.rs',
                    'crates/raft-proofs/node_init.rs', 'crates/raft-proofs/persistence.rs',
                    'crates/raft-proofs/observation.rs', 'crates/raft-verified/tests/node_init.rs',
                    'crates/raft-verified/tests/persistence.rs']:
            self.assertEqual((self.candidate/rel).read_bytes(), (ROOT/rel).read_bytes())
        for name in ['node-tests', 'node-proof', 'persist-tests', 'persist-proof', 'persist-replay']:
            self.check(name, 'PASS')

    def test_code_satisfies_its_generated_contract_but_independent_target_rejects_it(self):
        self.write('contracts.rs', (ROOT/'crates/raft-verified/src/contracts.rs').read_text())
        node = (self.candidate/'crates/raft-verified/src/node.rs').read_text()
        self.write('node.rs', node.replace('term: 0, voted_for:', 'term: 1, voted_for:'))
        result = self.check('node-proof', 'FAIL')
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertFalse(result['function_results']['node_init::check_node_init'])
        self.assertTrue(any(name.endswith('Node::new') and passed
                            for name, passed in result['function_results'].items()))
        self.check('node-tests', 'FAIL')

    def test_false_invariant_and_unconnected_observer_cannot_discharge_bridge(self):
        original = (SOLUTIONS/'redesigned_state.rs').read_text()
        for changed, target in [
            (original.replace('self.issued == self.core.image.serial', 'false'), 'check_observation_input'),
            (original.replace('ensures *result == self.view(),', 'ensures true,'), 'check_observe'),
        ]:
            with self.subTest(target=target):
                self.write('node_state.rs', changed)
                result = self.check('node-proof', 'FAIL')
                self.assertEqual(result['obligation_ids_checked'], [])
                # A false invariant fails the executable representation constructor;
                # a weakened observer contract fails its independent consumer.
                if target == 'check_observe':
                    self.assertFalse(result['function_results']['node_init::observation::check_observe'])
                else:
                    self.assertTrue(any(name.endswith('Node::from_observation') and not passed
                                        for name, passed in result['function_results'].items()))

    def test_verus_visibility_error_is_source_repair_not_missing_tool_or_proof(self):
        original = (SOLUTIONS/'redesigned_state.rs').read_text()
        self.write('node_state.rs', original.replace('pub core:', 'pub(crate) core:'))
        result = self.check('node-proof', 'FAIL')
        self.assertEqual(result['failure_stage'], 'source-compilation')
        self.assertEqual(result['proof_result'], 'NOT_RUN')
        self.assertEqual(result['obligation_ids_checked'], [])
        self.assertEqual(classify(result)['action'], 'repair')


if __name__ == '__main__':
    unittest.main()

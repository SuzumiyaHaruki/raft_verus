"""Fresh experiments omit maintenance answers; partial generation stays partial."""
from pathlib import Path
import tempfile
import unittest

from orchestrator.config import load_config, profile_by_id
from orchestrator.scheduler import load_tasks, render_prompt
from orchestrator.verifier_runner import verify_profile
from orchestrator.workspace import create_candidate, snapshot
from orchestrator.tests.real_repair_demo import contract_edit

ROOT = Path(__file__).resolve().parents[2]


class StartingPointTests(unittest.TestCase):
    def test_fresh_copy_omits_all_test_scripts_answers_and_past_runs(self):
        with tempfile.TemporaryDirectory() as location:
            candidate = create_candidate(ROOT, Path(location)/'candidate')
            for relative in ('orchestrator/tests', 'Agent.md', 'evidence', 'target', '.git'):
                self.assertFalse((candidate/relative).exists(), relative)
            for relative in ('input/starting-point.md', 'crates/raft-verified/tests/node_init.rs',
                             'crates/raft-verified/tests/persistence.rs', 'crates/raft-proofs/node_init.rs',
                             'crates/raft-spec/src/persistence.rs', 'orchestrator/wire'):
                self.assertTrue((candidate/relative).exists(), relative)
            self.assertFalse(any(p.startswith('orchestrator/tests/') for p in snapshot(ROOT)))
            # Supply the host and API scaffold, never a preimplemented protocol slice.
            self.assertTrue((candidate/'crates/raft-verified/src/madraft.rs').is_file())
            protocol = (candidate/'crates/raft-verified/src/protocol.rs').read_text()
            self.assertIn('Err(InputError::UnsupportedEvent)', protocol)
            self.assertNotIn('fn elect', protocol)
            self.assertTrue((candidate/'experiments/madraft-poc/src/cluster.rs').is_file())
            self.assertTrue((candidate/'crates/raft-proofs/core.rs').is_file())

    def test_new_copy_does_not_inherit_another_run_or_rewrite_it(self):
        with tempfile.TemporaryDirectory() as location:
            base = Path(location)
            first = create_candidate(ROOT, base/'first')
            body = first/'crates/raft-verified/src/node.rs'
            generated = body.read_text() + '\n// SIMULATED saved attempt, not a fresh seed.\n'
            body.write_text(generated)
            second = create_candidate(ROOT, base/'second')
            self.assertEqual(body.read_text(), generated)
            self.assertEqual((second/body.relative_to(first)).read_text(),
                             (ROOT/body.relative_to(first)).read_text())
            self.assertNotEqual((second/body.relative_to(first)).read_text(), generated)

    def test_prompts_state_supplied_inputs_and_do_not_offer_answers(self):
        tasks = load_tasks(ROOT, load_config(ROOT))
        for task in tasks:
            with self.subTest(task=task['id']):
                prompt = render_prompt(ROOT, task)
                self.assertIn('Python', prompt)
                self.assertIn('supplied', prompt)
                self.assertNotIn('fixtures/solutions', prompt)
                self.assertNotIn('node_init_solution.rs', prompt)
        init = next(t for t in tasks if t['id'] == 'STATE-INIT')
        self.assertNotIn('reconcile-proof', init['check_profile_ids'])
        self.assertNotIn('persist-proof', init['check_profile_ids'])

    def test_real_initialization_can_pass_while_future_bodies_remain_unimplemented(self):
        config = load_config(ROOT)
        if config['toolchain']['resolved']['missing']:
            self.skipTest('Real toolchain unavailable; no simulated checks substituted')
        with tempfile.TemporaryDirectory() as location:
            outside = Path(location)
            candidate = create_candidate(ROOT, outside/'candidate')
            untouched = {rel: (candidate/rel).read_bytes() for rel in (
                'crates/raft-verified/src/reconcile.rs', 'crates/raft-verified/src/persistence.rs',
                'crates/raft-proofs/progress_lemmas.rs')}
            # Explicit SIMULATED generation in the test process, after the copy.
            (candidate/'crates/raft-verified/src/node.rs').write_text(
                (ROOT/'orchestrator/tests/fixtures/node_init_solution.rs').read_text())
            edit = contract_edit(True)
            (candidate/edit['path']).write_text(edit['text'])
            for name, expected in [('node-tests', 'PASS'), ('node-proof', 'PASS'),
                                   ('persist-tests', 'FAIL'), ('persist-proof', 'FAIL'),
                                   ('reconcile-proof', 'FAIL')]:
                with self.subTest(profile=name):
                    result = verify_profile(candidate, profile_by_id(config, name), outside/name)
                    self.assertEqual(result['result'], expected, result)
                    self.assertEqual(result['evidence_kind'], 'REAL_LOCAL')
                    if expected == 'FAIL' and name.endswith('proof'):
                        self.assertGreater(result['verification_summary']['errors'], 0)
                        self.assertEqual(result['obligation_ids_checked'], [])
            for rel, original in untouched.items():
                self.assertEqual((candidate/rel).read_bytes(), original)

            # Generating initialization alone must not reveal a working vote implementation.
            vote = verify_profile(candidate, profile_by_id(config, 'madraft-poc-verus'), outside/'vote-proof')
            self.assertEqual(vote['result'], 'FAIL', vote)
            self.assertTrue(vote['function_results']['madraft_vote::check_follower'], vote)
            self.assertFalse(vote['function_results']['madraft_vote::check_vote'], vote)
            self.assertFalse(vote['function_results']['madraft_vote::check_durable_reply'], vote)
            self.assertEqual(vote['obligation_ids_checked'], [])

    def test_public_event_contract_and_useful_behavior_are_checked_independently(self):
        config = load_config(ROOT)
        if config['toolchain']['resolved']['missing']:
            self.skipTest('Real toolchain unavailable; no simulated checks substituted')
        with tempfile.TemporaryDirectory() as location:
            outside = Path(location)
            candidate = create_candidate(ROOT, outside/'candidate')
            profile = profile_by_id(config, 'core-proof')
            missing = verify_profile(candidate, profile, outside/'missing-contract')
            self.assertEqual(missing['result'], 'FAIL', missing)
            self.assertGreater(missing['verification_summary']['errors'], 0)
            self.assertEqual(missing['obligation_ids_checked'], [])
            # SIMULATED contract edit, not a protocol implementation. Real Verus follows.
            body = candidate/'crates/raft-verified/src/protocol.rs'
            body.write_text(body.read_text().replace('ensures self.wf(),',
                            'ensures self.wf(), result.is_err(),'))
            barrier = verify_profile(candidate, profile, outside/'reject-all-barrier')
            self.assertEqual(barrier['result'], 'PASS', barrier)
            behavior = verify_profile(candidate, profile_by_id(config, 'core-vote-tests'),
                                      outside/'reject-all-behavior')
            self.assertEqual(behavior['result'], 'FAIL', behavior)
            self.assertEqual(behavior['evidence_kind'], 'REAL_LOCAL')
            # The unchanged reconciliation caller still rejects its weak local contract.
            merge = verify_profile(candidate, profile_by_id(config, 'reconcile-proof'), outside/'merge')
            self.assertEqual(merge['result'], 'FAIL', merge)
            self.assertGreater(merge['verification_summary']['errors'], 0)

    def test_actual_progress_proof_needs_a_generated_strategy(self):
        config = load_config(ROOT)
        if config['toolchain']['resolved']['missing']:
            self.skipTest('Real toolchain unavailable; no simulated checks substituted')
        with tempfile.TemporaryDirectory() as location:
            outside = Path(location)
            candidate = create_candidate(ROOT, outside/'candidate')
            solutions = ROOT/'orchestrator/tests/fixtures/solutions'
            (candidate/'crates/raft-verified/src/persistence.rs').write_text(
                (solutions/'persistence.rs').read_text())
            edit = contract_edit()
            (candidate/edit['path']).write_text(edit['text'])
            profile = profile_by_id(config, 'progress-proof')
            missing = verify_profile(candidate, profile, outside/'missing-strategy')
            self.assertEqual(missing['result'], 'FAIL', missing)
            self.assertTrue(any('progress_lemmas' in name and not passed
                                for name, passed in missing['function_results'].items()), missing)
            self.assertEqual(missing['obligation_ids_checked'], [])
            (candidate/'crates/raft-proofs/progress_lemmas.rs').write_text(
                (solutions/'progress_lemmas.rs').read_text())
            generated = verify_profile(candidate, profile, outside/'scripted-strategy')
            self.assertEqual(generated['result'], 'PASS', generated)
            self.assertEqual(generated['evidence_kind'], 'REAL_LOCAL')


if __name__ == '__main__':
    unittest.main()

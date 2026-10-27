"""The Markdown interface retains the original semantic-reference safeguards."""
from pathlib import Path
import tempfile
import unittest

from orchestrator.cli import validate_repository
from orchestrator.config import load_config
from orchestrator.workspace import create_candidate

REPO = Path(__file__).resolve().parents[2]


class RegistryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='raft-registry-test-')
        self.root = Path(self.tmp.name)/'source'
        create_candidate(REPO, self.root)
        self.config = load_config(self.root)

    def tearDown(self):
        self.tmp.cleanup()

    def change_row(self, identifier, column, value):
        path = self.root/'input/proofs.md'
        lines = path.read_text().splitlines()
        headers = next(line for line in lines if line.startswith('| ID | Status'))
        index = [part.strip() for part in headers.strip('|').split('|')].index(column)
        for n, line in enumerate(lines):
            if line.startswith('| '+identifier+' |'):
                cells = [part.strip() for part in line.strip('|').split('|')]
                cells[index] = value
                lines[n] = '| '+' | '.join(cells)+' |'
                break
        path.write_text('\n'.join(lines)+'\n')

    def test_actual_registry_structurally_valid(self):
        result = validate_repository(self.root, self.config)
        self.assertEqual(result['status'], 'PASS', result)
        self.assertFalse(result['full_acceptance'])

    def test_unknown_assumption_is_rejected(self):
        self.change_row('PO-INIT', 'Assumptions', 'UNKNOWN')
        self.assertEqual(validate_repository(self.root, self.config)['status'], 'FAIL')

    def test_review_status_is_separate_from_proof_completion(self):
        path = self.root/'input/requirements.md'
        path.write_text(path.read_text().replace('Status: DRAFT.', 'Status: APPROVED.'))
        result = validate_repository(self.root, self.config)
        self.assertFalse(any('must remain DRAFT' in error for error in result['errors']))
        self.assertFalse(result['full_acceptance'])

    def test_safety_cannot_inherit_liveness_assumption(self):
        self.change_row('PO-INIT', 'Assumptions', 'A-L-HOST')
        result = validate_repository(self.root, self.config)
        self.assertEqual(result['status'], 'FAIL')
        self.assertTrue(any('safety depends' in x for x in result['errors']))

    def test_unproved_checked_status_rejected(self):
        self.change_row('PO-INIT', 'Status', 'CHECKED')
        self.assertEqual(validate_repository(self.root, self.config)['status'], 'FAIL')

    def test_liveness_cannot_be_removed_to_get_full_pass(self):
        path = self.root/'input/properties.md'
        path.write_text('\n'.join(line for line in path.read_text().splitlines()
                                  if not line.startswith('| L-DELIVER /'))+'\n')
        self.assertEqual(validate_repository(self.root, self.config)['status'], 'FAIL')

    def test_coverage_and_profile_mismatches_rejected(self):
        next(profile for profile in self.config['profiles'] if profile['id'] == 'full')['expected_obligation_ids'] = []
        self.assertEqual(validate_repository(self.root, self.config)['status'], 'FAIL')

    def test_malformed_markdown_and_duplicate_obligations_rejected(self):
        path = self.root/'input/proofs.md'
        original = path.read_text()
        line = next(x for x in original.splitlines() if x.startswith('| PO-INIT |'))
        for replacement in (line+'\n'+line, line+' extra |'):
            path.write_text(original.replace(line, replacement, 1))
            self.assertEqual(validate_repository(self.root, self.config)['status'], 'FAIL')


if __name__ == '__main__':
    unittest.main()

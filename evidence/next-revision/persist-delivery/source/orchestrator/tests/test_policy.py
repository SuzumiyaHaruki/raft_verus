from pathlib import Path
import tempfile
import unittest

from orchestrator.policy import check_changes, scan_trust
from orchestrator.workspace import snapshot


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        for path in ('input/requirements.md', 'orchestrator/checker.py', 'Cargo.lock', 'src/lib.rs'):
            p = self.root/path
            p.parent.mkdir(exist_ok=True)
            p.write_text('original')
        self.before = snapshot(self.root)
        self.task = {'permitted_change_paths': ['src'], 'forbidden_change_paths': ['secrets']}
        self.protected = ['contracts', 'orchestrator', 'Cargo.lock']

    def tearDown(self):
        self.tmp.cleanup()

    def findings(self):
        return check_changes(self.before, snapshot(self.root), self.task, self.protected)

    def test_contract_checker_lock_and_deleted_file_are_protected(self):
        for path in ('input/requirements.md', 'orchestrator/checker.py', 'Cargo.lock'):
            (self.root/path).write_text('changed')
        (self.root/'input/requirements.md').unlink()
        self.assertEqual({f['path'] for f in self.findings()},
                         {'input/requirements.md', 'orchestrator/checker.py', 'Cargo.lock'})

    def test_untracked_outside_scope_rejected(self):
        (self.root/'new-file').write_text('untracked')
        self.assertTrue(self.findings())

    def test_symlink_rejected_even_within_permitted_scope(self):
        (self.root/'src/link').symlink_to('/tmp')
        self.assertTrue(self.findings())

    def test_allowed_source_change(self):
        (self.root/'src/lib.rs').write_text('fn lemma() {}')
        self.assertEqual(self.findings(), [])

    def test_scanner_distinguishes_comments_and_strings(self):
        source = self.root/'src/lib.rs'
        source.write_text('// assume(false)\n/* external_body /* unsafe */ */\n'
                          'fn f() { let s = r##"assume unsafe"##; assume(false); }\n'
                          '#[verifier::external_body]\nfn g() {}\n')
        findings = scan_trust(self.root, ['src/lib.rs'])
        self.assertEqual([f['token'] for f in findings], ['assume', 'external_body'])

    def test_scanner_flags_unfinished_unsafe_and_conditional_coverage(self):
        (self.root/'src/lib.rs').write_text('#[cfg(feature="skip")] unsafe fn x() { todo!(); }')
        self.assertEqual({f['token'] for f in scan_trust(self.root, ['src/lib.rs'])}, {'cfg', 'unsafe', 'todo'})


if __name__ == '__main__':
    unittest.main()

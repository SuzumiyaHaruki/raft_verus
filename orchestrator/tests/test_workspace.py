from pathlib import Path
import tempfile
import unittest

from orchestrator.workspace import (changed_paths, checked_path, create_candidate,
                                     digest_json, input_manifest, normalize_path, snapshot)


class WorkspaceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.source = self.root/'source'
        self.source.mkdir()
        (self.source/'source.rs').write_text('original')

    def tearDown(self):
        self.tmp.cleanup()

    def test_content_untracked_mode_delete_rename(self):
        before = snapshot(self.source)
        (self.source/'source.rs').chmod(0o755)
        self.assertIn('source.rs', changed_paths(before, snapshot(self.source)))
        (self.source/'source.rs').rename(self.source/'renamed.rs')
        self.assertEqual(changed_paths(before, snapshot(self.source)), ['renamed.rs', 'source.rs'])
        (self.source/'new.rs').write_text('untracked')
        self.assertIn('new.rs', snapshot(self.source))

    def test_copy_preserves_source_and_refuses_existing_destination(self):
        dest = self.root/'candidate'
        create_candidate(self.source, dest)
        (dest/'source.rs').write_text('candidate')
        self.assertEqual((self.source/'source.rs').read_text(), 'original')
        with self.assertRaises(FileExistsError):
            create_candidate(self.source, dest)

    def test_symlink_escape_and_input_validation(self):
        (self.source/'escape').symlink_to(self.root)
        self.assertEqual(snapshot(self.source)['escape']['kind'], 'symlink')
        with self.assertRaises(ValueError):
            checked_path(self.source, 'escape/secret')
        with self.assertRaises(ValueError):
            create_candidate(self.source, self.root/'candidate')
        with self.assertRaises(ValueError):
            input_manifest(self.source, ['missing'])

    def test_path_traversal_absolute_backslash_and_root_rejected(self):
        for path in ('../x', '/tmp/x', 'src/../../x', 'src\\x', '.', ''):
            with self.subTest(path=path), self.assertRaises(ValueError):
                normalize_path(path)

    def test_snapshot_binding_invalidates_changed_input(self):
        before = digest_json(input_manifest(self.source, ['source.rs']))
        (self.source/'source.rs').write_text('changed')
        self.assertNotEqual(before, digest_json(input_manifest(self.source, ['source.rs'])))

    def test_nested_reserved_names_are_still_in_manifest(self):
        path = self.source/'src/evidence/hidden.rs'
        path.parent.mkdir(parents=True)
        path.write_text('must be checked')
        self.assertIn('src/evidence/hidden.rs', snapshot(self.source))


if __name__ == '__main__':
    unittest.main()

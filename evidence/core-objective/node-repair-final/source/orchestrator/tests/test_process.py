"""Offline transport tests exercise real OS subprocesses, never a model."""
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import threading
import time
import unittest

from orchestrator.process import run_process

FIXTURE = Path(__file__).parent / 'fixtures' / 'fake_codex.py'


class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='raft offline café spaces ')
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def call(self, scenario='ok', **kwargs):
        prompt = json.dumps({'scenario': scenario, 'pid_file': str(self.root/'child.pid')})
        return run_process([sys.executable, str(FIXTURE), '--output-last-message',
                            str(self.root/'report.json')], self.root, stdin=prompt, **kwargs)

    def test_success_two_streams(self):
        r = self.call()
        self.assertEqual((r.status, r.exit_code), ('COMPLETED', 0))
        self.assertIn('turn.completed', r.stdout)
        self.assertTrue((self.root/'report.json').is_file())

    def test_nonzero_auth_permission_and_signal(self):
        for scenario in ('nonzero', 'auth', 'permission', 'signal'):
            with self.subTest(scenario=scenario):
                r = self.call(scenario)
                self.assertNotEqual(r.exit_code, 0)

    def test_missing_and_nonexecutable(self):
        self.assertEqual(run_process(['/no/such/raft-fixture'], self.root).status, 'BLOCKED')
        path = self.root/'not executable'
        path.write_text('fixture')
        self.assertEqual(run_process([str(path)], self.root).status, 'BLOCKED')

    def test_timeout(self):
        r = self.call('timeout', timeout=0.1)
        self.assertEqual(r.status, 'TIMEOUT')
        self.assertLess(r.elapsed_seconds, 3)

    def test_cancel_and_descendant_cleanup(self):
        cancel = threading.Event()
        def signal_when_spawned():
            for _ in range(200):
                if (self.root/'child.pid').exists():
                    cancel.set()
                    return
                time.sleep(.01)
            cancel.set()
        thread = threading.Thread(target=signal_when_spawned)
        thread.start()
        r = self.call('child', timeout=5, cancel_event=cancel)
        thread.join()
        self.assertEqual(r.status, 'CANCELLED')
        pid = int((self.root/'child.pid').read_text())
        # An orphan may briefly be a zombie awaiting the host init's reaper.
        proc = Path(f'/proc/{pid}/stat')
        for _ in range(100):
            if not proc.exists() or proc.read_text().split()[2] == 'Z':
                break
            time.sleep(.01)
        else:
            self.fail('descendant survived process group cancellation')

    def test_large_logs_are_bounded(self):
        r = self.call('large', max_log_bytes=1024)
        self.assertEqual(r.status, 'LOG_LIMIT')
        self.assertTrue(r.stderr_truncated)
        self.assertLessEqual(len(r.stderr.encode()), 1024)

    def test_large_stdin_unicode_and_shell_characters_are_data(self):
        text = "naïve text ' \" \n $(touch INJECTED) `touch INJECTED2` ; & | " * 5000
        r = run_process([sys.executable, '-c', 'import sys; sys.stdout.write(sys.stdin.read())'],
                        self.root, stdin=text, max_log_bytes=2_000_000)
        self.assertEqual(r.stdout, text)
        self.assertFalse((self.root/'INJECTED').exists())
        self.assertFalse((self.root/'INJECTED2').exists())

    def test_both_large_streams_do_not_deadlock(self):
        code = 'import sys; sys.stdout.write("a"*200000); sys.stderr.write("b"*200000)'
        r = run_process([sys.executable, '-c', code], self.root, timeout=3, max_log_bytes=300000)
        self.assertEqual((r.status, r.exit_code), ('COMPLETED', 0))
        self.assertEqual(len(r.stdout), 200000)
        self.assertEqual(len(r.stderr), 200000)

    def test_invalid_limits_and_argv(self):
        for args, kw in [([], {}), (['echo'], {'timeout': 0}), (['echo'], {'max_log_bytes': 0})]:
            with self.assertRaises(ValueError):
                run_process(args, self.root, **kw)


if __name__ == '__main__':
    unittest.main()

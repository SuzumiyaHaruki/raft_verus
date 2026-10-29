"""Offline controller tests. Every agent process here is explicitly SIMULATED."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from process import run_process
from tools import ROOT


class LauncherTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        (ROOT / 'runs').mkdir(exist_ok=True)
        self.directory = ROOT / 'runs' / (Path(self.tmp.name).name + '-experiment')
        self.addCleanup(shutil.rmtree, self.directory, ignore_errors=True)
        self.personal_home = Path(self.tmp.name) / 'operator-codex'
        self.personal_home.mkdir()
        (self.personal_home / 'AGENTS.md').write_text('SIMULATED personal instructions')
        (self.personal_home / 'AGENTS.override.md').write_text('SIMULATED personal override')
        (self.personal_home / 'config.toml').write_text('model = "SIMULATED-default"\ndeveloper_instructions = "SIMULATED personal instructions"\n')
        (self.personal_home / 'auth.json').write_text('{"SIMULATED": "not a credential"}\n')
        (self.personal_home / 'rules').mkdir()
        (self.personal_home / 'rules/default.rules').write_text('# SIMULATED execution policy\n')
        self.env = dict(os.environ, CODEX_HOME=str(self.personal_home))
        self.fake = Path(self.tmp.name) / 'simulated_codex'
        self.fake.write_text('#!' + sys.executable + '\n' + '''
import json, os, pathlib, sys, time
with pathlib.Path('simulated-homes.jsonl').open('a') as f:
    f.write(json.dumps(os.environ['CODEX_HOME']) + '\\n')
with pathlib.Path('simulated-calls.jsonl').open('a') as f:
    f.write(json.dumps(sys.argv[1:]) + '\\n')
print(json.dumps({'type':'thread.started','thread_id':'SIMULATED-test-thread'}), flush=True)
with pathlib.Path('SIMULATED-progress.txt').open('a') as f:
    f.write('SIMULATED generation; no Rust or Verus check.\\n')
with pathlib.Path('madraft/src/raft/raft.rs').open('a') as f:
    f.write('// SIMULATED edit inside the experiment workspace.\\n')
if pathlib.Path('sleep.txt').exists(): time.sleep(10)
out = pathlib.Path(sys.argv[sys.argv.index('--output-last-message')+1])
out.write_text('SIMULATED agent response, not a verification result.\\n')
''')
        self.fake.chmod(0o755)

    def run_helper(self, action, *extra):
        return subprocess.run([sys.executable, str(ROOT / 'scripts/run.py'), action,
                               str(self.directory), '--codex', str(self.fake), *extra],
                              capture_output=True, text=True, env=self.env)

    def test_preview_creates_nothing_and_calls_nothing(self):
        result = self.run_helper('new')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('PREVIEW', result.stdout)
        self.assertFalse(self.directory.exists())

    def test_resume_preserves_source_conversation_and_budget(self):
        prototype = ROOT / 'madraft/src/raft/raft.rs'
        original = prototype.read_bytes()
        first = self.run_helper('new', '--execute', '--total-seconds', '20', '--model', 'SIMULATED-model', '--effort', 'medium')
        self.assertEqual(first.returncode, 0, first.stderr)
        marker = self.directory / 'madraft' / 'student-progress.txt'
        marker.write_text('SIMULATED edit retained across resume')
        statefile = self.directory / 'runs/session.json'
        before = json.loads(statefile.read_text())
        resumed = self.run_helper('resume', '--execute')
        self.assertEqual(resumed.returncode, 0, resumed.stderr)
        after = json.loads(statefile.read_text())
        calls = [json.loads(s) for s in (self.directory / 'simulated-calls.jsonl').read_text().splitlines()]
        self.assertIn('resume', calls[1])
        self.assertIn('SIMULATED-test-thread', calls[1])
        self.assertIn('SIMULATED-model', calls[1])
        self.assertIn('model_reasoning_effort="medium"', calls[1])
        self.assertEqual(after['attempts'], 2)
        self.assertGreater(after['seconds_spent'], before['seconds_spent'])
        self.assertEqual(after['total_seconds'], 20)
        self.assertTrue(marker.exists())
        self.assertEqual(prototype.read_bytes(), original)
        candidate = self.directory / 'madraft/src/raft/raft.rs'
        self.assertEqual(candidate.read_text().count('// SIMULATED edit'), 2)
        self.assertFalse((self.directory / '.git').exists())
        self.assertFalse((self.directory / 'madraft/target').exists())
        self.assertFalse((self.directory / 'README.md').exists())
        self.assertFalse((self.directory / 'docs').exists())
        self.assertFalse((self.directory / 'requirements/provenance.md').exists())
        self.assertFalse((self.directory / 'scripts/run.py').exists())
        self.assertFalse((self.directory / 'scripts/tests').exists())
        self.assertEqual({p.name for p in (self.directory / 'scripts').iterdir()},
                         {'check.py', 'tools.py', 'process.py'})
        self.assertTrue((self.directory / 'AGENTS.md').is_file())
        self.assertTrue((self.directory / 'madraft/README.md').is_file())
        self.assertIn('python3 scripts/check.py proof', (self.directory / 'AGENTS.md').read_text())

    def test_fresh_experiment_does_not_inherit_previous_edits(self):
        self.assertEqual(self.run_helper('new', '--execute').returncode, 0)
        first = self.directory
        (first / 'madraft/student-only.txt').write_text('SIMULATED prior work')
        (first / 'runs/codex-home/SIMULATED-history.txt').write_text('SIMULATED prior session')
        self.directory = first.with_name(first.name + '-fresh')
        self.addCleanup(shutil.rmtree, self.directory, ignore_errors=True)
        self.assertEqual(self.run_helper('new', '--execute').returncode, 0)
        self.assertFalse((self.directory / 'madraft/student-only.txt').exists())
        self.assertFalse((self.directory / 'runs/codex-home/SIMULATED-history.txt').exists())
        candidate = self.directory / 'madraft/src/raft/raft.rs'
        self.assertEqual(candidate.read_text().count('// SIMULATED edit'), 1)
        self.assertFalse(candidate.samefile(first / 'madraft/src/raft/raft.rs'))
        self.assertFalse(candidate.samefile(ROOT / 'madraft/src/raft/raft.rs'))

    def test_personal_instructions_excluded_and_isolated_home_reused(self):
        originals = {p: p.read_bytes() for p in self.personal_home.rglob('*') if p.is_file()}
        self.assertEqual(self.run_helper('new', '--execute').returncode, 0)
        home = self.directory / 'runs/codex-home'
        self.assertNotEqual(home, self.personal_home)
        self.assertEqual(home.stat().st_mode & 0o777, 0o700)
        self.assertEqual({p.name for p in home.iterdir()}, {'auth.json', 'rules'})
        for name in ('auth.json', 'rules'):
            self.assertTrue((home / name).samefile(self.personal_home / name))
        (home / 'SIMULATED-session.txt').write_text('SIMULATED conversation state')
        self.assertEqual(self.run_helper('resume', '--execute').returncode, 0)
        homes = [json.loads(line) for line in (self.directory / 'simulated-homes.jsonl').read_text().splitlines()]
        self.assertEqual(homes, [str(home), str(home)])
        self.assertTrue((home / 'SIMULATED-session.txt').exists())
        self.assertEqual(originals, {p: p.read_bytes() for p in self.personal_home.rglob('*') if p.is_file()})

    def test_destination_must_be_a_run_workspace(self):
        for destination in (Path(self.tmp.name) / 'outside', ROOT / 'madraft/new-run',
                            ROOT / 'runs/another-run/nested'):
            self.directory = destination
            for action in ('new', 'resume'):
                result = self.run_helper(action, '--execute')
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('direct child', result.stderr)
            self.assertFalse(destination.exists())

    def test_exhaustion_cannot_reset_on_resume(self):
        self.assertEqual(self.run_helper('new', '--execute').returncode, 0)
        statefile = self.directory / 'runs/session.json'
        state = json.loads(statefile.read_text())
        state['seconds_spent'] = state['total_seconds']
        statefile.write_text(json.dumps(state))
        result = self.run_helper('resume', '--execute')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('budget exhausted', result.stderr)
        self.assertEqual(len((self.directory / 'simulated-calls.jsonl').read_text().splitlines()), 1)

    def test_missing_tool_is_recorded_honestly(self):
        self.fake.unlink()
        result = self.run_helper('new', '--execute')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('No such file', (self.directory / 'runs/agent-1.stderr.log').read_text())
        state = json.loads((self.directory / 'runs/session.json').read_text())
        self.assertEqual(state['last_process'], 'BLOCKED')
        self.assertIsNone(state['thread_id'])

    def test_existing_directory_is_not_overwritten(self):
        self.directory.mkdir()
        result = self.run_helper('new', '--execute')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.directory / 'simulated-calls.jsonl').exists())

    def test_interrupted_run_requires_process_recovery(self):
        self.assertEqual(self.run_helper('new', '--execute').returncode, 0)
        statefile = self.directory / 'runs/session.json'
        state = json.loads(statefile.read_text())
        state['in_flight'] = True
        statefile.write_text(json.dumps(state))
        result = self.run_helper('resume', '--execute')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('stopped unexpectedly', result.stderr)

    def test_timeout_records_spending_and_preserves_raw_output(self):
        self.assertEqual(self.run_helper('new', '--execute', '--total-seconds', '0.5').returncode, 0)
        (self.directory / 'sleep.txt').touch()
        result = self.run_helper('resume', '--execute')
        self.assertNotEqual(result.returncode, 0)
        state = json.loads((self.directory / 'runs/session.json').read_text())
        self.assertEqual(state['last_process'], 'TIMEOUT')
        self.assertGreaterEqual(state['seconds_spent'], 0.5)
        self.assertIn('SIMULATED', (self.directory / 'runs/agent-2.jsonl').read_text())


class ProcessTests(unittest.TestCase):
    def test_output_limit_is_a_failure(self):
        result = run_process([sys.executable, '-c', 'print("x" * 100000)'], ROOT,
                             max_log_bytes=100, timeout=5)
        self.assertEqual(result.status, 'LOG_LIMIT')
        self.assertTrue(result.stdout_truncated)
        self.assertEqual(len(result.stdout), 100)

    def test_failure_is_not_hidden(self):
        result = run_process([sys.executable, '-c', 'raise SystemExit(7)'], ROOT)
        self.assertEqual(result.exit_code, 7)


if __name__ == '__main__':
    unittest.main()

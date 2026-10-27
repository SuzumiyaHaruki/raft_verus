"""The relocated installation is a real directory; wrappers are labeled test fixtures."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from orchestrator.config import load_config, profile_by_id
from orchestrator.toolchain import resolve
from orchestrator.verifier_runner import verify_profile
from orchestrator.workspace import create_candidate
from orchestrator.tests.real_repair_demo import contract_edit
ROOT = Path(__file__).resolve().parents[2]

class ToolchainTests(unittest.TestCase):
    def test_override_with_spaces_resolves_both_paths_and_missing_tools(self):
        with tempfile.TemporaryDirectory(prefix='raft tool override ') as location:
            with patch.dict(os.environ, RAFT_TOOLCHAIN_ROOT=location):
                config = load_config(ROOT)
            selected = config['toolchain']['resolved']
            self.assertTrue(all(p.startswith(location) for p in selected['paths'].values()))
            self.assertEqual(profile_by_id(config, 'persist-proof')['argv'][0], selected['paths']['verus'])
            self.assertEqual(profile_by_id(config, 'node-tests')['tool_env']['RAFT_TOOLCHAIN_ROOT'], location)
            result = verify_profile(ROOT, profile_by_id(config, 'persist-proof'), Path(location)/'evidence')
            self.assertEqual(result['result'], 'BLOCKED')
            self.assertIn(selected['paths']['vstd'], result['reason'])

    def test_relocated_wrapper_and_proof_select_same_installation(self):
        # Copies the small shell fixtures into a path with spaces. This tests
        # selection/argv only; real installed checks are exercised separately.
        with tempfile.TemporaryDirectory(prefix='raft relocated tools ') as location:
            base = Path(location)
            selection = resolve(dict(install_root=str(base), rust_for_verus='fixture-rust'), {})
            for name, value in selection['paths'].items():
                path = Path(value); path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('#!/bin/sh\nprintf "SIMULATED relocated '+name+'\\n"\n')
                path.chmod(0o755)
            cfg = (ROOT/'workflow.toml').read_text().replace('1.92.0-x86_64-unknown-linux-gnu', 'fixture-rust')
            config_path = base/'config with spaces.toml'; config_path.write_text(cfg)
            env = dict(os.environ, RAFT_TOOLCHAIN_ROOT=str(base), RAFT_WORKFLOW_CONFIG=str(config_path), PYTHONDONTWRITEBYTECODE='1')
            for command, expected in [('build', 'cargo'), ('persist-proof', 'verus')]:
                result = subprocess.run([sys.executable, str(ROOT/'crates/raft-verified/check.py'), command], env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn('SIMULATED relocated '+expected, result.stdout)
                self.assertIn(selection['paths'][expected], result.stderr)

    def test_real_relocated_build_and_proof_when_installed(self):
        config = load_config(ROOT)
        installed = config['toolchain']['resolved']
        if installed['missing']:
            self.skipTest('Real toolchain unavailable; no fake proof substituted')
        with tempfile.TemporaryDirectory(prefix='raft real tools with spaces ') as location:
            base = Path(location)
            original = Path(installed['environment']['RAFT_TOOLCHAIN_ROOT'])
            for name in ('verus-x86-linux', 'rustup'):
                (base/name).symlink_to(original/name, target_is_directory=True)
            env = dict(os.environ, RAFT_TOOLCHAIN_ROOT=str(base), PYTHONDONTWRITEBYTECODE='1', CARGO_TARGET_DIR=str(base/'build'))
            candidate = create_candidate(ROOT, base/'candidate')
            # SIMULATED implementation injection; the following tools really run.
            (candidate/'crates/raft-verified/src/persistence.rs').write_text(
                (ROOT/'orchestrator/tests/fixtures/solutions/persistence.rs').read_text())
            edit = contract_edit()
            (candidate/edit['path']).write_text(edit['text'])
            for command in ('build', 'persist-proof'):
                result = subprocess.run([sys.executable, str(candidate/'crates/raft-verified/check.py'), command],
                                        env=env, capture_output=True, text=True, timeout=120)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn(str(base), result.stderr)
                if command == 'persist-proof': self.assertIn('0 errors', result.stdout)

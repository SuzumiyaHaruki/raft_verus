"""Explicit experiment inputs and byte comparisons; no previous run is an input."""
from pathlib import Path
import json
import hashlib
import os
import shutil
import subprocess

HELPERS = ('check.py', 'process.py', 'tools.py', 'inputs.py')
SKIP = {'target', 'runs', '__pycache__', '.git', '.codex', '.agents', 'codex-home', '.verus-log'}
INPUT_VERSION = 'madraft-joint-proof-4'


def fingerprint(files):
    """Research input identity, not semantic approval or a substitute for checks."""
    digest = hashlib.sha256()
    for name, path in sorted(files.items()):
        body = Path(path).read_bytes()
        digest.update(name.encode() + b'\0' + str(len(body)).encode() + b'\0' + body)
    return digest.hexdigest()


def files_under(directory):
    directory = Path(directory)
    if directory.is_symlink():
        raise ValueError(f'Symlink is not an input/source directory: {directory}')
    if not directory.exists():
        return {}
    result = {}
    for parent, dirs, names in os.walk(directory, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in SKIP)
        for name in dirs + sorted(names):
            path = Path(parent) / name
            if path.is_symlink():
                raise ValueError(f'Symlink is not a candidate/input source: {path}')
            if path.is_file() and path.suffix not in ('.pyc', '.rlib', '.rmeta'):
                result[str(path.relative_to(directory))] = path
    return result


def input_files(root):
    root = Path(root)
    names = ['AGENTS.md', 'tools.ini', '.gitignore']
    names += [f'scripts/{name}' for name in HELPERS]
    names += [f'madraft/{name}' for name in ('Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md',
                                               'proof/README.md', 'proof/progress.rs')]
    names += [f'madraft/src/{name}' for name in files_under(root / 'madraft/src') if name.endswith('.rs')]
    names += [f'requirements/{p.name}' for p in sorted((root / 'requirements').iterdir())
              if p.is_file() and p.suffix in ('.md', '.rs')]
    for name in names:
        path = root / name
        if path.is_symlink() or any(p.is_symlink() for p in path.parents if p != root and root in p.parents) or not path.is_file():
            raise ValueError(f'Input must be a regular file: {path}')
    return sorted(names)


def copy_files(source, destination, names):
    for name in names:
        target = Path(destination) / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(Path(source) / name, target)


def differences(left, right, names=None):
    """Compare actual bytes, including additions/deletions, without content hashes."""
    if names is not None:
        a = {n: Path(left) / n for n in names if (Path(left) / n).is_file()}
        b = {n: Path(right) / n for n in names if (Path(right) / n).is_file()}
    else:
        a, b = files_under(left), files_under(right)
    names = set(names) if names is not None else a.keys() | b.keys()
    return sorted(name for name in names if name not in a or name not in b or a[name].is_symlink() or b[name].is_symlink()
                  or a[name].read_bytes() != b[name].read_bytes())


def protected_files(authority):
    names = [f'requirements/{name}' for name in files_under(Path(authority) / 'requirements')]
    names += [f'scripts/{name}' for name in HELPERS]
    names += ['AGENTS.md', 'tools.ini', 'madraft/Cargo.toml', 'madraft/Cargo.lock', 'madraft/LICENSE']
    names += [f'madraft/src/{name}' for name in files_under(Path(authority) / 'madraft/src')
              if Path(name).name in ('tests.rs', 'tester.rs', 'test_support.rs')]
    return names


def save_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)


def git_revision(root):
    result = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=root, capture_output=True, text=True)
    return result.stdout.strip() if result.returncode == 0 else 'UNAVAILABLE: Git metadata not exposed'

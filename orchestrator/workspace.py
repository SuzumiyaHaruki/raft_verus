"""Source manifests include untracked files, file modes, directories and links."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
from pathlib import Path, PurePosixPath

# Controller evidence and raw agent reports are never verifier source inputs.
DEFAULT_EXCLUDES = {".git", "evidence", ".agent-output", "__pycache__", "target", "Agent.md"}
# Offline tests contain complete solutions and scripted edits. None are agent input.
DEFAULT_EXCLUDES.add("orchestrator/tests")


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest_json(value) -> str:
    return digest_bytes(json.dumps(value, sort_keys=True, separators=(",", ":"),
                                   ensure_ascii=False).encode("utf-8"))


def normalize_path(value: str) -> str:
    path = PurePosixPath(value)
    if not value or path.is_absolute() or ".." in path.parts or "\\" in value:
        raise ValueError(f"unsafe relative path: {value!r}")
    normalized = path.as_posix()
    if normalized == ".":
        raise ValueError("root-wide path is not permitted")
    return normalized


def checked_path(root: Path, relative: str) -> Path:
    path = root / normalize_path(relative)
    root = root.resolve()
    try:
        path.resolve().relative_to(root)
    except ValueError as exc:
        raise ValueError(f"path escapes root: {relative}") from exc
    return path


def snapshot(root: str | Path, excludes: set[str] | None = None) -> dict:
    root = Path(root)
    omitted = DEFAULT_EXCLUDES if excludes is None else excludes
    result = {}

    def visit(directory: Path):
        for path in sorted(directory.iterdir()):
            relative = path.relative_to(root).as_posix()
            if relative in omitted:
                continue
            info = path.lstat()
            item = {"mode": stat.S_IMODE(info.st_mode)}
            if stat.S_ISLNK(info.st_mode):
                item.update(kind="symlink", target=os.readlink(path))
            elif stat.S_ISREG(info.st_mode):
                item.update(kind="file", digest=digest_bytes(path.read_bytes()), size=info.st_size)
            elif stat.S_ISDIR(info.st_mode):
                item.update(kind="directory")
            else:
                item.update(kind="special")
            result[relative] = item
            if item["kind"] == "directory":
                visit(path)
    visit(root)
    return result


def create_candidate(source: Path, destination: Path) -> Path:
    if destination.exists():
        raise FileExistsError(destination)
    # Refuse existing links: a copied link could read/write outside this workspace.
    unsafe = [p for p, item in snapshot(source).items()
              if item["kind"] in {"symlink", "special"}]
    if unsafe:
        raise ValueError(f"candidate copy refuses symlinks/special files: {unsafe}")
    def ignore(directory, names):
        prefix = Path(directory).relative_to(source)
        return {name for name in names if (prefix / name).as_posix() in DEFAULT_EXCLUDES}
    shutil.copytree(source, destination, ignore=ignore)
    return destination


def changed_paths(before: dict, after: dict) -> list[str]:
    return sorted(p for p in before.keys() | after.keys() if before.get(p) != after.get(p))


def input_manifest(root: Path, paths: list[str]) -> dict:
    all_files = snapshot(root)
    result = {}
    for path in paths:
        path = normalize_path(path)
        checked_path(root, path)
        matches = {p: entry for p, entry in all_files.items()
                   if p == path or p.startswith(path + "/")}
        if not matches:
            raise ValueError(f"missing input: {path}")
        result.update(matches)
    return result

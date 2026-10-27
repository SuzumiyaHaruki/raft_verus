"""Conservative source review; this scanner is not a Rust/Verus parser."""
from __future__ import annotations
import fnmatch
import re
from collections import Counter
from pathlib import Path
from .workspace import changed_paths, normalize_path


def matches(path: str, rules: list[str]) -> bool:
    for rule in rules:
        rule = normalize_path(rule)
        if path == rule or path.startswith(rule.rstrip("/") + "/") or fnmatch.fnmatchcase(path, rule):
            return True
    return False


def check_changes(before: dict, after: dict, task: dict, protected: list[str] | dict) -> list[dict]:
    if isinstance(protected, dict):
        protected = protected.get("paths", [])
    findings = []
    for path in changed_paths(before, after):
        reason = None
        try:
            normalize_path(path)
        except ValueError:
            reason = "unsafe path"
        if matches(path, protected):
            reason = "protected input changed"
        elif matches(path, task.get("forbidden_change_paths", [])):
            reason = "forbidden task path changed"
        elif not matches(path, task.get("permitted_change_paths", [])):
            # Creating the parent directory of an allowed output is harmless;
            # changes to existing directory permissions are still checked.
            parent_creation = (path not in before and after.get(path, {}).get("kind") == "directory"
                               and any(p.startswith(path + "/")
                                       for p in task.get("permitted_change_paths", [])))
            if not parent_creation:
                reason = "outside task modification scope"
        if after.get(path, {}).get("kind") in {"symlink", "special"}:
            reason = "symlink or special-file change refused"
        if reason:
            findings.append({"path": path, "reason": reason, "action": "REVIEW_REQUIRED"})
    return findings


def code_only(source: str) -> str:
    """Mask comments and ordinary/raw strings while retaining line positions.

    Macro expansion, generated code and dependency semantics require review.
    Rust lifetimes are retained. Character literals are conservatively retained.
    """
    pattern = re.compile(r'//[^\n]*|/\*|r(\#*)"|"', re.MULTILINE)
    output = list(source)
    cursor = 0
    while True:
        match = pattern.search(source, cursor)
        if not match:
            break
        start = match.start()
        token = match.group(0)
        end = match.end()
        if token.startswith("//"):
            pass
        elif token == "/*":
            depth = 1
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
        elif token.startswith("r"):
            delimiter = '"' + match.group(1)
            found = source.find(delimiter, end)
            end = len(source) if found < 0 else found + len(delimiter)
        else:
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
        for index in range(start, min(end, len(source))):
            if output[index] != "\n":
                output[index] = " "
        cursor = end
    return "".join(output)


TRUST_PATTERN = re.compile(
    r'\b(assume|axiom|external_body|external_fn_specification|assume_specification|'
    r'external|unsafe|todo|unimplemented|admit|cfg|cfg_attr)\b')


def scan_trust(root: Path, paths: list[str]) -> list[dict]:
    findings = []
    for relative in paths:
        path = root / relative
        if path.suffix != ".rs" or not path.is_file() or path.is_symlink():
            continue
        code = code_only(path.read_text(encoding="utf-8"))
        for match in TRUST_PATTERN.finditer(code):
            findings.append({"path": relative, "line": code.count("\n", 0, match.start()) + 1,
                             "token": match.group(), "action": "REVIEW_REQUIRED",
                             "reason": "potential trust, unfinished code or conditional coverage change",
                             "affected_theorems": "requires human mapping to obligations",
                             "alternative": "prove the fact or submit a separate reviewed TCB proposal"})
    return findings


def new_trust_findings(reference: Path, candidate: Path, paths: list[str]) -> list[dict]:
    """An unchanged reviewed construct is not a new trust proposal.

    Compare complete masked lines and multiplicities, so moving a line does not
    trigger review but adding another escape or changing its arguments does.
    Independent Verus verification still uses the reviewed whole-file command.
    """
    def signature(root, finding):
        lines = code_only((root / finding["path"]).read_text()).splitlines()
        index = finding["line"] - 1
        return finding["path"], finding["token"], "\n".join(lines[index:index + 2]).strip()
    known = Counter(signature(reference, finding) for finding in scan_trust(reference, paths))
    new = []
    for finding in scan_trust(candidate, paths):
        key = signature(candidate, finding)
        if known[key]:
            known[key] -= 1
        else:
            new.append(finding)
    return new

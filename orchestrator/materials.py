"""Read module assignments and the human-edited Markdown proof checklist."""
import re
from pathlib import Path


def module_assignment(path: Path, step: str) -> tuple[str, str]:
    """Shared module context plus exactly one `## Step: ...` section.

    Steps use level-three subheadings. Fenced examples are not section markers.
    This deliberately supports the repository's small Markdown convention only.
    """
    if not isinstance(step, str) or not step.strip() or "\n" in step:
        raise ValueError("module assignment needs a step title")
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    headings, fence = [], None
    for index, line in enumerate(lines):
        marker = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if marker:
            token = marker.group(1)
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            continue
        if fence is None and line.startswith("## "):
            headings.append((index, line[3:].strip()))
    steps = [(index, title[6:]) for index, title in headings if title.startswith("Step: ")]
    if len({title for _, title in steps}) != len(steps):
        raise ValueError(f"{path.name}: duplicate step title")
    matches = [index for index, title in steps if title == step]
    if len(matches) != 1:
        raise ValueError(f"{path.name}: missing step {step!r}")
    start = matches[0]
    end = next((index for index, _ in headings if index > start), len(lines))
    context = "".join(lines[:steps[0][0]]).strip()
    body = "".join(lines[start + 1:end]).strip()
    if not context or not body:
        raise ValueError(f"{path.name}: empty module context or step")
    return context, f"## Step: {step}\n\n{body}\n"


def table(path: Path, heading: str) -> list[dict]:
    text = path.read_text(encoding="utf-8")
    marker = "## " + heading + "\n"
    if marker not in text:
        raise ValueError(f"{path.name}: missing {heading} table")
    section = text.split(marker, 1)[1].split("\n## ", 1)[0]
    lines = [line.strip() for line in section.splitlines() if line.strip().startswith("|")]
    if len(lines) < 3:
        raise ValueError(f"{path.name}: empty {heading} table")
    cells = lambda line: [cell.strip() for cell in line.strip("|").split("|")]
    headers = cells(lines[0])
    if len(headers) != len(set(headers)):
        raise ValueError("duplicate table columns")
    rows = []
    for line in lines[2:]:
        values = cells(line)
        if len(values) != len(headers):
            raise ValueError(f"{path.name}: malformed {heading} row")
        rows.append(dict(zip(headers, values)))
    if any(not row.get("ID") for row in rows) or len({r['ID'] for r in rows}) != len(rows):
        raise ValueError(f"{path.name}: missing or duplicate IDs in {heading}")
    return rows


def items(value: str) -> list[str]:
    return [] if value == "-" else [item.strip() for item in value.split(",")]


def load_proofs(root: Path) -> tuple[list[dict], list[dict]]:
    path = root / "input/proofs.md"
    obligations = []
    for row in table(path, "Obligations"):
        required = {"ID", "Status", "Goal", "Depends", "Assumptions", "TCB", "Source", "Symbols", "Decisions"}
        if set(row) != required or any(not value for value in row.values()):
            raise ValueError("invalid obligation table columns or empty cells")
        obligations.append(dict(id=row['ID'], status=row['Status'], statement=row['Goal'],
            dependencies=items(row['Depends']), assumption_ids=items(row['Assumptions']),
            tcb_entry_ids=items(row['TCB']), code_targets=items(row['Source']),
            spec_symbols=items(row['Symbols']), blocking_decision_ids=items(row['Decisions'])))
    coverage = []
    for row in table(path, "Additional coverage"):
        if set(row) != {"ID", "Obligations", "Source", "Symbols", "Checks"} or any(not v for v in row.values()):
            raise ValueError("invalid coverage table columns or empty cells")
        coverage.append(dict(id=row['ID'], obligation_ids=items(row['Obligations']),
            source_paths=items(row['Source']), spec_symbols=items(row['Symbols']),
            check_profile_ids=items(row['Checks'])))
    return obligations, coverage

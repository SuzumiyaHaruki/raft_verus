#!/usr/bin/env python3
"""DRAFT: scripted candidate edits followed by real Rust and Verus checks.

This deliberately calls only the bundled fake Codex process. It never contacts a
model service, enables real generation, or copies a candidate back into source.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))

from orchestrator.config import load_config
from orchestrator.evidence import write_json
from orchestrator.scheduler import run_task
from orchestrator.workspace import create_candidate


def demonstrate(output: Path, task_id: str = "REPLICATION-RECONCILE") -> dict:
    output.mkdir(parents=True, exist_ok=False)
    source = create_candidate(ROOT, output / "source")
    config = load_config(source)
    config["codex"] = {
        "executable": str(ROOT / "orchestrator/tests/fixtures/fake_codex.py"),
        "supported_versions": ["codex-cli FIXTURE"],
        "fixture": True,
    }
    is_node = task_id == "STATE-INIT"
    relative = "crates/raft-verified/src/" + ("node.rs" if is_node else "reconcile.rs")
    solution = ROOT / "orchestrator/tests/fixtures/node_init_solution.rs" if is_node else source / relative
    repaired = solution.read_text(encoding="utf-8")
    marker = "term: 0, voted_for:" if is_node else "    // MATCHING_TAIL: a wholly matching request, including empty, changes nothing."
    if repaired.count(marker) != 1:
        raise ValueError("controlled mutation anchor changed; review this demo before rerunning")
    defective = repaired.replace(marker, "term: 1, voted_for:" if is_node else marker + "\n    log.truncate(previous + incoming.len());")
    config["codex"]["fixture_edits"] = [
        {"path": relative, "text": defective},
        {"path": relative, "text": repaired},
    ]
    # Exercise the existing controller loop; only the temporary demo's selection
    # is narrowed, while both independent oracles remain unchanged.
    task = config["tasks"][task_id]
    if not is_node:
        task["checks"] = ["reconcile-tests", "reconcile-proof"]
        task["obligations"] = ["LOCAL-RECONCILE"]
    task["budget"] = {"max_agent_calls": 2, "max_repair_rounds": 1,
                      "call_timeout_seconds": 300, "task_timeout_seconds": 1200}
    result = run_task(source, config, task_id, execute_codex=True,
                      fixture_mode=True)
    run = source / "evidence/runs" / result["run_id"]
    checks = [json.loads(p.read_text()) for p in sorted(run.rglob("*.check.json"))
              if "candidate" not in p.relative_to(run).parts]
    names = ("node-tests", "node-proof", "node-replay") if is_node else ("reconcile-tests", "reconcile-proof")
    observed = {name: [c for c in checks if c["profile_id"] == name] for name in names}
    correct = result.get("status") == "PASSED"
    for name, records in observed.items():
        correct = correct and len(records) == 2
        correct = correct and [c["result"] for c in records] == ["FAIL", "PASS"]
        correct = correct and all(c["evidence_kind"] == "REAL_LOCAL" for c in records)
        if name.endswith("-proof") and len(records) == 2:
            correct = correct and records[0].get("verification_summary", {}).get("errors", 0) > 0
            correct = correct and records[1].get("obligation_ids_checked") == ["LOCAL-NODE-INIT" if is_node else "LOCAL-RECONCILE"]
    if is_node:
        measurements = [c for c in checks if c["profile_id"] == "node-bench"]
        correct = correct and len(measurements) == 1 and measurements[0]["result"] == "MEASURED"
        correct = correct and "Previous attempts" in (run / "attempt-2/prompt.txt").read_text()
    summary = {
        "status": "PASS" if correct else "FAIL",
        "generation_kind": "SIMULATED",
        "check_evidence_kind": "REAL_LOCAL",
        "full_acceptance": False,
        "meaning": "Scripted " + ("wrong initial term" if is_node else "truncate-tail") + " defect rejected, scripted repair rechecked; no live Codex session.",
        "source_copy": str(source), "controller_run": str(run),
        "controller_conclusion": result,
        "checks": [{k: c.get(k) for k in ("profile_id", "result", "evidence_kind",
                    "verification_summary", "obligation_ids_checked", "evidence_paths")}
                   for c in checks],
    }
    write_json(output / "summary.json", summary)
    return summary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True,
                        help="new evidence directory; existing evidence is never overwritten")
    parser.add_argument("--task", choices=("STATE-INIT", "REPLICATION-RECONCILE"), default="REPLICATION-RECONCILE")
    args = parser.parse_args()
    summary = demonstrate(args.output.resolve(), args.task)
    print(json.dumps(summary, indent=2))
    sys.exit(0 if summary["status"] == "PASS" else 1)

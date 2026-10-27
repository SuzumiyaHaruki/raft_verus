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
from orchestrator.scheduler import run_task, run_plan
from orchestrator.workspace import create_candidate

SOLUTIONS = ROOT / "orchestrator/tests/fixtures/solutions"

def contract_edit(initial_only=False):
    """Explicit SIMULATED contract generation; never used by the real runner."""
    text = (SOLUTIONS / "contracts.rs").read_text()
    if initial_only:
        seed = (ROOT / "crates/raft-verified/src/contracts.rs").read_text()
        start = text.index("    match result {")
        body = text[start:text.index("\npub open spec fn issue_result")].rstrip()[:-1]
        text = seed.replace("{ true }", "{\n" + body + "}", 1)
        text = text.replace("use crate::{state::*, io::*};", "use crate::{state::*, io::*, initialization};")
    return dict(path="crates/raft-verified/src/contracts.rs", text=text)

def persistence_solution(redesign=False):
    text = (SOLUTIONS / "persistence.rs").read_text()
    if redesign:
        text = text.replace("self.state.", "self.core.image.")
        text = text.replace("self.core.image.serial = self.core.image.serial + 1;",
                            "self.core.image.serial = self.core.image.serial + 1;\n        self.issued = self.core.image.serial;")
    return text

def demonstrate(output: Path, task_id: str = "STATE-INIT", redesign: bool = False) -> dict:
    output.mkdir(parents=True, exist_ok=False)
    source = create_candidate(ROOT, output / "source")
    config = load_config(source)
    config["codex"].update({
        "executable": str(ROOT / "orchestrator/tests/fixtures/fake_codex.py"),
        "supported_versions": ["codex-cli FIXTURE"],
        "fixture": True,
    })
    if task_id in {"RUNTIME-PERSIST", "PROGRESS-FOUNDATION"}:
        relative = "crates/raft-verified/src/persistence.rs"
        repaired = persistence_solution(redesign)
        marker = "if !matches!(kind, CompletionKind::Persist)"
        if repaired.count(marker) != 1:
            raise ValueError("controlled wrong-kind mutation anchor changed")
        defective = repaired.replace(marker, marker + " && operation.sequence == u64::MAX")
        lemmas = "crates/raft-proofs/persistence_lemmas.rs"
        proof_repair = (SOLUTIONS / "persistence_lemmas.rs").read_text().replace("{}", "{ assert(after.serial == before.serial + 1); }", 1)
        config["codex"]["fixture_task_edits"] = {
            "STATE-INIT": [dict(files=[contract_edit(True), dict(path="crates/raft-verified/src/node.rs", text=(ROOT / "orchestrator/tests/fixtures/node_init_solution.rs").read_text())])],
            "RUNTIME-PERSIST": [dict(files=[contract_edit(), dict(path=relative, text=defective)]),
                dict(files=[dict(path=relative, text=repaired), dict(path=lemmas, text=proof_repair)])]}
        if redesign:
            layout = dict(path="crates/raft-verified/src/node_state.rs", text=(SOLUTIONS / "redesigned_state.rs").read_text())
            node = dict(path="crates/raft-verified/src/node.rs", text=(ROOT / "orchestrator/tests/fixtures/node_init_solution.rs").read_text())
            # Code and its local predicate agree, yet violate the independent initial-state requirement.
            wrong = dict(node, text=node["text"].replace("term: 0, voted_for:", "term: 1, voted_for:"))
            vacuous = dict(path="crates/raft-verified/src/contracts.rs", text=(ROOT / "crates/raft-verified/src/contracts.rs").read_text())
            config["codex"]["fixture_task_edits"]["STATE-INIT"] = [
                dict(files=[layout, wrong, vacuous]), dict(files=[node, contract_edit(True)])]
        if task_id == "PROGRESS-FOUNDATION":
            config["codex"]["fixture_task_edits"][task_id] = [dict(
                path="crates/raft-proofs/progress_lemmas.rs",
                text=(SOLUTIONS / "progress_lemmas.rs").read_text())]
        result = run_plan(source, config, [task_id], True, fixture_mode=True)
        checks = [json.loads(p.read_text()) for run_id in result["task_runs"]
                  for p in sorted((source / "evidence/runs" / run_id).glob("attempt-*/*.check.json"))]
        observed = {name: [c for c in checks if c["profile_id"] == name] for name in
                    ("persist-tests", "persist-proof", "persist-replay")}
        correct = result["status"] == "PASSED" and result["generation_kind"] == "SIMULATED"
        for name, records in observed.items():
            # The next selected proof step also reruns completed-task checks.
            expected = ["FAIL", "PASS", "PASS"] if task_id == "PROGRESS-FOUNDATION" else ["FAIL", "PASS"]
            correct &= [c["result"] for c in records] == expected
            correct &= all(c["evidence_kind"] == "REAL_LOCAL" for c in records)
            if name.endswith("proof") and records:
                correct &= records[0].get("verification_summary", {}).get("errors", 0) > 0
                stderr = Path(records[0]["evidence_paths"][1]).read_text()
                correct &= "postcondition not satisfied" in stderr and "error[E" not in stderr
        if task_id == "PROGRESS-FOUNDATION":
            progress = [c for c in checks if c["profile_id"] == "progress-proof"]
            correct &= len(progress) == 1 and progress[0]["result"] == "PASS"
        if redesign:
            init_proofs = [c for c in checks if c["profile_id"] == "node-proof"]
            correct &= bool(init_proofs) and init_proofs[0]["result"] == "FAIL"
            if init_proofs:
                functions = init_proofs[0].get("function_results", {})
                correct &= functions.get("node_init::check_node_init") is False
                correct &= any(name.endswith("Node::new") and passed for name, passed in functions.items())
        summary = dict(status="PASS" if correct else "FAIL", generation_kind="SIMULATED", check_evidence_kind="REAL_LOCAL",
            design_variant="nested storage with generated serial invariant" if redesign else "scaffold layout",
            full_acceptance=False, meaning="One scaffold candidate: scripted initialization, coordinator defect and repair, then any selected progress proof; actual independent checks.",
            source_copy=str(source), controller_conclusion=result,
            checks=[{k: c.get(k) for k in ("profile_id", "result", "evidence_kind", "verification_summary", "evidence_paths")} for c in checks])
        write_json(output / "summary.json", summary)
        return summary
    is_node = task_id == "STATE-INIT"
    relative = "crates/raft-verified/src/" + ("node.rs" if is_node else "reconcile.rs")
    solution = ROOT / "orchestrator/tests/fixtures/node_init_solution.rs" if is_node else SOLUTIONS / "reconcile.rs"
    repaired = solution.read_text(encoding="utf-8")
    marker = "term: 0, voted_for:" if is_node else "    // MATCHING_TAIL: a wholly matching request, including empty, changes nothing."
    if repaired.count(marker) != 1:
        raise ValueError("controlled mutation anchor changed; review this demo before rerunning")
    defective = repaired.replace(marker, "term: 1, voted_for:" if is_node else marker + "\n    log.truncate(previous + incoming.len());")
    config["codex"]["fixture_edits"] = [
        {"path": relative, "text": defective},
        {"path": relative, "text": repaired},
    ]
    if is_node:
        config["codex"]["fixture_edits"] = [dict(files=[contract_edit(True), edit])
                                             for edit in config["codex"]["fixture_edits"]]
    # STATE-INIT uses its full configured checks; the legacy helper demo narrows
    # selection to tests/proof. Neither demonstration changes its oracles.
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
        prompt = (run / "attempt-2/prompt.txt").read_text()
        correct = correct and all(text in prompt for text in
            ("Previous attempts", "node-tests=FAIL", "node-proof=FAIL", "attempt-1"))
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
    parser.add_argument("--task", choices=("STATE-INIT", "RUNTIME-PERSIST", "PROGRESS-FOUNDATION", "REPLICATION-RECONCILE"), default="STATE-INIT")
    parser.add_argument("--redesign", action="store_true", help="SIMULATED alternative layout, local contracts and invariant; use with a persistence/progress plan")
    args = parser.parse_args()
    if args.redesign and args.task not in {"RUNTIME-PERSIST", "PROGRESS-FOUNDATION"}:
        parser.error("--redesign requires RUNTIME-PERSIST or PROGRESS-FOUNDATION")
    summary = demonstrate(args.output.resolve(), args.task, args.redesign)
    print(json.dumps(summary, indent=2))
    sys.exit(0 if summary["status"] == "PASS" else 1)

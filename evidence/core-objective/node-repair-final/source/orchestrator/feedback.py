"""English repair context from independent observations, not agent conclusions."""
from __future__ import annotations
import json
import re
import shlex
from pathlib import Path


def classify(report: dict) -> dict:
    purpose = report.get("purpose", "proof")
    result = report.get("result")
    if result in {"ERROR", "BLOCKED", "TIMEOUT", "NOT_RUN"}:
        return {"category": "infrastructure", "action": "stop", "diagnosis":
                "Tool, configuration, resource, or protocol problem. Do not change Raft semantics; a solver timeout does not demonstrate a code bug."}
    if purpose == "benchmark":
        return {"category": "measurement", "action": "stop", "diagnosis":
                "Repeat comparable measurements and inspect noise. Missing data or a slow sample is not a demonstrated algorithm regression."}
    if purpose == "proof":
        return {"category": "proof", "action": "repair", "diagnosis":
                "The goal was not established. Possible causes include code, a missing lemma or invariant, specification mismatch, or solver resources; this is not itself a counterexample."}
    if report.get("scenario"):
        return {"category": "scenario", "action": "repair", "diagnosis":
                "Replay the saved trace and inspect states. The core, adapter, scenario driver, or expected observation may be responsible."}
    if purpose == "policy":
        return {"category": "review", "action": "review", "diagnosis":
                "A requirement or checking-policy conflict needs human review before changing the target."}
    return {"category": "functional", "action": "repair", "diagnosis":
            "Reproduce the build or test failure, preserve the regression, and repair within the declared scope."}


def excerpt(paths: list[str]) -> str:
    """Prefer diagnostic neighborhoods; retain complete raw files separately."""
    blocks = []
    for location in paths:
        path = Path(location)
        if not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        selected = set()
        for index, line in enumerate(lines):
            if re.search(r"error|fail|panic|assert|postcondition|invariant|timeout|mismatch", line, re.I):
                selected.update(range(max(0, index - 2), min(len(lines), index + 5)))
        if not selected:
            selected.update(range(min(len(lines), 12)))
        body = "\n".join(f"{index + 1}: {lines[index]}" for index in sorted(selected))[:10000]
        if re.search(r"[\u3400-\u9fff]", body):
            body = "English explanation: non-English raw diagnostics are retained at this artifact; no translation is presented as raw output."
        blocks.append(f"{path}:\n{body}")
    return "\n".join(blocks)


def repair_context(task: dict, report: dict, revision: str) -> dict:
    observation = classify(report)
    observation.update(task_goal=task["goal"], candidate_revision=revision,
        failed_command=report.get("argv", []), expected=report.get("expected_behavior", "Declared independent check succeeds"),
        observed=report.get("reason", report.get("result", "Unknown result")),
        reproduction="cd " + shlex.quote(report.get("working_directory", ".")) + " && " + shlex.join(report.get("argv", [])),
        artifact_paths=report.get("evidence_paths", []), error_excerpt=excerpt(report.get("evidence_paths", [])),
        editable_scope=task["permitted_change_paths"], required_reruns=task["check_profile_ids"])
    if report.get("scenario"):
        observation["scenario"] = report["scenario"]
    if report.get("purpose") == "proof":
        observation["proof_context"] = {"targets": report.get("expected_targets", []),
            "obligations": report.get("expected_obligation_ids", []), "sources": report.get("entrypoints", [])}
    if report.get("purpose") == "benchmark":
        observation["measurement_context"] = {"measurements": report.get("measurements", []),
            "reference": report.get("reference", "No approved reference threshold"),
            "performance_verdict": report.get("performance_verdict", "INCONCLUSIVE")}
    return observation


def render_context(context: dict) -> str:
    # Keys are small report labels, not a second author-maintained task registry.
    return "\n".join(f"{key.replace('_', ' ').capitalize()}: " +
        (value if isinstance(value, str) else json.dumps(value, ensure_ascii=True))
        for key, value in context.items())

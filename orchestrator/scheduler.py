"""Serial bounded workflow. Formal isolation is intentionally not implemented."""
from __future__ import annotations
import datetime
import difflib
import re
import time
from pathlib import Path
from .codex_runner import discover_codex, run_codex
from .config import profile_by_id
from .evidence import create_run, write_json, write_text
from .policy import check_changes, matches, scan_trust
from .schemas import read_json, validate_schema, wire_schema
from .verifier_runner import verify_profile
from .workspace import (changed_paths, checked_path, create_candidate, digest_bytes,
                        digest_json, input_manifest, normalize_path, snapshot)


def load_tasks(root: Path, config: dict) -> list[dict]:
    """Task prose comes from Markdown; execution settings come from one TOML file."""
    tasks = []
    for identifier, entry in config.get("tasks", {}).items():
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", identifier):
            raise ValueError("unsafe task ID")
        prompt = "input/" + identifier + ".md"
        tasks.append(dict(id=identifier, goal=checked_path(root, prompt).read_text(encoding="utf-8"),
            dependency_ids=entry.get("after", []),
            contract_digest=digest_json(input_manifest(root, ["input"])),
            input_paths=config.get("workflow", {}).get("context", ["input", "crates"]) + [prompt],
            permitted_change_paths=entry.get("edit", []),
            forbidden_change_paths=config["workflow"]["protected_paths"] + ["evidence", ".git", ".agent-output"],
            expected_outputs=entry.get("outputs", []), target_obligation_ids=entry.get("obligations", []),
            check_profile_ids=entry.get("checks", []), prompt_template=prompt,
            budget=dict(config.get("budget", {}))))
    return tasks


def render_prompt(root: Path, task: dict, diagnostics=None) -> str:
    """Plain English task and diagnostic feedback; no nested task JSON in prompts."""
    text = (root / "input/instructions.md").read_text(encoding="utf-8")
    text += f"\n\nTask: {task['id']}\n\n" + task['goal'] + "\n\n"
    for label, key in [("Read", "input_paths"), ("May edit", "permitted_change_paths"),
                       ("Must not edit", "forbidden_change_paths"), ("Expected outputs", "expected_outputs"),
                       ("Proof targets", "target_obligation_ids"), ("Independent checks", "check_profile_ids")]:
        text += label + ": " + ", ".join(task[key]) + "\n"
    if diagnostics:
        text += "\nActual independent feedback:\n"
        if diagnostics.get("agent_protocol"):
            protocol = diagnostics["agent_protocol"]
            text += f"Agent report protocol: {protocol['status']}: {protocol.get('reason', '')}\n"
        if diagnostics.get("missing_outputs"):
            text += "Missing outputs: " + ", ".join(diagnostics['missing_outputs']) + "\n"
        for check in diagnostics.get("checks", []):
            text += f"- {check['profile_id']}: {check['result']}: {check.get('reason', '')}\n"
            for log in check.get("evidence_paths", []):
                # Only controller-created evidence paths enter this feedback.
                body = Path(log).read_text(encoding="utf-8", errors="replace")[-12000:]
                if re.search(r"[\u3400-\u9fff]", body):
                    text += "  Non-English raw output retained for human review; not included in LLM feedback.\n"
                else:
                    text += f"\n{Path(log).name} (tail):\n{body}\n"
    return text


def task_order(tasks: list[dict]) -> list[str]:
    indexed = {task["id"]: task for task in tasks}
    if len(indexed) != len(tasks):
        raise ValueError("duplicate task ID")
    visiting, visited, ordered = set(), set(), []

    def visit(identifier):
        if identifier not in indexed:
            raise ValueError(f"unknown dependency: {identifier}")
        if identifier in visiting:
            raise ValueError(f"cyclic dependency: {identifier}")
        if identifier in visited:
            return
        visiting.add(identifier)
        for dependency in indexed[identifier]["dependency_ids"]:
            visit(dependency)
        visiting.remove(identifier)
        visited.add(identifier)
        ordered.append(identifier)
    for identifier in sorted(indexed):
        visit(identifier)
    return ordered


def validate_tasks(root: Path, tasks: list[dict], config: dict) -> list[str]:
    errors = []
    try:
        task_order(tasks)
    except (ValueError, KeyError) as exc:
        errors.append(str(exc))
    for task in tasks:
        try:
            for field in ("dependency_ids", "input_paths", "permitted_change_paths", "forbidden_change_paths",
                          "expected_outputs", "target_obligation_ids", "check_profile_ids"):
                value = task[field]
                if not isinstance(value, list) or not all(isinstance(x, str) and x for x in value):
                    raise ValueError(f"{field} must be a string list")
            if not task["permitted_change_paths"] or not task["expected_outputs"] or not task["target_obligation_ids"]:
                raise ValueError("task needs editable paths, outputs and proof targets")
            if not task["goal"].strip():
                raise ValueError("empty task Markdown")
            input_manifest(root, task["input_paths"])
            checked_path(root, task["prompt_template"])
            if not (root / task["prompt_template"]).is_file():
                raise ValueError("missing prompt template")
            for field in ("input_paths", "permitted_change_paths", "forbidden_change_paths", "expected_outputs"):
                for value in task[field]:
                    value = normalize_path(value)
                    if field != "forbidden_change_paths" and value.split("/")[0] in {".git", "evidence", ".agent-output"}:
                        raise ValueError("reserved path in task: " + value)
            protected_paths = config["workflow"]["protected_paths"]
            for output in task["expected_outputs"]:
                if (not matches(output, task["permitted_change_paths"])
                        or matches(output, task["forbidden_change_paths"])
                        or matches(output, protected_paths)):
                    raise ValueError("expected output is outside permitted editable paths: " + output)
            for identifier in task["check_profile_ids"]:
                profile_by_id(config, identifier)
            if not task["check_profile_ids"]:
                raise ValueError("task has no independent checks")
            if len(set(task["check_profile_ids"])) != len(task["check_profile_ids"]):
                raise ValueError("duplicate task check profile")
        except (ValueError, KeyError, OSError) as exc:
            errors.append(f"{task.get('id', '<missing-id>')}: {exc}")
    return errors


def approval_bindings(root: Path, config: dict) -> dict:
    protected = config["workflow"]["protected_paths"]
    protected_manifest = input_manifest(root, protected)
    policy = input_manifest(root, ["orchestrator"])
    policy["effective_config"] = {k: v for k, v in config.items() if not k.startswith("_")}
    return {"contract_digest": digest_json(input_manifest(root, ["input"])),
            "protected_manifest_digest": digest_json(protected_manifest),
            "checker_policy_digest": digest_json(policy),
            "toolchain_manifest_digest": digest_json(config.get("toolchain", {})),
            "trust_baseline_digest": digest_bytes((root / "input/trust.md").read_bytes())}


def validate_approval(root: Path, config: dict, task: dict, approval_file: str | Path | None) -> dict:
    if approval_file is None:
        raise ValueError("external human approval is required")
    location = Path(approval_file).resolve()
    if location.is_relative_to(root.resolve()):
        raise ValueError("approval must be outside the agent workspace")
    approval = read_json(location)
    validate_schema(approval, wire_schema("approval"))
    if approval.get("is_template") or approval.get("lifecycle_status") != "HUMAN_APPROVED":
        raise ValueError("example/draft approval is not valid approval")
    if any(re.search(r"EXAMPLE|TEMPLATE|NOT.APPROVED|PLACEHOLDER", str(approval.get(key, "")), re.I)
           for key in ("approval_id", "approved_by", "approved_at")):
        raise ValueError("example approval values refused")
    try:
        approved_at = datetime.datetime.fromisoformat(approval["approved_at"].replace("Z", "+00:00"))
        if approved_at.tzinfo is None:
            raise ValueError("approval time requires a timezone")
    except ValueError as exc:
        raise ValueError("invalid approval timestamp") from exc
    bindings = approval_bindings(root, config)
    for key, value in bindings.items():
        if not re.fullmatch(r"[0-9a-f]{64}", approval[key]) or approval[key] != value:
            raise ValueError(f"approval digest mismatch: {key}")
    if task["contract_digest"] != bindings["contract_digest"]:
        raise ValueError("task is not bound to the approved contract")
    if task["id"] not in approval["approved_task_scope"]:
        raise ValueError("task outside approved scope")
    return approval


def task_fingerprint(root: Path, config: dict, task: dict) -> str:
    return digest_json({"task": task, "inputs": input_manifest(root, task["input_paths"]),
                        "bindings": approval_bindings(root, config)})


def dependency_ready(root: Path, config: dict, task_id: str) -> bool:
    tasks = {task["id"]: task for task in load_tasks(root, config)}
    if task_id not in tasks:
        return False
    current = task_fingerprint(root, config, tasks[task_id])
    for path in sorted((root / "evidence/runs").glob("*/conclusion.json"), reverse=True):
        record = read_json(path)
        if record.get("task_id") != task_id:
            continue
        # Only locally checked real evidence can satisfy a real future dependency.
        if (record.get("status") == "PASSED" and record.get("evidence_kind") == "REAL_LOCAL"
                and record.get("current_fingerprint") == current):
            return True
    return False


def plan(root: Path, config: dict) -> dict:
    tasks = load_tasks(root, config)
    errors = validate_tasks(root, tasks, config)
    if errors:
        return {"status": "ERROR", "errors": errors}
    indexed = {task["id"]: task for task in tasks}
    return {"status": "DRAFT_NOT_APPROVED", "approval_bindings_candidate": approval_bindings(root, config),
            "tasks": [{"id": identifier, "dependency_ids": indexed[identifier]["dependency_ids"],
                       "input_fingerprint": task_fingerprint(root, config, indexed[identifier]),
                       "permitted_change_paths": indexed[identifier]["permitted_change_paths"],
                       "check_profile_ids": indexed[identifier]["check_profile_ids"], "executable": False}
                      for identifier in task_order(tasks)]}


def _save_patch(source: Path, candidate: Path, changes: list[str], directory: Path) -> None:
    patches = []
    for relative in changes:
        old, new = source / relative, candidate / relative
        try:
            before = old.read_text().splitlines(True) if old.is_file() and not old.is_symlink() else []
            after = new.read_text().splitlines(True) if new.is_file() and not new.is_symlink() else []
        except (UnicodeError, OSError):
            continue
        patches.extend(difflib.unified_diff(before, after, fromfile="a/" + relative, tofile="b/" + relative))
    write_text(directory / "candidate.patch", "".join(patches))


def run_task(root: Path, config: dict, task_id: str, execute_codex: bool = False,
             approval_file: str | Path | None = None, cancel_event=None) -> dict:
    task_started = time.monotonic()
    root = Path(root).resolve()
    tasks = load_tasks(root, config)
    errors = validate_tasks(root, tasks, config)
    if errors:
        return {"status": "BLOCKED", "reason": "invalid task definitions", "errors": errors}
    task = next((task for task in tasks if task["id"] == task_id), None)
    if task is None:
        return {"status": "BLOCKED", "reason": "unknown task ID"}
    if not execute_codex:
        return {"status": "DRY_RUN", "task_id": task_id, "agent_calls": 0,
                "input_fingerprint": task_fingerprint(root, config, task),
                "check_profile_ids": task["check_profile_ids"], "approval": "NOT_CHECKED_NOT_EXECUTABLE"}
    run_id, directory = create_run(root, "task")
    conclusion = {"run_id": run_id, "task_id": task_id, "status": "PENDING", "agent_calls": 0,
                  "history": ["PENDING"], "evidence_kind": "REAL_LOCAL", "full_acceptance": False,
                  "input_fingerprint": task_fingerprint(root, config, task), "current_fingerprint": None}
    write_json(directory / "task.json", task)
    write_json(directory / "input_manifest.json", input_manifest(root, task["input_paths"]))
    bindings = approval_bindings(root, config)
    write_json(directory / "approval_bindings.json", bindings)

    def transition(status):
        conclusion["status"] = status
        conclusion["history"].append(status)

    def finish(status, reason):
        transition(status)
        conclusion["reason"] = reason
        write_json(directory / "conclusion.json", conclusion)
        return conclusion

    settings = config.get("workflow", {})
    try:
        validate_approval(root, config, task, approval_file)
    except ValueError as exc:
        return finish("BLOCKED", str(exc))
    if settings.get("isolation", "local-development") != "local-development":
        return finish("BLOCKED", "formal agent/checker OS isolation is not implemented")
    if not settings.get("allow_real_codex", False):
        return finish("BLOCKED", "bootstrap configuration disables real Codex execution")
    if any(not dependency_ready(root, config, dep) for dep in task["dependency_ids"]):
        return finish("BLOCKED", "dependency lacks current matching real evidence")
    budget = dict(config.get("budget", {}))
    for key, value in task["budget"].items():
        budget[key] = min(value, budget.get(key, value))
    if budget.get("max_agent_calls", 0) <= 0 or budget.get("task_timeout_seconds", 0) <= 0:
        return finish("BLOCKED", "task budget exhausted")
    deadline = task_started + budget["task_timeout_seconds"]
    if deadline <= time.monotonic():
        return finish("FAILED", "whole-task deadline expired during preparation")
    transition("READY")
    tool_info = discover_codex(config.get("codex", {}), root, directory, deadline)
    write_json(directory / "tool_versions.json", tool_info)
    if tool_info["status"] != "READY":
        return finish("BLOCKED", tool_info["reason"])
    try:
        candidate = create_candidate(root, directory / "candidate")
    except (OSError, ValueError) as exc:
        return finish("BLOCKED", str(exc))
    conclusion["candidate_path"] = str(candidate)
    # Unlike repository manifests this includes every candidate path except the
    # fixed raw report directory; run_codex separately validates that directory.
    before = snapshot(candidate, excludes={".agent-output"})
    write_json(directory / "candidate_before.json", before)
    protected = config["workflow"]["protected_paths"]
    previous_failure = None
    diagnostics = None
    attempts = min(budget["max_agent_calls"], budget.get("max_repair_rounds", 0) + 1)
    for attempt in range(attempts):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return finish("FAILED", "whole-task time budget exhausted")
        if cancel_event is not None and cancel_event.is_set():
            return finish("CANCELLED", "task cancelled")
        if approval_bindings(root, config) != bindings:
            return finish("REVIEW_REQUIRED", "approved source inputs changed during task")
        iteration = directory / f"attempt-{attempt + 1}"
        iteration.mkdir(mode=0o700)
        prompt = render_prompt(root, task, diagnostics)
        write_text(iteration / "prompt.txt", prompt)
        runner_config = dict(config.get("codex", {}), call_timeout_seconds=min(budget["call_timeout_seconds"], remaining),
                             max_log_bytes=budget["max_log_bytes"])
        report_path = candidate / ".agent-output/agent_report.json"
        if attempt and report_path.is_file() and not report_path.is_symlink():
            report_path.unlink()
        transition("RUNNING")
        conclusion["agent_calls"] += 1
        generated = run_codex(runner_config, candidate, prompt,
                              wire_schema("agent_report"), iteration, cancel_event)
        if generated["status"] == "CANCELLED":
            return finish("CANCELLED", "agent process cancelled")
        if generated["status"] in {"BLOCKED", "TIMEOUT", "LOG_LIMIT", "CLI_ERROR"}:
            return finish("BLOCKED" if generated["status"] == "BLOCKED" else "FAILED", generated["status"])
        transition("CHECKING")
        after = snapshot(candidate, excludes={".agent-output"})
        changes = changed_paths(before, after)
        findings = check_changes(before, after, task, protected) + scan_trust(candidate, changes)
        raw_dir = candidate / ".agent-output"
        if raw_dir.is_symlink() or (raw_dir.exists() and {p.name for p in raw_dir.iterdir()} != {"agent_report.json"}):
            findings.append({"path": ".agent-output", "reason": "unexpected raw-report directory contents"})
        write_json(iteration / "candidate_manifest.json", after)
        write_json(iteration / "changed_paths.json", changes)
        write_json(iteration / "policy_findings.json", findings)
        _save_patch(root, candidate, changes, iteration)
        if findings:
            return finish("REVIEW_REQUIRED", "candidate violates modification or trust policy")
        report = generated.get("agent_report")
        if report and report["task_id"] != task_id:
            return finish("FAILED", "agent report belongs to another task")
        if report and (report["proposed_spec_changes"] or report["proposed_trust_changes"]):
            return finish("REVIEW_REQUIRED", "agent proposed specification or trust changes")
        checks = []
        if generated["status"] == "CANDIDATE":
            missing = [p for p in task["expected_outputs"] if not checked_path(candidate, p).exists()]
            if missing:
                diagnostics = {"missing_outputs": missing}
            else:
                for identifier in task["check_profile_ids"]:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        return finish("FAILED", "whole-task time budget exhausted before checking")
                    profile = dict(profile_by_id(config, identifier))
                    profile["timeout_seconds"] = min(profile.get("timeout_seconds", 120), remaining)
                    checks.append(verify_profile(candidate, profile, iteration, run_id, cancel_event))
                diagnostics = {"checks": checks}
                if any(check["result"] == "TYPECHECKED" for check in checks):
                    return finish("BLOCKED", "typechecking does not discharge task proof obligations")
                if cancel_event is not None and cancel_event.is_set():
                    return finish("CANCELLED", "task cancelled during independent checking")
                if any("CANCELLED" in check.get("reason", "") for check in checks):
                    return finish("CANCELLED", "checker process cancelled")
                if snapshot(candidate, excludes={".agent-output"}) != after:
                    return finish("REVIEW_REQUIRED", "candidate changed across independent checks")
                if all(check["result"] == "PASS" for check in checks):
                    if approval_bindings(root, config) != bindings:
                        return finish("REVIEW_REQUIRED", "approved inputs changed during checking")
                    simulated = any(check["evidence_kind"] == "SIMULATED" for check in checks)
                    checked_obligations = {obligation for check in checks for obligation in check["obligation_ids_checked"]}
                    if not simulated and not set(task["target_obligation_ids"]).issubset(checked_obligations):
                        return finish("BLOCKED", "independent profile coverage does not cover task obligations")
                    conclusion["current_fingerprint"] = task_fingerprint(candidate, config, task)
                    if any(check["evidence_kind"] == "SIMULATED" for check in checks):
                        conclusion["evidence_kind"] = "SIMULATED"
                    return finish("PASSED", "local task checks passed; candidate awaits human integration")
                if any(check["result"] in {"BLOCKED", "TIMEOUT", "ERROR"} for check in checks):
                    return finish("BLOCKED", "checker unavailable, uncertain, or protocol error")
        else:
            diagnostics = {"agent_protocol": generated}
        # Ignore run timestamps when identifying repeated failures.
        signature = digest_json({"candidate": after, "results": [(c["profile_id"], c["result"], c["reason"]) for c in checks],
                                 "protocol": generated["status"]})
        if signature == previous_failure:
            return finish("FAILED", "repeated identical failure without a candidate change")
        previous_failure = signature
        if attempt + 1 < attempts:
            transition("NEEDS_REPAIR")
    return finish("FAILED", "bounded agent/repair budget exhausted")

"""Serial bounded workflow. Formal isolation is intentionally not implemented."""
from __future__ import annotations
import difflib
import json
import re
import shlex
import time
from pathlib import Path
from .codex_runner import discover_codex, run_codex
from .config import profile_by_id
from .evidence import create_run, write_json, write_text
from .feedback import classify, repair_context, render_context
from .materials import module_assignment
from .policy import check_changes, matches, new_trust_findings
from .schemas import read_json, wire_schema
from .verifier_runner import verify_profile
from .workspace import (changed_paths, checked_path, create_candidate, digest_bytes,
                        digest_json, input_manifest, normalize_path, snapshot)


def load_tasks(root: Path, config: dict) -> list[dict]:
    """Task prose comes from Markdown; execution settings come from one TOML file."""
    tasks = []
    for identifier, entry in config.get("tasks", {}).items():
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", identifier):
            raise ValueError("unsafe task ID")
        module = entry.get("module")
        step = entry.get("step")
        context = ""
        if module is not None:
            if not isinstance(module, str) or not module.startswith("input/") or not module.endswith(".md"):
                raise ValueError("module must name an input Markdown file")
            prompt = module
            context, goal = module_assignment(checked_path(root, prompt), step)
        else:
            if step is not None:
                raise ValueError("step requires a module document")
            # Preserve compatibility with existing standalone task fixtures.
            prompt = "input/" + identifier + ".md"
            goal = checked_path(root, prompt).read_text(encoding="utf-8")
        kind = entry.get("kind", "implementation")
        budget = dict(config.get("budget", {}))
        budget.update(config.get("budgets", {}).get(kind, {}))
        budget.update(entry.get("budget", {}))
        tasks.append(dict(id=identifier, goal=goal, module_document=module,
            step_title=step, module_context=context,
            kind=kind, status=entry.get("status", "ready"), blocked_reason=entry.get("blocked_reason", ""),
            dependency_ids=entry.get("after", []),
            contract_digest=digest_json(input_manifest(root, ["input"])),
            input_paths=list(dict.fromkeys(entry.get("context", config.get("workflow", {}).get("context", ["input", "crates"])) + [prompt])),
            permitted_change_paths=entry.get("edit", []),
            forbidden_change_paths=config["workflow"]["protected_paths"] + ["evidence", ".git", ".agent-output"],
            expected_outputs=entry.get("outputs", []), target_obligation_ids=entry.get("obligations", []),
            check_profile_ids=entry.get("checks", []), prompt_template=prompt,
            budget=budget))
    return tasks


def render_prompt(root: Path, task: dict, diagnostics=None) -> str:
    """Plain English task and diagnostic feedback; no nested task JSON in prompts."""
    text = (root / "input/instructions.md").read_text(encoding="utf-8")
    if task.get("module_document"):
        text += (f"\n\nModule context: {task['module_document']}\n\n{task['module_context']}\n\n"
                 "Execute only the selected step below within its editable scope. "
                 "The module responsibilities describe the destination, not extra work for this call. "
                 "Passing this step does not complete the module or the Raft project.\n")
    text += f"\n\nTask: {task['id']}\n\n" + task['goal'] + "\n\n"
    text += f"Task stage: {task['status']}\n"
    if task.get("blocked_reason"):
        text += "Missing prerequisites: " + task['blocked_reason'] + "\n"
    for label, key in [("Read", "input_paths"), ("May edit", "permitted_change_paths"),
                       ("Must not edit", "forbidden_change_paths"), ("Expected outputs", "expected_outputs"),
                       ("Proof targets", "target_obligation_ids"), ("Independent checks", "check_profile_ids")]:
        text += label + ": " + ", ".join(task[key]) + "\n"
    if diagnostics:
        text += "\nPrevious attempts (agent explanations are claims, not evidence):\n"
        for item in diagnostics.get("attempts", []):
            text += (f"- Attempt {item['number']}: {item['summary']}\n"
                     f"  Changed: {', '.join(item['changed_paths']) or '(none)'}\n"
                     f"  Independent results: {item['results']}\n"
                     f"  Saved patch and raw logs: {item['artifacts']}\n")
        text += "\nActual independent feedback:\n"
        if diagnostics.get("agent_protocol"):
            protocol = diagnostics["agent_protocol"]
            text += f"Agent report protocol: {protocol['status']}: {protocol.get('reason', '')}\n"
        if diagnostics.get("missing_outputs"):
            text += "Missing outputs: " + ", ".join(diagnostics['missing_outputs']) + "\n"
        for check in diagnostics.get("checks", []):
            text += f"- {check['profile_id']}: {check['result']}: {check.get('reason', '')}\n"
            if check["result"] not in {"PASS", "TYPECHECKED", "MEASURED"}:
                text += render_context(repair_context(task, check, diagnostics.get("candidate_revision", "unrecorded"))) + "\n"
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
            if task.get("status") not in {None, "ready", "planned"}:
                raise ValueError("task status must be ready or planned")
            planned = task.get("status") == "planned"
            if planned and not task.get("blocked_reason"):
                raise ValueError("planned task must name its missing prerequisites")
            if not planned and (not task["permitted_change_paths"] or not task["expected_outputs"]):
                raise ValueError("ready task needs editable paths and outputs")
            if not task["goal"].strip():
                raise ValueError("empty task Markdown")
            if not planned:
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
            if not planned and not task["check_profile_ids"]:
                raise ValueError("task has no independent checks")
            if len(set(task["check_profile_ids"])) != len(task["check_profile_ids"]):
                raise ValueError("duplicate task check profile")
        except (ValueError, KeyError, OSError) as exc:
            errors.append(f"{task.get('id', '<missing-id>')}: {exc}")
    return errors


def input_bindings(root: Path, config: dict) -> dict:
    """Existing evidence bindings detect source/checker changes during a run."""
    protected = config["workflow"]["protected_paths"]
    protected_manifest = input_manifest(root, protected)
    policy = input_manifest(root, ["orchestrator"])
    policy["effective_config"] = {k: v for k, v in config.items() if not k.startswith("_")}
    return {"contract_digest": digest_json(input_manifest(root, ["input"])),
            "protected_manifest_digest": digest_json(protected_manifest),
            "checker_policy_digest": digest_json(policy),
            "toolchain_manifest_digest": digest_json(config.get("toolchain", {})),
            "trust_baseline_digest": digest_bytes((root / "input/trust.md").read_bytes())}


def task_fingerprint(root: Path, config: dict, task: dict) -> str:
    # Reuse of check evidence depends on this task and its actual inputs,
    # not unrelated prose or a human review record.
    relevant = {key: value for key, value in task.items() if key != "contract_digest"}
    paths = list(dict.fromkeys(task["input_paths"] +
        [p for p in task["expected_outputs"] if checked_path(root, p).exists()] + ["orchestrator"]))
    for identifier in task["check_profile_ids"]:
        profile = profile_by_id(config, identifier)
        paths.extend(profile.get("source_inputs", profile.get("entrypoints", [])))
        paths.extend(profile.get("proof_entrypoints", []))
    return digest_json({"task": relevant, "inputs": input_manifest(root, paths),
        "checks": [profile_by_id(config, key) for key in task["check_profile_ids"]],
        "toolchain": config.get("toolchain", {})})


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
    return {"status": "DRAFT", "completion_scope": "SELECTED_STEPS_ONLY",
            "module_completion": "NOT_ASSESSED", "full_acceptance": False,
            "codex_request": codex_request(config),
            "input_bindings": input_bindings(root, config),
            "tasks": [{"id": identifier, "dependency_ids": indexed[identifier]["dependency_ids"],
                       "module_document": indexed[identifier]["module_document"],
                       "step_title": indexed[identifier]["step_title"],
                       "input_fingerprint": task_fingerprint(root, config, indexed[identifier]) if indexed[identifier]["status"] == "ready" else None,
                       "stage": indexed[identifier]["status"], "blocked_reason": indexed[identifier]["blocked_reason"],
                       "permitted_change_paths": indexed[identifier]["permitted_change_paths"],
                       "check_profile_ids": indexed[identifier]["check_profile_ids"], "executable": False}
                      for identifier in task_order(tasks)]}


def format_plan(result: dict) -> str:
    """A readable module/step view; JSON remains available to existing callers."""
    if result.get("errors"):
        return "Invalid plan:\n" + "\n".join(result["errors"])
    lines = ["DRAFT Raft development plan", "Step readiness is not module completion. Use an explicit launch to execute ready steps.",
             "Module completion: NOT_ASSESSED. Whole-project acceptance: incomplete."]
    request = result["codex_request"]
    lines.append(f"Requested model: {request['model'] or 'Codex default'}; reasoning effort: "
                 f"{request['model_reasoning_effort'] or 'Codex default'} (not a resolved service result).")
    modules = dict.fromkeys(task["module_document"] for task in result["tasks"])
    for module in modules:
        lines.append("\n" + (module or "Standalone assignments"))
        for task in result["tasks"]:
            if task["module_document"] != module:
                continue
            lines.append(f"  [{task['stage']}] {task['id']}: {task['step_title'] or task['id']}")
            if task["dependency_ids"]:
                lines.append("    After: " + ", ".join(task["dependency_ids"]))
            if task["blocked_reason"]:
                lines.append("    Missing: " + task["blocked_reason"])
    return "\n".join(lines)


def codex_request(config: dict) -> dict:
    return {key: config.get("codex", {}).get(key) for key in ("model", "model_reasoning_effort")}


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


def execution_blocker(config: dict, fixture_mode: bool) -> str | None:
    """Preserve existing execution settings; human code review is after delivery."""
    if fixture_mode:
        fixture = Path(__file__).parent / "tests/fixtures/fake_codex.py"
        if (not config.get("codex", {}).get("fixture")
                or Path(config["codex"].get("executable", "")).resolve() != fixture.resolve()):
            return "fixture mode permits only the bundled offline fake Codex executable"
    settings = config.get("workflow", {})
    if settings.get("isolation", "local-development") != "local-development":
        return "formal agent/checker OS isolation is not implemented"
    if not fixture_mode and not settings.get("allow_real_codex", False):
        return "configuration disables real Codex execution"
    return None


def generation_kind(config: dict, fixture_mode: bool) -> str:
    fixture = Path(__file__).parent / "tests/fixtures/fake_codex.py"
    simulated = (fixture_mode or config.get("codex", {}).get("fixture")
                 or Path(config.get("codex", {}).get("executable", "codex")).resolve() == fixture.resolve())
    return "SIMULATED" if simulated else "REAL_CODEX"


def run_task(root: Path, config: dict, task_id: str, execute_codex: bool = False,
             cancel_event=None, *,
             fixture_mode: bool = False, candidate_path: Path | None = None,
             completed: dict | None = None) -> dict:
    task_started = time.monotonic()
    root = Path(root).resolve()
    tasks = load_tasks(root, config)
    errors = validate_tasks(root, tasks, config)
    if errors:
        return {"status": "BLOCKED", "reason": "invalid task definitions", "errors": errors}
    task = next((task for task in tasks if task["id"] == task_id), None)
    if task is None:
        return {"status": "BLOCKED", "reason": "unknown task ID"}
    if task["status"] == "planned":
        return {"status": "BLOCKED", "task_id": task_id, "agent_calls": 0, "reason": task["blocked_reason"]}
    if not execute_codex:
        return {"status": "DRY_RUN", "task_id": task_id, "agent_calls": 0,
                "completion_scope": "STEP_ONLY", "module_document": task["module_document"],
                "codex_request": codex_request(config),
                "input_fingerprint": task_fingerprint(root, config, task),
                "check_profile_ids": task["check_profile_ids"]}
    run_id, directory = create_run(root, "task")
    conclusion = {"run_id": run_id, "task_id": task_id, "status": "PENDING", "agent_calls": 0,
                  "completion_scope": "STEP_ONLY", "module_document": task["module_document"],
                  "codex_request": codex_request(config),
                  "history": ["PENDING"], "evidence_kind": "REAL_LOCAL", "full_acceptance": False,
                  "generation_kind": generation_kind(config, fixture_mode),
                  "obligation_ids_checked": [],
                  "input_fingerprint": task_fingerprint(root, config, task), "current_fingerprint": None}
    write_json(directory / "task.json", task)
    write_json(directory / "input_manifest.json", input_manifest(root, task["input_paths"]))
    bindings = input_bindings(root, config)
    write_json(directory / "input_bindings.json", bindings)

    def transition(status):
        conclusion["status"] = status
        conclusion["history"].append(status)

    def finish(status, reason):
        transition(status)
        conclusion["reason"] = reason
        write_json(directory / "conclusion.json", conclusion)
        return conclusion

    blocker = execution_blocker(config, fixture_mode)
    if blocker:
        return finish("BLOCKED", blocker)
    settings = config["workflow"]
    def ready(dependency):
        if completed is None:
            return dependency_ready(root, config, dependency)
        record = completed.get(dependency, {})
        dependency_task = next(t for t in tasks if t["id"] == dependency)
        return (record.get("status") == "PASSED" and record.get("evidence_kind") == "REAL_LOCAL"
                and record.get("current_fingerprint") == task_fingerprint(candidate_path, config, dependency_task))
    if any(not ready(dep) for dep in task["dependency_ids"]):
        return finish("BLOCKED", "dependency lacks current matching real evidence")
    budget = task["budget"]
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
        candidate = candidate_path if candidate_path is not None else create_candidate(root, directory / "candidate")
        if candidate.is_symlink() or not candidate.resolve().is_relative_to((root / "evidence/runs").resolve()):
            raise ValueError("candidate must be a controller-owned evidence workspace")
    except (OSError, ValueError) as exc:
        return finish("BLOCKED", str(exc))
    conclusion["candidate_path"] = str(candidate)
    if input_manifest(candidate, settings["protected_paths"]) != input_manifest(root, settings["protected_paths"]):
        return finish("REVIEW_REQUIRED", "candidate protected inputs differ from the source checkout")
    # Unlike repository manifests this includes every candidate path except the
    # fixed raw report directory; run_codex separately validates that directory.
    before = snapshot(candidate, excludes={".agent-output"})
    write_json(directory / "candidate_before.json", before)
    protected = config["workflow"]["protected_paths"]
    previous_failure = None
    diagnostics = None
    attempt_history = []
    attempts = min(budget["max_agent_calls"], budget.get("max_repair_rounds", 0) + 1)
    for attempt in range(attempts):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return finish("FAILED", "whole-task time budget exhausted")
        if cancel_event is not None and cancel_event.is_set():
            return finish("CANCELLED", "task cancelled")
        if input_bindings(root, config) != bindings:
            return finish("REVIEW_REQUIRED", "source/checker inputs changed during task")
        iteration = directory / f"attempt-{attempt + 1}"
        iteration.mkdir(mode=0o700)
        prompt = render_prompt(root, task, diagnostics)
        build_path = (iteration / "agent-build").resolve()
        prompt += ("\nFor local Rust or check.py commands, use the explicit environment prefix "
                   + "CARGO_TARGET_DIR=" + shlex.quote(str(build_path)) + " PYTHONDONTWRITEBYTECODE=1 "
                   + "before the command. This controller-owned scratch directory is writable through --add-dir; "
                   + "the prefix also works when the Codex shell filters inherited environment variables.\n")
        if fixture_mode:
            edits = config["codex"].get("fixture_edits", [])
            options = {"task_id": task_id}
            if edits:
                edit = edits[min(attempt, len(edits) - 1)]
                options.update(write_path=edit["path"], write_text=edit["text"])
            prompt += "\nFIXTURE_JSON=" + json.dumps(options) + "\n"
        write_text(iteration / "prompt.txt", prompt)
        runner_config = dict(config.get("codex", {}), call_timeout_seconds=min(budget["call_timeout_seconds"], remaining),
                             max_log_bytes=budget["max_log_bytes"])
        report_path = candidate / ".agent-output/agent_report.json"
        if report_path.is_file() and not report_path.is_symlink():
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
        findings = check_changes(before, after, task, protected) + new_trust_findings(root, candidate, changes)
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
                    if profile.get("purpose") == "benchmark" and any(
                            check.get("required", True) and check["result"] not in {"PASS", "TYPECHECKED", "MEASURED"}
                            for check in checks):
                        conclusion.setdefault("deferred_checks", []).append({"profile_id": identifier, "attempt": attempt + 1,
                            "reason": "required correctness check failed; repair before measuring performance"})
                        continue
                    profile["timeout_seconds"] = min(profile.get("timeout_seconds", 120), remaining)
                    checks.append(verify_profile(candidate, profile, iteration, run_id, cancel_event))
                diagnostics = {"checks": checks, "candidate_revision": digest_json(after)}
                conclusion["check_evidence_kind"] = "SIMULATED" if any(c["evidence_kind"] == "SIMULATED" for c in checks) else "REAL_LOCAL"
                conclusion["evidence_kind"] = conclusion["check_evidence_kind"]
                write_json(iteration / "feedback.json", [repair_context(task, check, digest_json(after)) for check in checks
                           if check["result"] not in {"PASS", "TYPECHECKED", "MEASURED"}])
                if cancel_event is not None and cancel_event.is_set():
                    return finish("CANCELLED", "task cancelled during independent checking")
                if any("CANCELLED" in check.get("reason", "") for check in checks):
                    return finish("CANCELLED", "checker process cancelled")
                if snapshot(candidate, excludes={".agent-output"}) != after:
                    return finish("REVIEW_REQUIRED", "candidate changed across independent checks")
                required = [check for check in checks if check.get("required", True)]
                if required and all(check["result"] in {"PASS", "TYPECHECKED", "MEASURED"} for check in required):
                    if input_bindings(root, config) != bindings:
                        return finish("REVIEW_REQUIRED", "source/checker inputs changed during checking")
                    checked_obligations = {obligation for check in checks if check["evidence_kind"] == "REAL_LOCAL"
                                           for obligation in check["obligation_ids_checked"]}
                    if not set(task["target_obligation_ids"]).issubset(checked_obligations):
                        return finish("BLOCKED", "independent profile coverage does not cover task obligations")
                    conclusion["obligation_ids_checked"] = sorted(checked_obligations)
                    conclusion["current_fingerprint"] = task_fingerprint(candidate, config, task)
                    if any(check["evidence_kind"] == "SIMULATED" for check in checks):
                        conclusion["evidence_kind"] = "SIMULATED"
                    conclusion["check_evidence_kind"] = conclusion["evidence_kind"]
                    return finish("PASSED", "local task checks passed; candidate awaits human integration")
                if any(check["result"] in {"BLOCKED", "TIMEOUT", "ERROR"} for check in checks):
                    return finish("BLOCKED", "checker unavailable, uncertain, or protocol error")
                if any(classify(check)["action"] == "review" for check in required if check["result"] != "PASS"):
                    return finish("REVIEW_REQUIRED", "checking policy or model conflict requires review")
                if any(classify(check)["action"] == "stop" for check in required if check["result"] not in {"PASS", "TYPECHECKED", "MEASURED"}):
                    return finish("BLOCKED", "measurement or infrastructure requires investigation, not automatic semantic repair")
        else:
            diagnostics = {"agent_protocol": generated}
        summary = (report or {}).get("summary", "No valid agent explanation")[:1200]
        if re.search(r"[\u3400-\u9fff]", summary):
            summary = "Non-English agent explanation retained in the raw report; omitted from prompt."
        attempt_history.append(dict(number=attempt + 1, summary=summary, changed_paths=changes,
            results=", ".join(f"{c['profile_id']}={c['result']}" for c in checks) or generated["status"],
            artifacts=str(iteration)))
        diagnostics["attempts"] = list(attempt_history)
        conclusion["failed_attempts"] = list(attempt_history)
        # Ignore run timestamps when identifying repeated failures.
        signature = digest_json({"candidate": after, "results": [(c["profile_id"], c["result"], c["reason"]) for c in checks],
                                 "protocol": generated["status"]})
        if signature == previous_failure:
            return finish("FAILED", "repeated identical failure without a candidate change")
        previous_failure = signature
        if attempt + 1 < attempts:
            transition("NEEDS_REPAIR")
    return finish("FAILED", "bounded agent/repair budget exhausted")


def _save_plan_state(directory: Path, state: dict) -> None:
    """One small generated checkpoint, replaced atomically; raw runs remain immutable."""
    temporary = directory / "plan-state.tmp"
    temporary.write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")
    temporary.replace(directory / "plan-state.json")


def run_plan(root: Path, config: dict, task_ids: list[str] | None = None,
             execute_codex: bool = False, cancel_event=None, *,
             resume_id: str | None = None, fixture_mode: bool = False) -> dict:
    """Bounded serial tasks share one candidate. Resume rechecks stale results."""
    root = Path(root).resolve()
    tasks = {task["id"]: task for task in load_tasks(root, config)}
    errors = validate_tasks(root, list(tasks.values()), config)
    if errors:
        return {"status": "BLOCKED", "errors": errors}
    if not execute_codex:
        return {"status": "DRY_RUN", "agent_calls": 0, "resume_id": resume_id,
                "completion_scope": "SELECTED_STEPS_ONLY", "full_acceptance": False,
                "codex_request": codex_request(config),
                "tasks": task_ids or [key for key in task_order(list(tasks.values())) if tasks[key]["status"] == "ready"]}
    blocker = execution_blocker(config, fixture_mode)
    if blocker:
        return {"status": "BLOCKED", "reason": blocker, "agent_calls": 0, "full_acceptance": False}
    if resume_id:
        if not re.fullmatch(r"[A-Za-z0-9_-]+", resume_id):
            return {"status": "BLOCKED", "reason": "invalid plan run ID"}
        directory = root / "evidence/runs" / resume_id
        state = read_json(directory / "plan-state.json")
        candidate = directory / "candidate"
        if state.get("candidate_path") != str(candidate.resolve()) or not candidate.is_dir():
            return {"status": "BLOCKED", "reason": "saved candidate is missing or moved"}
        state["resume_count"] += 1
    else:
        selected = set(task_ids or [key for key in tasks if tasks[key]["status"] == "ready"])
        if selected - tasks.keys():
            return {"status": "BLOCKED", "reason": "unknown selected task"}
        # Include prerequisites; planned prerequisites remain explicit blockers.
        def include(key):
            for dependency in tasks[key]["dependency_ids"]:
                if dependency not in selected:
                    selected.add(dependency)
                    include(dependency)
        for key in list(selected):
            include(key)
        run_id, directory = create_run(root, "plan")
        candidate = create_candidate(root, directory / "candidate")
        state = {"run_id": run_id, "status": "READY", "candidate_path": str(candidate.resolve()),
                 "task_ids": [key for key in task_order(list(tasks.values())) if key in selected],
                 "completed": {}, "task_runs": [], "resume_count": 0, "agent_calls": 0,
                 "full_acceptance": False, "generation_kind": generation_kind(config, fixture_mode)}
    def stop(status, reason):
        state.update(status=status, reason=reason, completion_scope="SELECTED_STEPS_ONLY",
                     module_completion="NOT_ASSESSED", codex_request=codex_request(config))
        _save_plan_state(directory, state)
        return state
    for key in state["task_ids"]:
        if key not in tasks:
            return stop("REVIEW_REQUIRED", "saved task is no longer configured: " + key)
        task = tasks[key]
        if task["status"] == "planned":
            return stop("BLOCKED", task["blocked_reason"])
    protected = config["workflow"]["protected_paths"]
    if input_manifest(candidate, protected) != input_manifest(root, protected):
        return stop("REVIEW_REQUIRED", "saved candidate protected inputs changed; preserve work for review")
    if state.get("active_before") and state.get("active_task"):
        active = tasks[state["active_task"]]
        current = snapshot(candidate, excludes={".agent-output"})
        findings = check_changes(state["active_before"], current, active, protected)
        findings += new_trust_findings(root, candidate, changed_paths(state["active_before"], current))
        if findings:
            return stop("REVIEW_REQUIRED", "interrupted candidate contains changes outside the active task or new trust constructs")
    _save_plan_state(directory, state)

    def refresh_completed():
        for key in state["task_ids"]:
            if key not in state["completed"]:
                continue
            task = tasks[key]
            record = state["completed"][key]
            current = task_fingerprint(candidate, config, task)
            if record.get("current_fingerprint") == current:
                continue
            run_id, check_dir = create_run(root, "resume-check")
            group_before = snapshot(candidate, excludes={".agent-output"})
            checks = []
            for identifier in task["check_profile_ids"]:
                profile = profile_by_id(config, identifier)
                if profile.get("purpose") == "benchmark" and any(
                        check.get("required", True) and check["result"] not in {"PASS", "TYPECHECKED", "MEASURED"}
                        for check in checks):
                    record.setdefault("deferred_checks", []).append({"profile_id": identifier, "recheck_run_id": run_id,
                        "reason": "required correctness check failed; repair before measuring performance"})
                    continue
                checks.append(verify_profile(candidate, profile, check_dir, run_id, cancel_event))
            if snapshot(candidate, excludes={".agent-output"}) != group_before:
                record.update(status="REVIEW_REQUIRED", current_fingerprint=None, recheck_run_id=run_id)
                write_json(check_dir / "conclusion.json", record)
                return "candidate changed during the resumed independent check group; no results reused"
            required = [check for check in checks if check.get("required", True)]
            covered = {item for check in checks if check["evidence_kind"] == "REAL_LOCAL"
                       for item in check["obligation_ids_checked"]}
            valid = (bool(required) and all(check["result"] in {"PASS", "TYPECHECKED", "MEASURED"} for check in required)
                     and set(task["target_obligation_ids"]).issubset(covered))
            record.update(status="PASSED" if valid else "BLOCKED", current_fingerprint=current if valid else None,
                          recheck_run_id=run_id, obligation_ids_checked=sorted(covered))
            record["evidence_kind"] = "SIMULATED" if any(c["evidence_kind"] == "SIMULATED" for c in checks) else "REAL_LOCAL"
            write_json(check_dir / "conclusion.json", record)
            if not valid:
                return "changed candidate no longer satisfies completed task " + key + "; inspect the saved recheck before resuming"
        return None

    for key in state["task_ids"]:
        if cancel_event is not None and cancel_event.is_set():
            return stop("CANCELLED", "plan cancelled; candidate and completed tasks preserved")
        reason = refresh_completed()
        if reason:
            return stop("BLOCKED", reason)
        if state["completed"].get(key, {}).get("status") == "PASSED":
            continue
        state["active_task"] = key
        state["active_before"] = snapshot(candidate, excludes={".agent-output"})
        _save_plan_state(directory, state)
        result = run_task(root, config, key, True, cancel_event=cancel_event,
                          fixture_mode=fixture_mode, candidate_path=candidate, completed=state["completed"])
        state["task_runs"].append(result.get("run_id"))
        state["agent_calls"] += result.get("agent_calls", 0)
        if result["status"] != "PASSED":
            return stop(result["status"], result.get("reason", "task incomplete"))
        state["completed"][key] = result
        _save_plan_state(directory, state)
    reason = refresh_completed()
    if reason:
        return stop("BLOCKED", reason)
    return stop("PASSED", "selected local tasks complete in saved candidate; no source merge or full project acceptance")

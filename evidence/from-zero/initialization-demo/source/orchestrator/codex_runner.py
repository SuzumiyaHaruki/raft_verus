"""Codex generates a candidate; only the independent checker accepts evidence."""
from __future__ import annotations
import json
import os
import time
from pathlib import Path
from .config import validate_codex_settings
from .evidence import save_process, write_json
from .process import run_process
from .schemas import read_json, validate_schema

KNOWN_EVENTS = {"thread.started", "turn.started", "turn.completed", "turn.failed", "error",
                "item.started", "item.updated", "item.completed"}


def discover_codex(config: dict, cwd: Path, evidence_dir: Path, deadline: float | None = None) -> dict:
    executable = config.get("executable", "codex")
    results = {}
    for name, arguments in [("version", ["--version"]), ("help", ["--help"]),
                            ("exec-help", ["exec", "--help"])]:
        remaining = 15 if deadline is None else min(15, deadline - time.monotonic())
        if remaining <= 0:
            return {"status": "BLOCKED", "reason": "whole-task deadline expired during CLI discovery", "version": None}
        result = run_process([executable, *arguments], cwd, timeout=remaining, max_log_bytes=262144)
        save_process(evidence_dir, "codex-" + name, result)
        results[name] = result
        if result.status != "COMPLETED" or result.exit_code != 0:
            return {"status": "BLOCKED", "reason": f"codex {name} unavailable", "version": None}
    general = results["help"].stdout + results["help"].stderr
    execution = results["exec-help"].stdout + results["exec-help"].stderr
    required = ["--sandbox", "--json", "--output-schema", "--output-last-message", "--add-dir", "--skip-git-repo-check"]
    if config.get("model"):
        required.append("--model")
    if config.get("model_reasoning_effort"):
        required.append("--config")
    if "--ask-for-approval" not in general or any(flag not in execution for flag in required):
        return {"status": "BLOCKED", "reason": "required CLI flags unsupported", "version": None}
    version = results["version"].stdout.strip()
    supported = config.get("supported_versions", [])
    if version not in supported:
        return {"status": "BLOCKED", "reason": "CLI version outside reviewed supported_versions", "version": version}
    return {"status": "READY", "version": version}


def run_codex(config: dict, candidate: Path, prompt: str, schema_path: Path,
              output_dir: Path, cancel_event=None) -> dict:
    validate_codex_settings(config)
    raw_dir = candidate / ".agent-output"
    raw_dir.mkdir(exist_ok=True)
    report_path = raw_dir / "agent_report.json"
    if report_path.exists():
        raise ValueError("stale agent report must not be reused")
    build_directory = output_dir / "agent-build"
    build_directory.mkdir(exist_ok=True)
    argv = [config.get("executable", "codex"), "--ask-for-approval", "never", "exec",
            "--sandbox", "workspace-write", "--json", "--output-schema", str(schema_path.resolve()),
            "--output-last-message", str(report_path.resolve()), "--skip-git-repo-check",
            "--add-dir", str(build_directory.resolve())]
    if config.get("model"):
        argv.extend(["--model", config["model"]])
    if config.get("model_reasoning_effort"):
        argv.extend(["--config", "model_reasoning_effort=" + json.dumps(config["model_reasoning_effort"])])
    argv.append("-")
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = str(build_directory.resolve())
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    result = run_process(argv, candidate, stdin=prompt, env=environment,
                         timeout=config.get("call_timeout_seconds", 120),
                         max_log_bytes=config.get("max_log_bytes", 1048576), cancel_event=cancel_event)
    save_process(output_dir, "codex", result)
    outcome = {"status": result.status, "exit_code": result.exit_code, "warnings": [],
               "events": [], "agent_report": None, "usage": None, "thread_ids": [],
               "model_requested": config.get("model"), "model_resolved": None,
               "reasoning_effort_requested": config.get("model_reasoning_effort")}
    if result.status != "COMPLETED":
        return outcome
    if result.exit_code != 0:
        outcome.update(status="CLI_ERROR", reason="nonzero exit; see restricted stderr log")
        return outcome
    try:
        lines = result.stdout.splitlines()
        if not lines:
            raise ValueError("missing JSONL event stream")
        for line in lines:
            event = json.loads(line)
            if not isinstance(event, dict) or not isinstance(event.get("type"), str):
                raise ValueError("JSONL event must contain a type string")
            outcome["events"].append(event)
            if event["type"] not in KNOWN_EVENTS:
                outcome["warnings"].append("unknown JSONL event: " + event["type"])
            if event["type"] in {"turn.failed", "error"}:
                raise ValueError("Codex emitted an error event")
            if event["type"] == "turn.completed":
                outcome["usage"] = event.get("usage")
            if isinstance(event.get("thread_id"), str):
                outcome["thread_ids"].append(event["thread_id"])
        if raw_dir.is_symlink() or raw_dir.resolve().parent != candidate.resolve():
            raise ValueError("agent output directory escaped candidate")
        if report_path.is_symlink() or not report_path.is_file():
            raise ValueError("final report missing or symlink")
        if report_path.stat().st_size > config.get("max_log_bytes", 1048576):
            raise ValueError("final report exceeds size limit")
        report = read_json(report_path)
        validate_schema(report, schema_path)
        outcome.update(status="CANDIDATE", agent_report=report)
        write_json(output_dir / "agent_report.json", report)
    except (ValueError, OSError) as exc:
        outcome.update(status="PROTOCOL_ERROR", reason=str(exc))
    write_json(output_dir / "codex_protocol.json", outcome)
    return outcome

"""Small offline-first command line; no command silently starts Codex exec."""
from __future__ import annotations
import argparse
import json
import os
import platform
import re
import shutil
import sys
from pathlib import Path
from .codex_runner import discover_codex
from .config import load_config, profile_by_id, validate_codex_settings
from .evidence import create_run, save_process, write_json
from .process import run_process
from .scheduler import format_plan, load_tasks, plan, run_plan, run_task, task_order, validate_tasks
from .schemas import read_json
from .materials import load_proofs
from .verifier_runner import current_proof_evidence, verify_profile
from .workspace import checked_path


def doctor(root: Path, config: dict) -> dict:
    run_id, directory = create_run(root, "doctor")
    tools = {"python": {"status": "AVAILABLE", "version": platform.python_version(), "executable": sys.executable}}
    codex = discover_codex(config.get("codex", {}), root, directory)
    tools["codex"] = codex
    toolchain = config.get("toolchain", {})
    overrides = toolchain.get("tool_env", {})
    if not isinstance(overrides, dict) or any(key not in {"PATH", "RUSTUP_HOME", "CARGO_HOME"} or not isinstance(value, str)
                                            for key, value in overrides.items()):
        raise ValueError("toolchain.tool_env permits only PATH/RUSTUP_HOME/CARGO_HOME strings")
    environment = {key: os.environ[key] for key in ("PATH", "HOME", "USER", "LANG", "LC_ALL", "RUSTUP_HOME", "CARGO_HOME", "LD_LIBRARY_PATH")
                   if key in os.environ}
    environment.update(overrides)
    for name in ("rustc", "cargo", "verus", "z3"):
        executable = toolchain.get(name + "_executable", name)
        args = [executable, "--version"]
        result = run_process(args, root, timeout=15, max_log_bytes=262144, env=environment)
        save_process(directory, name, result)
        tools[name] = {"status": "AVAILABLE" if result.status == "COMPLETED" and result.exit_code == 0 else "BLOCKED",
                       "version": result.stdout.strip() or None,
                       "executable": shutil.which(executable, path=environment.get("PATH")),
                       "configured_tool_environment": overrides}
    tools["verus-tla"] = {"status": "BLOCKED", "version": None,
                          "reason": "no configured locally checked compatible source/toolchain combination"}
    record = {"run_id": run_id, "status": "PARTIAL", "tools": tools,
              "meaning": "read-only local capability observations; no Codex exec or mathematical proof"}
    write_json(directory / "conclusion.json", record)
    return record


def validate_repository(root: Path, config: dict) -> dict:
    errors = []
    try:
        documents = {name: (root / "input" / name).read_text(encoding="utf-8")
                     for name in ("requirements.md", "properties.md", "assumptions.md", "trust.md", "proofs.md")}
        for name, body in documents.items():
            if not re.search(r"Status: (DRAFT|REVIEWED|APPROVED)\.", body):
                errors.append(f"{name} must declare its review status")
        raft = "\n".join(documents.values())
        tasks = load_tasks(root, config)
        if not tasks:
            errors.append("no tasks defined")
        errors.extend(validate_tasks(root, tasks, config))
        obligations, coverage = load_proofs(root)
        indexed = {o['id']: o for o in obligations}
        ids = set(indexed)
        local_ids = set(config.get("local_obligations", {}))
        task_order([dict(id=o['id'], dependency_ids=o['dependencies']) for o in obligations])
        safety = {"S-ELECTION", "S-APPEND", "S-LOG", "S-COMPLETE", "S-APPLY", "S-INTERFACE"}
        liveness = {"L-ELECTION", "L-COMMIT", "L-DELIVER"}
        declared = re.findall(r"^\| ((?:S|L)-[A-Z-]+) / PO-", raft, re.M)
        if set(declared) != safety | liveness or len(declared) != 9:
            errors.append("required Raft safety/liveness properties missing or changed")
        if {"PO-" + identifier for identifier in safety | liveness} - ids:
            errors.append("a top-level property lacks its proof obligation")
        assumptions = re.findall(r"^### (A-[A-Z-]+)$", raft, re.M)
        tcb = re.findall(r"^\| (TCB-[A-Z-]+) \|", raft, re.M)
        if not assumptions or len(assumptions) != len(set(assumptions)) or not tcb or len(tcb) != len(set(tcb)):
            errors.append("missing or duplicate assumption/TCB IDs")
        for task in tasks:
            if set(task['target_obligation_ids']) - ids - local_ids:
                errors.append(f"{task['id']}: unknown obligations")
        inherited = {}
        model_source = checked_path(root, "crates/raft-spec/src/lib.rs").read_text()
        for identifier in task_order([dict(id=o['id'], dependency_ids=o['dependencies']) for o in obligations]):
            item = indexed[identifier]
            if set(item['assumption_ids']) - set(assumptions) or set(item['tcb_entry_ids']) - set(tcb):
                errors.append(f"{identifier}: unknown assumption or TCB reference")
            inherited[identifier] = set(item['assumption_ids'])
            for dependency in item['dependencies']:
                inherited[identifier].update(inherited[dependency])
            if identifier.startswith("PO-S-") and any(a.startswith("A-L-") for a in inherited[identifier]):
                errors.append(f"{identifier}: safety depends on a liveness environment assumption")
            if item['status'] not in {"OPEN", "IN_PROGRESS", "BLOCKED", "REVIEW_REQUIRED", "CHECKED"}:
                errors.append(f"{identifier}: unknown proof status")
            if item['status'] == "CHECKED" and not any(current_proof_evidence(root, profile, identifier)
                    for profile in config["profiles"] if identifier in profile.get("expected_obligation_ids", [])):
                errors.append(f"{identifier}: CHECKED requires current real independent proof evidence")
            if set(item['blocking_decision_ids']) - {f"ADR-{n:02d}" for n in range(1, 11)}:
                errors.append(f"{identifier}: unknown decision ID")
            source = model_source
            for target in item['code_targets']:
                source += checked_path(root, target).read_text()
            if any(not re.search(r"\b" + re.escape(symbol) + r"\b", source) for symbol in item['spec_symbols']):
                errors.append(f"{identifier}: a specification symbol was not located")
        full = profile_by_id(config, "full")
        if set(full.get('expected_obligation_ids', [])) != ids:
            errors.append("full checking profile and proof checklist disagree")
        for profile in config['profiles']:
            if set(profile.get('expected_obligation_ids', [])) - ids - local_ids:
                errors.append(f"{profile['id']}: unknown checked obligation")
            for entry in profile['entrypoints']:
                if not checked_path(root, entry).is_file():
                    errors.append(f"{profile['id']}: missing entrypoint {entry}")
        # Each main row covers its own source/symbols; additional rows cover branches.
        for entry in coverage:
            if set(entry['obligation_ids']) - ids:
                errors.append("coverage references unknown obligations")
            for check in entry['check_profile_ids']:
                profile_by_id(config, check)
            for source in entry['source_paths']:
                if not checked_path(root, source).is_file():
                    errors.append("coverage references a missing source")
        from .scheduler import input_bindings
        input_bindings(root, config)
    except (ValueError, KeyError, OSError, TypeError) as exc:
        errors.append(str(exc))
    return {"status": "PASS" if not errors else "FAIL", "check_kind": "STRUCTURAL_ONLY",
            "contract_status": "DRAFT", "full_acceptance": False, "errors": errors,
            "meaning": "Markdown references and configuration checks only; no proof obligation closed"}


def report_run(root: Path, run_id: str) -> dict:
    if not run_id or "/" in run_id or "\\" in run_id or run_id in {".", ".."}:
        raise ValueError("invalid run ID")
    directory = root / "evidence/runs" / run_id
    metadata = read_json(directory / "run.json")
    conclusion_path = directory / "conclusion.json"
    conclusion = read_json(conclusion_path if conclusion_path.is_file() else directory / "plan-state.json")
    checks = [read_json(path) for path in sorted(directory.rglob("*.check.json"))
              if "candidate" not in path.relative_to(directory).parts]
    return {"run": metadata, "controller_conclusion": conclusion, "independent_checks": checks,
            "evidence_scope": "LOCAL_DEVELOPMENT_NOT_TAMPER_RESISTANT", "full_acceptance": bool(conclusion.get("full_acceptance"))}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description="DRAFT Raft workflow; default operations do not invoke Codex exec")
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--config", default="workflow.toml")
    parser.add_argument("--model", help="Codex model override for this invocation; does not start generation")
    parser.add_argument("--reasoning-effort", help="Codex model_reasoning_effort override, for example high")
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("doctor", "validate"):
        commands.add_parser(name)
    planning = commands.add_parser("plan")
    planning.add_argument("--format", choices=("json", "text"), default="json")
    run = commands.add_parser("run-task")
    run.add_argument("--task", required=True)
    mode = run.add_mutually_exclusive_group()
    mode.add_argument("--dry-run", action="store_true")
    mode.add_argument("--execute-codex", action="store_true")
    for command in ("run-plan", "resume"):
        batch = commands.add_parser(command)
        if command == "resume":
            batch.add_argument("--run-id", required=True)
        else:
            batch.add_argument("--tasks", nargs="+")
        batch_mode = batch.add_mutually_exclusive_group()
        batch_mode.add_argument("--dry-run", action="store_true")
        batch_mode.add_argument("--execute-codex", action="store_true")
    prompt = commands.add_parser("prompt")
    prompt.add_argument("--task", required=True)
    verify = commands.add_parser("verify")
    verify.add_argument("--profile", required=True)
    report = commands.add_parser("report")
    report.add_argument("--run-id", required=True)
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve()
        config = load_config(root, args.config)
        if args.model is not None:
            config.setdefault("codex", {})["model"] = args.model
        if args.reasoning_effort is not None:
            config.setdefault("codex", {})["model_reasoning_effort"] = args.reasoning_effort
        validate_codex_settings(config.get("codex", {}))
        if args.command == "prompt":
            from .scheduler import render_prompt
            task = next((t for t in load_tasks(root, config) if t['id'] == args.task), None)
            if task is None:
                raise ValueError("unknown task ID")
            print(render_prompt(root, task))
            return 0
        if args.command == "doctor":
            result = doctor(root, config)
        elif args.command == "validate":
            result = validate_repository(root, config)
        elif args.command == "plan":
            result = plan(root, config)
            if args.format == "text":
                print(format_plan(result))
                return 1 if result.get("errors") else 0
        elif args.command == "run-task":
            result = run_task(root, config, args.task, args.execute_codex)
        elif args.command in {"run-plan", "resume"}:
            result = run_plan(root, config, getattr(args, "tasks", None), args.execute_codex,
                              resume_id=getattr(args, "run_id", None))
        elif args.command == "verify":
            run_id, directory = create_run(root, "verify")
            result = verify_profile(root, profile_by_id(config, args.profile), directory, run_id)
            write_json(directory / "conclusion.json", result)
        else:
            result = report_run(root, args.run_id)
        print(json.dumps(result, indent=2, ensure_ascii=False))
        status = result.get("status", result.get("result"))
        return 1 if status in {"FAIL", "FAILED", "ERROR", "BLOCKED", "REVIEW_REQUIRED", "CANCELLED", "TIMEOUT"} else 0
    except (OSError, ValueError, KeyError) as exc:
        print(json.dumps({"status": "ERROR", "reason": str(exc)}, ensure_ascii=False))
        return 2

"""Independent configured checker. Agent claims never enter acceptance logic."""
from __future__ import annotations
import json
import os
import re
import time
from pathlib import Path
from .evidence import save_process, utc_now, write_json
from .process import run_process
from .workspace import checked_path, digest_json, snapshot


def verify_profile(root: str | Path, profile: dict, evidence_dir: str | Path,
                   run_id: str = "local", cancel_event=None) -> dict:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", str(profile.get("id", ""))):
        raise ValueError("unsafe check profile ID")
    root, evidence_dir = Path(root), Path(evidence_dir)
    evidence_dir.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    before = snapshot(root)
    report = {"schema_version": "1", "check_id": profile["id"], "run_id": run_id,
              "check_kind": profile.get("kind", "verification"), "profile_id": profile["id"],
              "result": "NOT_RUN", "argv": profile.get("argv", []),
              "working_directory": str(root.resolve()), "input_manifest_digest": digest_json(before),
              "tool_versions": {}, "started_at": utc_now(), "finished_at": None,
              "exit_code": None, "stdout_digest": None, "stderr_digest": None,
              "evidence_paths": [], "obligation_ids_checked": [], "coverage_findings": [],
              "policy_findings": [], "evidence_kind": profile.get("evidence_kind", "REAL_LOCAL"),
              "contract_status": "DRAFT", "full_acceptance": False}

    def finish(result, reason):
        report.update(result=result, reason=reason, finished_at=utc_now())
        write_json(evidence_dir / (profile["id"] + ".check.json"), report)
        return report

    if report["evidence_kind"] not in {"REAL_LOCAL", "SIMULATED"}:
        return finish("ERROR", "unknown evidence kind")
    if profile.get("blocked_reason"):
        return finish("BLOCKED", profile["blocked_reason"])
    targets = profile.get("expected_targets", [])
    entries = profile.get("entrypoints", [])
    if not targets or not entries:
        return finish("ERROR", "no expected verification targets or entrypoints")
    try:
        for entry in entries:
            path = checked_path(root, entry)
            if (not path.is_file() or path.is_symlink()
                    or entry.split("/", 1)[0] in {"evidence", ".agent-output", ".git", "target", "__pycache__"}):
                raise ValueError("invalid checker source entrypoint: " + entry)
    except ValueError as exc:
        return finish("ERROR", str(exc))
    environment = {key: os.environ[key] for key in ("PATH", "HOME", "USER", "LANG", "LC_ALL", "RUSTUP_HOME", "CARGO_HOME", "LD_LIBRARY_PATH") if key in os.environ}
    overrides = profile.get("tool_env", {})
    if not isinstance(overrides, dict) or any(key not in {"PATH", "RUSTUP_HOME", "CARGO_HOME"} or not isinstance(value, str) for key, value in overrides.items()):
        return finish("ERROR", "tool_env permits only configured PATH/RUSTUP_HOME/CARGO_HOME strings")
    environment.update(overrides)
    report["tool_environment"] = overrides
    # Build outputs cannot mutate the source manifest being checked.
    environment["CARGO_TARGET_DIR"] = str((evidence_dir / "build").resolve())
    deadline = started + profile.get("timeout_seconds", 120)
    adapter = profile.get("adapter")
    if adapter not in {"fake-json-v1", "verus-calibrated-text-v1", "command-text-v1"}:
        return finish("BLOCKED", "checker output protocol has not been calibrated")
    if adapter == "fake-json-v1":
        report["evidence_kind"] = "SIMULATED"
    if adapter == "verus-calibrated-text-v1":
        calibration = profile.get("calibration", {})
        if not all(calibration.get(key) for key in ("tool_version", "summary_pattern", "positive_evidence", "negative_evidence")):
            return finish("BLOCKED", "real Verus requires reviewed version and actual positive/negative calibration evidence")
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return finish("TIMEOUT", "checker deadline expired before version discovery")
        version = run_process([profile["argv"][0], "--version"], root, timeout=min(15, remaining), env=environment)
        save_process(evidence_dir, profile["id"] + "-version", version)
        report["tool_versions"]["verus"] = version.stdout.strip()
        if version.status != "COMPLETED" or version.exit_code != 0 or version.stdout.strip() != calibration["tool_version"]:
            return finish("BLOCKED", "Verus version differs from calibrated version")
    argv = [part.replace("{root}", str(root.resolve())) for part in profile["argv"]]
    if any(flag in " ".join(argv) for flag in ("--no-verify", "--verify-function", "--verify-module")):
        return finish("ERROR", "disabled or partial verification argument rejected")
    report["argv"] = argv
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        return finish("TIMEOUT", "checker deadline expired before verification")
    result = run_process(argv, root, timeout=remaining,
                         max_log_bytes=profile.get("max_log_bytes", 1048576),
                         cancel_event=cancel_event, env=environment)
    paths = save_process(evidence_dir, profile["id"], result)
    report.update(exit_code=result.exit_code, stdout_digest=paths["stdout_digest"],
                  stderr_digest=paths["stderr_digest"], evidence_paths=[paths["stdout"], paths["stderr"]])
    after = snapshot(root)
    if before != after:
        report["policy_findings"].append("source changed while independent check was running")
        return finish("ERROR", "checked source snapshot was mutated")
    if result.status != "COMPLETED":
        status = {"TIMEOUT": "TIMEOUT", "BLOCKED": "BLOCKED"}.get(result.status, "ERROR")
        return finish(status, "checker process: " + result.status)
    text = result.stdout + "\n" + result.stderr
    if not text.strip():
        return finish("ERROR", "empty verifier output is not a pass")
    try:
        if adapter == "fake-json-v1":
            parsed = json.loads(result.stdout)
            if not isinstance(parsed, dict) or parsed.get("protocol") != "raft-check-v1":
                raise ValueError("unsupported fake verifier protocol")
            checked = parsed.get("checked_targets")
            verified, errors = parsed.get("verified"), parsed.get("errors")
            if not isinstance(checked, list) or not all(isinstance(s, str) for s in checked):
                raise ValueError("invalid checked_targets")
            if type(verified) is not int or type(errors) is not int or verified < 0 or errors < 0:
                raise ValueError("invalid verification counts")
            if result.exit_code != 0 or errors:
                return finish("FAIL", "independent verifier rejected candidate")
            if verified < 1 or not set(targets).issubset(checked):
                return finish("FAIL", "expected targets were not checked")
        elif adapter == "verus-calibrated-text-v1":
            match = re.search(profile["calibration"]["summary_pattern"], text)
            if not match or not {"verified", "errors"}.issubset(match.groupdict()):
                raise ValueError("unsupported Verus summary")
            verified, errors = int(match["verified"]), int(match["errors"])
            report["verification_summary"] = {"verified": verified, "errors": errors}
            expected_rejection = profile.get("expect_rejection", False)
            if profile.get("purpose") == "typecheck":
                if expected_rejection or profile.get("expected_obligation_ids"):
                    return finish("ERROR", "typecheck profile cannot reject or close proof obligations")
                if result.exit_code != 0 or errors:
                    return finish("FAIL", "candidate model failed Verus syntax/type checking")
                report["proof_result"] = "NOT_RUN"
                report["coverage_findings"].append("definitions typechecked; zero Raft proof obligations are discharged")
                return finish("TYPECHECKED", "candidate definitions checked; no mathematical theorem acceptance")
            if expected_rejection:
                if result.exit_code == 0 or errors < 1:
                    return finish("FAIL", "negative smoke was not rejected")
            elif result.exit_code != 0 or errors or verified < max(1, profile.get("minimum_verified", 1)):
                return finish("FAIL", "Verus reported rejection or insufficient checked definitions")
            report["coverage_findings"].append("symbol coverage is manifest-based; summary counts cannot prove per-symbol coverage")
        else:
            pattern = profile.get("success_pattern")
            if not pattern or not re.search(pattern, text) or result.exit_code != 0:
                return finish("FAIL", "configured build/test success evidence absent")
            report["coverage_findings"].append("build/test only; no mathematical proof")
    except (ValueError, TypeError, KeyError, re.error) as exc:
        return finish("ERROR", str(exc))
    report["obligation_ids_checked"] = profile.get("expected_obligation_ids", [])
    if report["evidence_kind"] == "SIMULATED" or profile.get("expect_rejection") or adapter == "command-text-v1":
        report["obligation_ids_checked"] = []
    return finish("PASS", "profile checks passed locally; no full Raft acceptance")

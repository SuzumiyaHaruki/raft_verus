"""Independent configured checker. Agent claims never enter acceptance logic."""
from __future__ import annotations
import json
import math
import os
import re
import time
from pathlib import Path
from .evidence import save_process, utc_now, write_json
from .process import run_process
from .workspace import checked_path, digest_json, input_manifest, snapshot

PURPOSES = {"build", "typecheck", "test", "proof", "benchmark", "policy"}


def check_binding(root: Path, profile: dict) -> str:
    paths = list(dict.fromkeys(profile.get("source_inputs", profile.get("entrypoints", [])) +
                               profile.get("proof_entrypoints", [])))
    sources = input_manifest(root, paths)
    # Reuse the existing evidence binding; checker changes invalidate old checks.
    if (root / "orchestrator").is_dir():
        sources.update(input_manifest(root, ["orchestrator"]))
    # Runtime deadlines can be clipped to the remaining task budget. They are
    # execution limits, not a different theorem or checker policy.
    policy = {key: value for key, value in profile.items() if key not in {"timeout_seconds", "max_log_bytes"}}
    return digest_json({"sources": sources, "profile": policy})


def current_proof_evidence(root: Path, profile: dict, obligation: str) -> bool:
    try:
        binding = check_binding(root, profile)
    except (OSError, ValueError):
        return False
    for path in (root / "evidence/runs").glob("**/*.check.json"):
        try:
            record = json.loads(path.read_text())
        except (OSError, ValueError):
            continue
        if (record.get("profile_id") == profile["id"] and record.get("result") == "PASS"
                and record.get("purpose") == "proof" and record.get("evidence_kind") == "REAL_LOCAL"
                and record.get("input_manifest_digest") == binding
                and obligation in record.get("obligation_ids_checked", [])):
            return True
    return False


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
              "purpose": profile.get("purpose", "proof" if profile.get("adapter") != "command-text-v1" else "build"),
              "required": profile.get("required", True),
              "expected_targets": profile.get("expected_targets", []),
              "expected_obligation_ids": profile.get("expected_obligation_ids", []),
              "entrypoints": profile.get("entrypoints", []),
              "expected_behavior": profile.get("expected_behavior", "Configured targets and required observations succeed"),
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
    if report["purpose"] not in PURPOSES:
        return finish("ERROR", "unknown check purpose")
    if profile.get("planned", False):
        return finish("BLOCKED", profile.get("blocked_reason", "check prerequisites are not implemented"))
    for entry in profile.get("planned_entrypoints", []):
        if not checked_path(root, entry).is_file():
            return finish("BLOCKED", "planned proof entrypoint is not implemented: " + entry)
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
    try:
        report["input_manifest_digest"] = check_binding(root, profile)
    except (ValueError, OSError) as exc:
        return finish("ERROR", str(exc))
    full_project = profile.get("full_project", profile["id"] == "full")
    if full_project:
        if profile.get("specification_status", profile.get("contract_status")) != "APPROVED":
            return finish("BLOCKED", "full acceptance requires separately approved specifications and complete safety, liveness and refinement coverage")
        from .materials import load_proofs
        obligations, _ = load_proofs(root)
        if set(profile.get("expected_obligation_ids", [])) != {o["id"] for o in obligations}:
            return finish("BLOCKED", "full checking coverage is incomplete")
        if profile.get("required_isolation") == "external":
            return finish("BLOCKED", "required external checker isolation is not implemented by this local controller")
    environment = {key: os.environ[key] for key in ("PATH", "HOME", "USER", "LANG", "LC_ALL", "RUSTUP_HOME", "CARGO_HOME", "LD_LIBRARY_PATH") if key in os.environ}
    overrides = profile.get("tool_env", {})
    if not isinstance(overrides, dict) or any(key not in {"PATH", "RUSTUP_HOME", "CARGO_HOME"} or not isinstance(value, str) for key, value in overrides.items()):
        return finish("ERROR", "tool_env permits only configured PATH/RUSTUP_HOME/CARGO_HOME strings")
    environment.update(overrides)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
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
    if profile.get("scenario"):
        scenario = dict(profile["scenario"])
        trace = scenario.get("trace_path")
        if trace:
            try:
                trace_source = checked_path(root, trace)
                trace_copy = evidence_dir / (profile["id"] + ".trace.json")
                trace_copy.write_bytes(trace_source.read_bytes())
                scenario["saved_trace"] = str(trace_copy.resolve())
            except (ValueError, OSError) as exc:
                return finish("ERROR", "scenario trace unavailable: " + str(exc))
        scenario["state_observations"] = paths["stdout"]
        report["scenario"] = scenario
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
            function_results = {}
            entire_crate = False
            if "--output-json" in argv:
                parsed = json.loads(result.stdout)
                summary = parsed["verification-results"]
                verified, errors = summary["verified"], summary["errors"]
                if type(verified) is not int or type(errors) is not int or verified < 0 or errors < 0:
                    raise ValueError("invalid Verus JSON verification counts")
                entire_crate = summary.get("is-verifying-entire-crate") is True
                if errors == 0 and result.exit_code == 0 and summary.get("success") is not True:
                    raise ValueError("Verus JSON did not confirm success")
                for module in parsed.get("times-ms", {}).get("smt", {}).get("smt-run-module-times", []):
                    for function in module.get("function-breakdown", []):
                        function_results[function["function"]] = function.get("success") is True
                report["function_results"] = function_results
                report["entire_crate_checked"] = entire_crate
            else:
                match = re.search(profile["calibration"]["summary_pattern"], text)
                if not match or not {"verified", "errors"}.issubset(match.groupdict()):
                    raise ValueError("unsupported Verus summary")
                verified, errors = int(match["verified"]), int(match["errors"])
            report["verification_summary"] = {"verified": verified, "errors": errors}
            expected_rejection = profile.get("expect_rejection", False)
            if report["purpose"] == "typecheck":
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
                rejection = profile.get("rejection_pattern")
                if not rejection or not re.search(rejection, text, re.S):
                    return finish("FAIL", "rejection did not match the intended proof failure")
                if re.search(r"error\[E\d+\]|could not compile", text):
                    return finish("FAIL", "compiler failure is not the expected negative proof result")
            elif result.exit_code != 0 or errors or verified < max(1, profile.get("minimum_verified", 1)):
                return finish("FAIL", "Verus reported rejection or insufficient checked definitions")
            if profile.get("expected_obligation_ids"):
                mapped = profile.get("proof_targets", {})
                proof_entries = profile.get("proof_entrypoints", [])
                invoked = {str(checked_path(root, entry).resolve()) for entry in proof_entries}
                if (not proof_entries or not any(str((root / arg).resolve()) in invoked for arg in argv[1:])
                        or set(mapped) != set(profile["expected_obligation_ids"])):
                    return finish("ERROR", "proof obligations require a whole-file Verus invocation and an explicit target map")
                if not entire_crate or not mapped or any(function_results.get(symbol) is not True for symbol in mapped.values()):
                    return finish("FAIL", "actual whole-crate Verus JSON must confirm each mapped function; names or unrelated smoke proofs are insufficient")
                # source_inputs also binds runtime tests and build adapters. Only
                # declared Verus entrypoints/dependencies are proof source: a
                # runtime #[cfg(test)] module is not a verification escape.
                source_paths = list(dict.fromkeys(entries + proof_entries))
                source = "\n".join(checked_path(root, path).read_text() for path in source_paths if checked_path(root, path).is_file())
                from .policy import scan_trust
                findings = scan_trust(root, source_paths)
                # This existing reviewed ghost-only model import is shared by the
                # Verus/Rust build path. Do not permit arbitrary conditional code.
                def ghost_model_import(finding):
                    if finding["token"] != "cfg":
                        return False
                    lines = checked_path(root, finding["path"]).read_text().splitlines()
                    index = finding["line"] - 1
                    return (lines[index].strip() == "#[cfg(verus_keep_ghost)]"
                            and index + 1 < len(lines) and lines[index + 1].strip() in
                            {"use crate::model;", "use crate::representation::log_view;",
                             "use crate::initialization::init_result;"})
                findings = [finding for finding in findings if not ghost_model_import(finding)]
                if findings:
                    report["policy_findings"].extend(findings)
                    return finish("ERROR", "proof source contains unreviewed verification escapes")
                for symbol in mapped.values():
                    # A mention or declaration without a postcondition cannot discharge a goal.
                    if not re.search(r"\bfn\s+" + re.escape(symbol.rsplit("::", 1)[-1]) + r"\s*\([^;]*?\bensures\b", source, re.S):
                        return finish("ERROR", "proof target has no checked function contract: " + symbol)
                report["coverage_findings"].append("complete Verus entrypoint verified; reviewed target mapping binds its function contracts to local obligations")
        else:
            pattern = profile.get("success_pattern")
            if result.exit_code != 0 or (report["purpose"] != "benchmark" and not pattern) or (pattern and not re.search(pattern, text)):
                return finish("FAIL", "configured build/test success evidence absent")
            if report["purpose"] == "proof":
                return finish("ERROR", "command output cannot discharge a formal proof")
            if report["purpose"] == "test":
                count_pattern = profile.get("selected_count_pattern", r"test result: ok\. (\d+) passed")
                selected = re.findall(count_pattern, text)
                try:
                    count = sum(int(value) for value in selected)
                except (TypeError, ValueError):
                    return finish("ERROR", "invalid selected test count")
                report["tests_selected"] = count
                if count < profile.get("minimum_tests", 1):
                    return finish("FAIL", "required test selection was empty or below its declared minimum")
            if report["purpose"] == "typecheck":
                report["proof_result"] = "NOT_RUN"
                return finish("TYPECHECKED", "typecheck completed; no theorem discharged")
            if report["purpose"] == "benchmark":
                measurements = []
                for line in result.stdout.splitlines():
                    try:
                        record = json.loads(line)
                    except ValueError:
                        continue
                    if isinstance(record, dict) and str(record.get("kind", "")).lower() in {"measurement", "benchmark", "core_microbenchmark"}:
                        measurements.append(record)
                report["measurements"] = measurements
                report["performance_verdict"] = "INCONCLUSIVE"
                report["reference"] = profile.get("reference", "No approved reference threshold")
                if len(measurements) < profile.get("minimum_measurements", 1):
                    return finish("FAIL", "required measurement data is missing")
                for measurement in measurements:
                    samples = measurement.get("samples_ns_per_operation", measurement.get("samples_ns", measurement.get("samples")))
                    if (not measurement.get("workload") or not isinstance(samples, list) or len(samples) < 2
                            or any(type(value) not in (int, float) or not math.isfinite(value) or value < 0 for value in samples)):
                        return finish("FAIL", "measurement requires an identified workload and repeated finite numeric samples")
                return finish("MEASURED", "measurement data recorded; no performance threshold or regression verdict")
            report["coverage_findings"].append("build/test only; no mathematical proof")
    except (ValueError, TypeError, KeyError, re.error) as exc:
        return finish("ERROR", str(exc))
    report["obligation_ids_checked"] = profile.get("expected_obligation_ids", [])
    if report["evidence_kind"] == "SIMULATED" or profile.get("expect_rejection") or adapter == "command-text-v1":
        report["obligation_ids_checked"] = []
    if full_project and report["evidence_kind"] == "REAL_LOCAL":
        report["full_acceptance"] = True
    return finish("PASS", "profile checks passed locally; no full Raft acceptance")

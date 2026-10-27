from __future__ import annotations
import re
from pathlib import Path

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib
    except ImportError:
        # Explicit local compatibility fallback; normal installation declares tomli.
        from pip._vendor import tomli as tomllib


def validate_codex_settings(settings: dict) -> None:
    for key in ("model", "model_reasoning_effort"):
        if key in settings and (not isinstance(settings[key], str) or not settings[key]
                                or any(c.isspace() for c in settings[key])):
            raise ValueError(f"codex.{key} must be a nonempty string without whitespace")


def load_config(root: Path, path: str | Path = "workflow.toml") -> dict:
    location = Path(path)
    if not location.is_absolute():
        location = root / location
    config = tomllib.loads(location.read_text(encoding="utf-8"))
    config["_config_path"] = str(location.resolve())
    validate_codex_settings(config.get("codex", {}))
    from .toolchain import resolve
    selected = None
    if "install_root" in config.get("toolchain", {}):
        selected = resolve(config["toolchain"])
        config["toolchain"]["resolved"] = selected
        config["toolchain"]["tool_env"] = selected["environment"]
        for name, path in selected["paths"].items():
            config["toolchain"][name + "_executable"] = path
    profiles = config.get("checks", config.get("profiles", []))
    if isinstance(profiles, dict):
        profiles = [dict(value, id=key) for key, value in profiles.items()]
    for profile in profiles:
        if selected:
            profile["resolved_toolchain"] = selected
            profile["argv"] = [part.replace("{verus}", selected["paths"]["verus"]) for part in profile["argv"]]
            profile["tool_env"] = dict(selected["environment"], RAFT_WORKFLOW_CONFIG=str(location.resolve()))
        profile.setdefault("purpose", "proof" if profile.get("adapter") != "command-text-v1" else "build")
        profile.setdefault("timeout_seconds", config.get("budget", {}).get("call_timeout_seconds", 120))
        profile.setdefault("max_log_bytes", config.get("budget", {}).get("max_log_bytes", 1048576))
        if profile.get("adapter") == "verus-calibrated-text-v1":
            profile.setdefault("tool_env", config.get("toolchain", {}).get("tool_env", {}))
            profile.setdefault("calibration", config.get("verus_calibration", {}))
    config["profiles"] = profiles
    protected = config.get("workflow", {}).get("protected_paths")
    if not isinstance(protected, list) or not protected or not all(isinstance(p, str) and p for p in protected):
        raise ValueError("workflow.protected_paths must be a nonempty path list")
    ids = [p.get("id") for p in profiles]
    if None in ids or len(ids) != len(set(ids)):
        raise ValueError("missing or duplicate profile IDs")
    budget = config.get("budget", {})
    for key, value in budget.items():
        minimum = 0 if key in {"max_agent_calls", "max_repair_rounds", "max_redesigns"} else 1
        if type(value) is not int or value < minimum:
            raise ValueError(f"invalid budget value: {key}")
    for key, value in config.get("plan_budget", {}).items():
        if key not in {"max_agent_calls", "active_seconds"} or type(value) is not int or value < 0:
            raise ValueError(f"invalid plan budget value: {key}")
    for profile in profiles:
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", str(profile.get("id", ""))):
            raise ValueError("unsafe check profile ID")
        if not profile.get("entrypoints") or not profile.get("expected_targets"):
            raise ValueError(f"profile {profile['id']} lacks targets or entrypoints")
        if profile["purpose"] not in {"build", "typecheck", "test", "proof", "benchmark", "policy"}:
            raise ValueError("invalid check purpose: " + profile["purpose"])
        if profile.get("purpose") != "proof" and profile.get("expected_obligation_ids"):
            raise ValueError("only a proof check may declare obligation coverage")
        if profile.get("adapter") not in {"fake-json-v1", "verus-calibrated-text-v1", "command-text-v1"}:
            raise ValueError(f"unknown checker adapter: {profile.get('adapter')}")
        for key in ("timeout_seconds", "max_log_bytes"):
            if key in profile and (type(profile[key]) not in (int, float) or profile[key] <= 0):
                raise ValueError(f"invalid {key} for {profile['id']}")
        if profile.get("success_pattern"):
            try:
                re.compile(profile["success_pattern"])
            except re.error as exc:
                raise ValueError("invalid success_pattern") from exc
        argv = profile.get("argv")
        if not isinstance(argv, list) or not argv or not all(isinstance(x, str) for x in argv):
            raise ValueError(f"profile {profile['id']} argv must be a string array")
        if any(x in " ".join(argv) for x in ["--no-verify", "--verify-function", "--verify-module"]):
            raise ValueError("partial/disabled verifier invocation needs a separate reviewed policy")
    return config


def profile_by_id(config: dict, identifier: str) -> dict:
    for profile in config["profiles"]:
        if profile["id"] == identifier:
            return profile
    raise ValueError(f"unknown check profile: {identifier}")

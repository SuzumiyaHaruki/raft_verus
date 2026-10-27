"""Small persistent budget/history records, owned by the existing serial plan."""
import shutil
import time
from pathlib import Path
from .policy import matches
from .workspace import checked_path, snapshot


def usage(budget):
    return dict(calls=0, repairs=0, redesigns=0, active_seconds=0.0,
                call_limit=budget["max_agent_calls"], repair_limit=budget.get("max_repair_rounds", 0),
                redesign_limit=budget.get("max_redesigns", 0), seconds_limit=budget["task_timeout_seconds"],
                attempts=[], diagnostics={}, stalls=0, failing=[], checkpoints={}, generation_kinds=[])


class Budget:
    def __init__(self, task, plan, save):
        self.task, self.plan, self.save = task, plan, save

    def remaining(self):
        return min(r["seconds_limit"] - r["active_seconds"] for r in self.records())

    def records(self):
        return [self.task] + ([self.plan] if self.plan is not None else [])

    def can_call(self, mode):
        return self.remaining() > 0 and all(r["calls"] < r["call_limit"] for r in self.records()) and (
            mode == "initial" or self.task["repairs"] < self.task["repair_limit"]) and (
            mode != "redesign" or self.task["redesigns"] < self.task["redesign_limit"])

    def reserve(self, seconds, mode=None):
        """Persist before launch. An interrupted reservation stays fully charged."""
        seconds = max(0, min(seconds, self.remaining()))
        for record in self.records():
            record["active_seconds"] += seconds
            if mode:
                record["calls"] += 1
                record["repairs"] += mode != "initial"
                record["redesigns"] += mode == "redesign"
        self.save()
        return seconds, time.monotonic()

    def settle(self, reservation):
        seconds, started = reservation
        elapsed = time.monotonic() - started
        for record in self.records():
            record["active_seconds"] += elapsed - seconds
        self.save()


def checkpoint(candidate, task, record, iteration, checks):
    passed = [c["profile_id"] for c in checks if c["result"] in {"PASS", "TYPECHECKED"}]
    if not passed:
        return
    destination = iteration / "checkpoint"
    files = [p for p, item in snapshot(candidate).items()
             if item["kind"] == "file" and matches(p, task["permitted_change_paths"])]
    for name in files:
        target = destination / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(candidate / name, target)
    for scope in passed:
        record["checkpoints"][scope] = dict(path=str(destination), files=files,
            editable_scope=task["permitted_change_paths"], checked_scope=scope,
            result=next(c["result"] for c in checks if c["profile_id"] == scope))


def restore(candidate, saved):
    """Caller validates ownership and saves the abandoned patch before this edit."""
    source = Path(saved["path"])
    for name, item in snapshot(candidate).items():
        if item["kind"] == "file" and matches(name, saved["editable_scope"]) and name not in saved["files"]:
            checked_path(candidate, name).unlink()
    for name in saved["files"]:
        target = checked_path(candidate, name)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(checked_path(source, name), target)

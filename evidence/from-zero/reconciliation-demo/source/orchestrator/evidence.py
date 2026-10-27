from __future__ import annotations
import datetime
import json
import os
import uuid
from pathlib import Path
from .workspace import digest_bytes


def utc_now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def create_run(root: Path, kind: str) -> tuple[str, Path]:
    identifier = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:12]
    directory = root / "evidence" / "runs" / identifier
    directory.mkdir(parents=True, mode=0o700, exist_ok=False)
    os.chmod(directory, 0o700)
    write_json(directory / "run.json", {"run_id": identifier, "kind": kind, "started_at": utc_now(),
                                      "evidence_scope": "LOCAL_DEVELOPMENT_NOT_TAMPER_RESISTANT"})
    return identifier, directory


def write_text(path: Path, text: str) -> None:
    # Existing evidence is never silently replaced.
    with path.open("x", encoding="utf-8") as stream:
        os.chmod(path, 0o600)
        stream.write(text)


def write_json(path: Path, value) -> None:
    write_text(path, json.dumps(value, ensure_ascii=False, indent=2) + "\n")


def save_process(directory: Path, prefix: str, result) -> dict:
    paths = {}
    for channel in ("stdout", "stderr"):
        path = directory / f"{prefix}.{channel}.log"
        content = getattr(result, channel)
        write_text(path, content)
        paths[channel] = str(path)
        paths[channel + "_digest"] = digest_bytes(content.encode("utf-8"))
    write_json(directory / f"{prefix}.process.json", result.to_dict())
    return paths

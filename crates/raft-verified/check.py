#!/usr/bin/env python3
"""DRAFT local checks using installed Verus dependencies and matching child Rust.

No downloads, default-toolchain changes, source rewriting or simulated checks.
The Verus macro erases proof syntax from the same source during Cargo builds.
"""
import argparse
import os
from pathlib import Path
import subprocess
import sys


def cargo_environment(selected, extra_flags=()):
    """Use the same installed macro erasure and Rust selection for core and hosts."""
    verus = Path(selected["paths"]["verus"])
    paths = selected["paths"]
    env = os.environ.copy()
    env.update(selected["environment"])
    env["RUSTC"] = paths["rustc"]
    env["RUSTDOC"] = paths["rustdoc"]
    flags = ["--extern", f"vstd={verus.parent / 'libvstd.rlib'}",
             "-L", f"dependency={verus.parent}", "--check-cfg",
             "cfg(verus_keep_ghost,verus_keep_ghost_body,verus_verify_core,verus_no_vstd)"]
    flags += list(extra_flags)
    # Use structured flag boundaries; the runner never invokes a shell.
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env["CARGO_ENCODED_RUSTDOCFLAGS"] = "\x1f".join(flags)
    env.pop("RUSTFLAGS", None)
    return env


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("check", choices=("build", "test", "replay", "bench", "proof", "node-proof", "node-replay", "node-bench", "persist-proof", "persist-replay", "persist-bench", "progress-proof", "core-proof"))
    parser.add_argument("extra", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root))
    from orchestrator.config import load_config
    selected = load_config(root, os.environ.get("RAFT_WORKFLOW_CONFIG", "workflow.toml"))["toolchain"]["resolved"]
    import json
    print("Toolchain: " + json.dumps(selected), file=sys.stderr)
    if selected["missing"]:
        print("BLOCKED: missing local tools: " + ", ".join(selected["missing"]), file=sys.stderr)
        return 2
    paths = selected["paths"]
    verus = Path(paths["verus"])
    rust = Path(paths["rustc"]).parent
    env = cargo_environment(selected)
    if args.check in {"proof", "node-proof", "persist-proof", "progress-proof", "core-proof"}:
        command = [str(verus), "--crate-type", "lib", "--no-cheating", "--triggers-mode", "silent",
                   "crates/raft-proofs/" + {"proof": "reconcile.rs", "node-proof": "node_init.rs", "persist-proof": "persistence.rs", "progress-proof": "progress.rs", "core-proof": "core.rs"}[args.check]]
    else:
        command = [str(rust / "cargo")]
        if args.check == "build":
            command += ["check", "--workspace", "--locked", "--offline"]
        elif args.check == "test":
            command += ["test", "--workspace", "--locked", "--offline"]
        else:
            command += ["run", "--locked", "--offline", "-p", "raft-verified", "--example"]
            command += [{"replay": "replay", "bench": "measure_reconcile",
                         "node-replay": "initialize", "node-bench": "initialize", "persist-replay": "persistence", "persist-bench": "persistence"}[args.check]]
            if args.check.endswith("bench"):
                command += ["--release"]
            if args.check in {"node-bench", "persist-bench"}:
                command += ["--", "--measure"]
        if args.check.endswith("bench"):
            env.setdefault("RAFT_CODE_VERSION", "DRAFT workspace; see independently bound check evidence")
    command += args.extra
    return subprocess.run(command, cwd=root, env=env, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

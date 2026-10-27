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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("check", choices=("build", "test", "replay", "bench", "proof", "node-proof", "node-replay", "node-bench"))
    parser.add_argument("extra", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    installed = Path.home() / ".local/share/raft-verus"
    verus = Path(os.environ.get("RAFT_VERUS", installed / "verus-x86-linux/verus"))
    rustup = Path(os.environ.get("RAFT_RUSTUP_HOME", installed / "rustup"))
    rust = rustup / "toolchains/1.92.0-x86_64-unknown-linux-gnu/bin"
    required = [verus, verus.parent / "libvstd.rlib", rust / "cargo", rust / "rustc"]
    for path in required:
        if not path.is_file():
            print(f"BLOCKED: missing local tool or dependency: {path}", file=sys.stderr)
            return 2
    env = os.environ.copy()
    env["RUSTUP_HOME"] = str(rustup)
    env["RUSTC"] = str(rust / "rustc")
    env["RUSTDOC"] = str(rust / "rustdoc")
    flags = ["--extern", f"vstd={verus.parent / 'libvstd.rlib'}",
             "-L", f"dependency={verus.parent}", "--check-cfg",
             "cfg(verus_keep_ghost,verus_keep_ghost_body,verus_verify_core,verus_no_vstd)"]
    # Use structured flag boundaries; the runner never invokes a shell.
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env["CARGO_ENCODED_RUSTDOCFLAGS"] = "\x1f".join(flags)
    env.pop("RUSTFLAGS", None)
    if args.check in {"proof", "node-proof"}:
        command = [str(verus), "--crate-type", "lib", "--no-cheating", "--triggers-mode", "silent",
                   "crates/raft-proofs/" + ("node_init.rs" if args.check == "node-proof" else "reconcile.rs")]
    else:
        command = [str(rust / "cargo")]
        if args.check == "build":
            command += ["check", "--workspace", "--locked", "--offline"]
        elif args.check == "test":
            command += ["test", "--workspace", "--locked", "--offline"]
        else:
            command += ["run", "--locked", "--offline", "-p", "raft-verified", "--example"]
            command += [{"replay": "replay", "bench": "measure_reconcile",
                         "node-replay": "initialize", "node-bench": "initialize"}[args.check]]
            if args.check.endswith("bench"):
                command += ["--release"]
            if args.check == "node-bench":
                command += ["--", "--measure"]
        if args.check.endswith("bench"):
            env.setdefault("RAFT_CODE_VERSION", "DRAFT workspace; see independently bound check evidence")
    command += args.extra
    return subprocess.run(command, cwd=root, env=env, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

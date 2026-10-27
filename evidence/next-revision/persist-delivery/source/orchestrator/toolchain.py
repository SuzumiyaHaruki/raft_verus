"""One DRAFT tool selection shared by Cargo, Verus and the independent runner."""
from pathlib import Path
import os


def resolve(settings: dict, environ=None) -> dict:
    env = os.environ if environ is None else environ
    base = Path(env.get("RAFT_TOOLCHAIN_ROOT", settings["install_root"])).expanduser().resolve()
    verus = base / "verus-x86-linux"
    rustup = base / "rustup"
    rust = rustup / "toolchains" / settings["rust_for_verus"] / "bin"
    paths = {"verus": verus / "verus", "vstd": verus / "libvstd.rlib", "z3": verus / "z3",
             "rustc": rust / "rustc", "cargo": rust / "cargo", "rustdoc": rust / "rustdoc"}
    return {"paths": {k: str(v) for k, v in paths.items()},
            "missing": [str(p) for p in paths.values() if not p.is_file()],
            "environment": {"RAFT_TOOLCHAIN_ROOT": str(base), "RUSTUP_HOME": str(rustup),
                            "CARGO_HOME": str(base / "cargo")}}

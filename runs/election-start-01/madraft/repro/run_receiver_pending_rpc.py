"""Run a supplemental diagnostic against unmodified actual host/core sources."""
from pathlib import Path
import configparser
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
config = configparser.ConfigParser()
config.read(root / "tools.ini")
deps = root / "runs/checks/cargo-target/release/deps"
vstd = Path(config["tools"]["verus"]).parent
with tempfile.TemporaryDirectory(prefix="receiver-pending-rpc-") as temporary:
    binary = Path(temporary) / "receiver_pending_rpc"
    command = [str(Path(config["tools"]["rust_bin"]) / "rustc"),
               "--edition=2021", "--cfg", "madsim", "-C", "opt-level=1",
               str(root / "madraft/repro/receiver_pending_rpc.rs"),
               "--extern", f"vstd={vstd / 'libvstd.rlib'}",
               "-L", f"dependency={vstd}", "-L", f"dependency={deps}",
               "-o", str(binary)]
    for name in ("log", "madsim", "serde", "futures", "bincode", "thiserror", "async_channel"):
        library = sorted(deps.glob(f"lib{name}-*.rlib"))[0]
        command += ["--extern", f"{name}={library}"]
    subprocess.run(command, check=True)
    subprocess.run([str(binary)], check=True, timeout=60)

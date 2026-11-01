"""Compile the channel diagnostic using this workspace's offline toolchain."""
from pathlib import Path
import configparser
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
config = configparser.ConfigParser()
config.read(root / "tools.ini")
deps = root / "runs/checks/cargo-target/release/deps"
libraries = sorted(deps.glob("libasync_channel-*.rlib"))
if not libraries:
    raise SystemExit("First run the election-start obligation or final check to build offline dependencies.")
with tempfile.TemporaryDirectory(prefix="election-observer-") as temporary:
    for name in ("closed_observer", "cancelled_observer"):
        binary = Path(temporary) / name
        command = [
            str(Path(config["tools"]["rust_bin"]) / "rustc"), "--edition=2021", "--cfg", "madsim",
            str(root / f"madraft/repro/{name}.rs"),
            "--extern", f"async_channel={libraries[0]}", "-L", f"dependency={deps}",
            "-o", str(binary),
        ]
        if name == "cancelled_observer":
            madsim = sorted(deps.glob("libmadsim-*.rlib"))[0]
            command += ["--extern", f"madsim={madsim}"]
        subprocess.run(command, check=True)
        subprocess.run([str(binary)], check=True)

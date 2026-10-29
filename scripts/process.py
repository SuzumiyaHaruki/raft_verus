"""Bounded POSIX subprocess transport. This is not an OS sandbox."""
from __future__ import annotations

import os
import signal
import subprocess
import sys
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping, Sequence


@dataclass
class ProcessResult:
    argv: list[str]
    status: str
    exit_code: int | None
    stdout: str
    stderr: str
    stdout_truncated: bool = False
    stderr_truncated: bool = False
    elapsed_seconds: float = 0.0


def run_process(argv: Sequence[str], cwd: str | Path, stdin: str = "", *,
                timeout: float = 120, max_log_bytes: int = 1_048_576,
                cancel_event: threading.Event | None = None,
                env: Mapping[str, str] | None = None,
                output_paths: tuple[Path, Path] | None = None,
                on_start=None) -> ProcessResult:
    """Drain both pipes concurrently, cap each stream, kill/reap the process group."""
    args = list(argv)
    if not args or any(not isinstance(a, str) for a in args):
        raise ValueError("argv must be a nonempty string array")
    if timeout <= 0 or max_log_bytes <= 0:
        raise ValueError("timeout and max_log_bytes must be positive")
    if os.name != "posix":
        return ProcessResult(args, "BLOCKED", None, "", "POSIX process groups required")
    started = time.monotonic()
    try:
        proc = subprocess.Popen(args, cwd=cwd, stdin=subprocess.PIPE,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                start_new_session=True, env=env, shell=False)
    except (FileNotFoundError, PermissionError, OSError) as exc:
        return ProcessResult(args, "BLOCKED", None, "", str(exc))
    # A separate watcher survives a SIGKILL of the controller. EOF on this
    # private pipe kills the worker's process group, including its children.
    reader, writer = os.pipe()
    try:
        watcher = subprocess.Popen(
            [sys.executable, str(Path(__file__).resolve()), '--watch', str(reader), str(proc.pid)],
            pass_fds=(reader,), stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL, start_new_session=True)
    except BaseException:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait()
        for fd in (reader, writer):
            os.close(fd)
        for pipe in (proc.stdin, proc.stdout, proc.stderr):
            pipe.close()
        raise
    os.close(reader)
    if on_start:
        try:
            on_start(proc.pid)
        except BaseException:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait()
            watcher.terminate()
            watcher.wait()
            os.close(writer)
            for pipe in (proc.stdin, proc.stdout, proc.stderr):
                pipe.close()
            raise
    buffers = [bytearray(), bytearray()]
    truncated = [False, False]
    log_limit = threading.Event()

    def consume(pipe, index):
        sink = output_paths[index].open("wb") if output_paths else None
        try:
            while True:
                data = pipe.read(65536)
                if not data:
                    break
                room = max_log_bytes - len(buffers[index])
                retained = data[:max(0, room)]
                buffers[index].extend(retained)
                if sink:
                    sink.write(retained)
                    sink.flush()
                if len(data) > room:
                    truncated[index] = True
                    log_limit.set()
        finally:
            pipe.close()
            if sink:
                sink.close()

    def feed():
        try:
            proc.stdin.write(stdin.encode("utf-8"))
            proc.stdin.flush()
        except (BrokenPipeError, OSError):
            pass
        finally:
            proc.stdin.close()

    threads = [threading.Thread(target=consume, args=(proc.stdout, 0), daemon=True),
               threading.Thread(target=consume, args=(proc.stderr, 1), daemon=True),
               threading.Thread(target=feed, daemon=True)]
    for thread in threads:
        thread.start()
    status = "COMPLETED"

    def kill_group(sig):
        try:
            os.killpg(proc.pid, sig)
        except ProcessLookupError:
            pass

    try:
        while proc.poll() is None:
            if cancel_event is not None and cancel_event.is_set():
                status = "CANCELLED"
                break
            if log_limit.is_set():
                status = "LOG_LIMIT"
                break
            if time.monotonic() - started >= timeout:
                status = "TIMEOUT"
                break
            time.sleep(0.01)
    except KeyboardInterrupt:
        status = "CANCELLED"
    finally:
        # Descendants may retain pipes after the immediate child exits.
        kill_group(signal.SIGTERM)
        try:
            proc.wait(timeout=0.2)
        except subprocess.TimeoutExpired:
            pass
        kill_group(signal.SIGKILL)
        proc.wait()
        for thread in threads:
            thread.join(timeout=1)
        watcher.terminate()
        watcher.wait()
        os.close(writer)
    if log_limit.is_set() and status == "COMPLETED":
        status = "LOG_LIMIT"
    return ProcessResult(args, status, proc.returncode,
                         buffers[0].decode("utf-8", errors="replace"),
                         buffers[1].decode("utf-8", errors="replace"),
                         *truncated, time.monotonic() - started)


if __name__ == '__main__' and len(sys.argv) == 4 and sys.argv[1] == '--watch':
    try:
        os.read(int(sys.argv[2]), 1)
        os.killpg(int(sys.argv[3]), signal.SIGKILL)
    except ProcessLookupError:
        pass

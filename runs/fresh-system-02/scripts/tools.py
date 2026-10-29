"""Local tool configuration; no installation or global configuration changes."""
import os
from pathlib import Path
import configparser

ROOT = Path(__file__).resolve().parents[1]


def settings(root=ROOT):
    cfg = configparser.ConfigParser(interpolation=None)
    cfg.read(root / 'tools.ini')
    return cfg


def environment(root=ROOT):
    cfg = settings(root)['tools']
    env = os.environ.copy()
    for key in ('verus', 'rust_bin', 'rustup_home', 'cargo_home'):
        value = os.environ.get('MADRAFT_' + key.upper(), cfg[key])
        cfg[key] = str((root / Path(value).expanduser()).resolve())
    rust = Path(cfg['rust_bin'])
    env.update(RUSTC=str(rust / 'rustc'), RUSTDOC=str(rust / 'rustdoc'),
               RUSTUP_HOME=cfg['rustup_home'], CARGO_HOME=cfg['cargo_home'])
    env['PATH'] = str(rust) + os.pathsep + env.get('PATH', '')
    vstd = Path(cfg['verus']).parent
    flags = ['--cfg', 'madsim', '--extern', f'vstd={vstd / "libvstd.rlib"}',
             '-L', f'dependency={vstd}', '--check-cfg',
             'cfg(verus_keep_ghost,verus_keep_ghost_body,verus_verify_core,verus_no_vstd)']
    env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join(flags)
    env['CARGO_ENCODED_RUSTDOCFLAGS'] = env['CARGO_ENCODED_RUSTFLAGS']
    env.pop('RUSTFLAGS', None)
    env.pop('RUSTDOCFLAGS', None)
    # The checker selects its own target directory and seed repetitions. Never
    # inherit another experiment's compiled protocol or a compiler wrapper.
    for key in ('CARGO_TARGET_DIR', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                'CARGO_BUILD_RUSTC_WRAPPER', 'CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER',
                'MADSIM_TEST_NUM'):
        env.pop(key, None)
    return cfg, env

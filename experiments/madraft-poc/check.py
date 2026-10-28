#!/usr/bin/env python3
"""DRAFT real local checks; no generation, downloads by default or global changes."""
import argparse
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT))
from orchestrator.config import load_config


def cargo_command(root, action, cache_root=ROOT):
    toolchain=load_config(root, os.environ.get('RAFT_WORKFLOW_CONFIG', 'workflow.toml'))['toolchain']
    selected=toolchain['resolved']
    if selected['missing']:
        raise FileNotFoundError('BLOCKED: '+', '.join(selected['missing']))
    helper=runpy.run_path(str(root/'crates/raft-verified/check.py'))['cargo_environment']
    env=helper(selected, ['--cfg','madsim','--check-cfg','cfg(madsim)'])
    env['CARGO_NET_GIT_FETCH_WITH_CLI']='true'
    env.setdefault('CARGO_TARGET_DIR',str(cache_root/'target/madraft-poc'))
    supplied_cache=Path(toolchain.get('madsim_cargo_home',cache_root/'target/madraft-cargo')).expanduser().resolve()
    # Candidate copies omit target; stage dependencies in the existing writable build scratch.
    cargo_home=Path(env['CARGO_TARGET_DIR'])/'cargo-home'
    if action=='fetch':
        cargo_home=supplied_cache
    elif not cargo_home.exists():
        if not supplied_cache.is_dir():
            raise FileNotFoundError('BLOCKED: missing offline MadSim cache: '+str(supplied_cache))
        shutil.copytree(supplied_cache,cargo_home)
    env['CARGO_HOME']=str(cargo_home)
    manifest=str(root/'experiments/madraft-poc/Cargo.toml')
    args=[selected['paths']['cargo'], {'build':'check','fetch':'fetch','replay':'run'}.get(action,'test'), '--manifest-path',manifest,'--locked']
    if action!='fetch':args+=['--offline']
    if action=='replay':args+=['--bin',action]
    elif action not in ('build','fetch'):
        args+=['--test','cluster' if action in ('cluster','election','replication','recovery') else 'scenarios']
        if action=='election':args+=['election_']
        if action=='replication':args+=['partition_replacement']
        if action=='recovery':args+=['restart_']
        if action=='fault':args+=['fault_']
        if action=='barrier':args+=['persistence_barrier']
        args+=['--','--nocapture','--test-threads=1']
    return args,env


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('check',choices=['build','runtime','fault','barrier','fetch','cluster','election','replication','recovery','replay'])
    a=p.parse_args()
    try:cmd,env=cargo_command(ROOT,a.check)
    except FileNotFoundError as e:print(str(e),file=sys.stderr);return 2
    selected=load_config(ROOT, os.environ.get('RAFT_WORKFLOW_CONFIG', 'workflow.toml'))['toolchain']['resolved']
    for tool,flag in [('rustc','-Vv'),('cargo','-V')]:
        subprocess.run([selected['paths'][tool],flag],env=env,check=True)
    print('Actual command: '+json.dumps(cmd),flush=True)
    return subprocess.run(cmd,cwd=ROOT,env=env).returncode

if __name__=='__main__':raise SystemExit(main())

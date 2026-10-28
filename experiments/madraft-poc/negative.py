#!/usr/bin/env python3
"""DRAFT: SIMULATED source mutation, independently checked by actual Rust and Verus."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT))
from orchestrator.config import load_config,profile_by_id
from orchestrator.verifier_runner import verify_profile
from orchestrator.workspace import create_candidate


def main():
    output=Path(os.environ.get('CARGO_TARGET_DIR',str(ROOT/'target/madraft-negative'))).resolve().parent/'negative-control'
    output.mkdir(parents=True,exist_ok=True)
    spec=importlib.util.spec_from_file_location('poc_check',ROOT/'experiments/madraft-poc/check.py')
    helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
    rel=Path('crates/raft-verified/src/madraft.rs');original=(ROOT/rel).read_text()
    anchor='            Ok(VoteAction::Persist { operation: op, retry: higher })'
    assert original.count(anchor)==1,'mutation anchor changed'
    wrong=original.replace(anchor,'''            // SIMULATED DEFECT: release a grant before durable completion.
            if grant { return Ok(VoteAction::Reply(Message { from: self.state.id, to: m.from,
                term: self.state.hard.term, rpc: m.rpc, body: MessageBody::VoteResponse { granted: true } })); }
'''+anchor)
    records=[]
    with tempfile.TemporaryDirectory(prefix='raft-madraft-negative-') as tmp:
        candidate=create_candidate(ROOT,Path(tmp)/'candidate')
        cfg=load_config(candidate)
        for phase,text in [('defect',wrong),('repair',original)]:
            (candidate/rel).write_text(text)
            # These are fixture edits, never a claim of real model generation.
            (output/(phase+'.rs')).write_text(text)
            cmd,env=helper.cargo_command(candidate,'barrier',ROOT)
            env['CARGO_TARGET_DIR']=str(output/'build')
            with (output/(phase+'.stdout.log')).open('w') as out,(output/(phase+'.stderr.log')).open('w') as err:
                p=subprocess.run(cmd,cwd=candidate,env=env,stdout=out,stderr=err,timeout=180)
            stdout=(output/(phase+'.stdout.log')).read_text()
            if phase=='defect':assert p.returncode!=0 and 'test result: FAILED' in stdout and 'persistence_barrier' in stdout,stdout
            else:assert p.returncode==0 and '1 passed' in stdout,stdout
            proof=verify_profile(candidate,profile_by_id(cfg,'madraft-poc-verus'),output/phase)
            expected='FAIL' if phase=='defect' else 'PASS'
            assert proof['result']==expected,proof
            functions=proof.get('function_results',{})
            assert any('handle_request_vote' in name and passed==(phase=='repair') for name,passed in functions.items()),proof
            records.append({'phase':phase,'generation_kind':'SIMULATED','check_evidence_kind':'REAL_LOCAL',
                'runtime_argv':cmd,'runtime_exit_code':p.returncode,'proof':proof})
    assert (ROOT/rel).read_text()==original,'original source must remain untouched'
    (output/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
    print('NEGATIVE_CONTROL_PASS selected=4 generation=SIMULATED checking=REAL_LOCAL artifacts='+str(output))
    return 0

if __name__=='__main__':raise SystemExit(main())

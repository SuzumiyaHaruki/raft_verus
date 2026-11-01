#!/usr/bin/env python3
"""Real maintenance checks only. Never executes Codex or creates a correct voter."""
import argparse
import difflib
import json
from pathlib import Path
import re
import shlex
import shutil
import sys
import time

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from inputs import copy_files, differences, files_under, input_files, save_json
from process import run_process
from tools import environment
from request_vote import prepare, TEST_FILTER
import check

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'review/request_vote'/str(time.time_ns()))
    args=parser.parse_args()
    out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    work=ROOT/'target/request-vote-maintenance'/out.name
    work.mkdir(parents=True,exist_ok=False)
    cfg,env=environment(ROOT)
    results={}
    def run(name,argv,cwd=ROOT,extra=None,timeout=120):
        e=env.copy();e.update(extra or {})
        r=run_process([str(a) for a in argv],cwd,env=e,timeout=timeout,max_log_bytes=16*1024*1024)
        (out/(name+'.log')).write_text('$ '+shlex.join([str(a) for a in argv])+'\n'+r.stdout+r.stderr+
            f'\nprocess={r.status} exit={r.exit_code}\n')
        results[name]=dict(command=[str(a) for a in argv],cwd=str(cwd),process=r.status,exit=r.exit_code,
            seconds=r.elapsed_seconds,log=str(out/(name+'.log')),environment={k:e[k] for k in
                ('RUSTC','RUSTUP_HOME','CARGO_HOME','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR','MADSIM_TEST_SEED') if k in e})
        print(name,r.status,r.exit_code,flush=True)
        save_json(out/'results.json',results)
        return r
    def require(condition,message):
        if not condition: raise RuntimeError(message)
    for name,argv in [('rustc',[cfg['rust_bin']+'/rustc','--version']),('cargo',[cfg['rust_bin']+'/cargo','--version']),
                      ('verus',[cfg['verus'],'--version'])]:
        r=run('version-'+name,argv); require(r.status=='COMPLETED' and r.exit_code==0,'tool unavailable: '+name)
    # Store exact authored inputs (no build directories, old runs or credentials).
    names=input_files(ROOT,'request-vote')+['scripts/run.py']+['tasks/request_vote/'+n for n in
        ('fixtures.rs','bad_receiver.rs','maintain.py')]
    copy_files(ROOT,out/'source-inputs',names)
    seed=work/'unfinished'
    copy_files(ROOT,seed,input_files(ROOT,'request-vote')); prepare(seed)
    resolved=check.settings(ROOT)
    for key in ('verus','rust_bin','rustup_home','cargo_home'):resolved['tools'][key]=cfg[key]
    with (seed/'tools.ini').open('w') as f:resolved.write(f)
    original=(ROOT/'madraft/src/raft/raft.rs').read_text().splitlines(True)
    prepared=(seed/'madraft/src/raft/raft.rs').read_text().splitlines(True)
    (out/'preparation.patch').write_text(''.join(difflib.unified_diff(original,prepared,'root/raft.rs','candidate/raft.rs')))
    r=run('fixed-definitions',[cfg['verus'],ROOT/'tasks/request_vote/api.rs','--crate-type','lib','--no-cheating'])
    require(r.exit_code==0 and 'verified, 0 errors' in r.stdout,'fixed definitions failed')
    for goal in ('history','confirmed','pending'):
        r=run('caller-typecheck-'+goal,[cfg['verus'],seed/'tasks/request_vote/caller.rs','--crate-type','lib','--cfg','rv_'+goal,'--no-verify'])
        results['caller-typecheck-'+goal]['meaning']='TYPECHECKED ONLY — no implementation or theorem proved'
        require(r.exit_code==0,'independent caller does not typecheck')
        r=run('conditional-caller-'+goal,[cfg['verus'],seed/'tasks/request_vote/caller.rs','--crate-type','lib',
            '--cfg','madsim','--cfg','rv_'+goal,'--verify-root'])
        results['conditional-caller-'+goal]['meaning']='CONDITIONAL HARNESS CHECK ONLY: candidate contracts/lemmas are unproved dependencies; never task acceptance'
        require(r.exit_code==0,'fixed caller cannot establish its goal even under declared candidate contracts')
    report=check.evaluate(seed,authority=seed,tool_root=ROOT,output=work/'independent',timeout=300,
        seeds=[1],task='request-vote',mode='final')
    # The seeded unreached precondition is an OPEN implementation, not a defect finding.
    for p in (work/'independent').glob('*.log'):shutil.copy2(p,out/p.name)
    save_json(out/'unfinished-results.json',report)
    require(report['checks']['build']['status']=='PASS','unfinished receiver must compile with original project')
    require(report['status']=='INCOMPLETE','unfinished receiver must be rejected')
    require(all(report['checks']['vote_'+g]['status']!='PASS' for g in ('rules','history','confirmed','pending')),'unfinished proof unexpectedly accepted')
    results['unfinished']=dict(status='REJECTED_UNIMPLEMENTED',meaning='Not a semantic mutation',report=str(out/'unfinished-results.json'))
    # Actual Rust compilation and actual Verus rejection of fixed-property fixtures.
    flags=env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f')
    for name,diagnostic in [('deny','eligible grant required'),('vote','same-term vote must not change'),('recovery','restored vote must not disappear')]:
        binary=work/('bad-'+name)
        source=ROOT/'tasks/request_vote/fixtures.rs'
        r=run('fixture-'+name+'-compile',[cfg['rust_bin']+'/rustc',source,'--edition','2021','--test','--cfg','bad_'+name,*flags,'-o',binary])
        require(r.exit_code==0,'fixture did not compile: '+name)
        r=run('fixture-'+name+'-functional',[binary,'--nocapture'])
        require(r.status=='COMPLETED' and r.exit_code!=0 and diagnostic in r.stdout+r.stderr,'wrong functional failure: '+name)
        r=run('fixture-'+name+'-verus',[cfg['verus'],source,'--crate-type','lib','--cfg','bad_'+name,'--no-cheating'])
        require(r.status=='COMPLETED' and r.exit_code!=0 and re.search('postcondition not satisfied|assertion failed',r.stdout+r.stderr),'wrong Verus failure: '+name)
    # An obviously incorrect receiver exercises the REAL MadRaft host path.
    # It is not a successful candidate or a proof baseline; it always grants.
    for name in ('fake-save','ignore-file','double-vote-after-pending-restart'):
        candidate=work/name
        copy_files(seed,candidate,files_under(seed))
        shutil.copy2(ROOT/'tasks/request_vote/bad_receiver.rs',candidate/'madraft/src/raft/vote_core.rs')
        host=candidate/'tasks/request_vote/host.rs'
        before=host.read_text()
        if name=='fake-save':
            after=before.replace('    checkpoint(h,w,false).await?;', '    return Ok(()); // DEFECT: no actual write\n    checkpoint(h,w,false).await?;')
            test='rpc_rules_and_file';diagnostic='authorized grant must be in real file'
        elif name=='ignore-file':
            after=before.replace('Ok(bytes) => Core::recover(config,decode(&bytes)?),',
                'Ok(_bytes) => Core::new(config, Initial { state:Persistent{term:0,voted_for:None,log:Log{term:0,index:0}},role:Role::Follower }), // DEFECT: ignores retained file')
            test='restart_confirmed';diagnostic='restore vote from file'
        else:
            after=before
            test='restart_written_pending';diagnostic='recovery must not forget vote'
        require(before!=after or name=='double-vote-after-pending-restart','host fixture patch did not apply')
        host.write_text(after)
        (out/(name+'.patch')).write_text(''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),'host.rs','bad-host.rs')))
        extra={'CARGO_TARGET_DIR':str(work/(name+'-target')),'MADSIM_TEST_SEED':'1','MADSIM_TEST_NUM':'1'}
        base=[cfg['rust_bin']+'/cargo','test','--manifest-path',candidate/'madraft/Cargo.toml','--offline','--locked','--release','--lib']
        r=run(name+'-compile',base+['--no-run'],candidate,extra,timeout=180)
        require(r.exit_code==0,'host fixture compilation failed')
        r=run(name+'-integration',base+[TEST_FILTER+test,'--','--exact','--nocapture'],candidate,extra)
        output=r.stdout+r.stderr
        require(r.status=='COMPLETED' and r.exit_code!=0 and diagnostic in output and 'running 1 test' in output,'host fixture failed for wrong reason')
        results[name+'-integration']['meaning']='REAL RPC/file rejection of an intentionally wrong fixture; no core-proof PASS claimed'
    r=run('system-regression',[sys.executable,'-m','unittest','discover','-s','scripts/tests','-v'],timeout=180)
    require(r.exit_code==0,'system regression failed')
    # Recheck the unmodified definitions, type/interface and unfinished seed after negatives.
    r=run('definitions-recheck',[cfg['verus'],ROOT/'tasks/request_vote/api.rs','--crate-type','lib','--no-cheating'])
    require(r.exit_code==0,'fixed definition recheck failed')
    results['summary']=dict(status='MAINTENANCE_CHECKS_PASSED',generation='NOT_RUN',task='NOT_COMPLETED',
        full_madraft='INCOMPLETE',global_safety='ALL_OPEN',core_proof_for_host_fixtures='NOT_ESTABLISHED',
        note='Simulated controller regressions are explicitly labeled in the unittest sources; Rust/Verus fixture commands above are real.')
    save_json(out/'results.json',results)
    print('MAINTENANCE_CHECKS_PASSED; generation NOT_RUN; evidence:',out)

if __name__=='__main__':main()

#!/usr/bin/env python3
"""Real local maintenance, never Codex or a completed Raft solution."""
import argparse
import difflib
from pathlib import Path
import re
import shlex
import shutil
import sys
import time

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
import check
import election_start as task
import request_vote
from inputs import copy_files,files_under,fingerprint,git_revision,input_files,save_json
from process import run_process
from tools import environment,settings


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'review/election_start'/str(time.time_ns()))
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    work=ROOT/'target/election-start-maintenance'/out.name;work.mkdir(parents=True,exist_ok=False)
    cfg,env=environment(ROOT);results={}
    def require(ok,message):
        if not ok:raise RuntimeError(message)
    def run(name,argv,cwd=ROOT,extra=None,timeout=120,meaning=None):
        argv=[str(a) for a in argv];e=env.copy();e.update(extra or {})
        r=run_process(argv,cwd,env=e,timeout=timeout,max_log_bytes=16*1024*1024)
        log=out/(name+'.log');log.write_text('$ '+shlex.join(argv)+'\n'+r.stdout+r.stderr+f'\nprocess={r.status} exit={r.exit_code}\n')
        results[name]=dict(command=argv,cwd=str(cwd),process=r.status,exit=r.exit_code,seconds=r.elapsed_seconds,
            log=str(log),meaning=meaning,environment={k:e[k] for k in ('RUSTC','RUSTUP_HOME','CARGO_HOME',
            'CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR','MADSIM_TEST_SEED') if k in e})
        save_json(out/'results.json',results);print(name,r.status,r.exit_code,flush=True);return r
    def prepare(scope,path):
        copy_files(ROOT,path,input_files(ROOT,scope))
        module=task if scope=='election-start' else request_vote;module.prepare(path)
        ini=settings(ROOT)
        for key in ('verus','rust_bin','rustup_home','cargo_home'):ini['tools'][key]=cfg[key]
        with (path/'tools.ini').open('w') as f:ini.write(f)
    def proof_command(seed,goal):
        argv=[cfg['verus'],seed/'tasks/election_start/caller.rs','--crate-type','lib','--cfg','madsim','--no-cheating']
        if goal!='rules':argv+=['--cfg','es_'+goal]
        return argv
    names=sorted(set(input_files(ROOT,'election-start')+input_files(ROOT,'request-vote')+
        ['scripts/run.py','scripts/tests/test_election_start.py','scripts/tests/test_request_vote.py']+
        ['tasks/election_start/'+n for n in ('maintain.py','fixtures.rs','bad_core.rs')]))
    copy_files(ROOT,out/'source-inputs',names)
    save_json(out/'inputs.json',dict(git_revision=git_revision(ROOT),reference='623219a314cd59b85d3a81ae97079a8b665228f3',
        input_version=task.VERSION,files=names,input_fingerprint=fingerprint({n:ROOT/n for n in names}),
        model=settings(ROOT)['agent']['model'],effort=settings(ROOT)['agent']['reasoning_effort'],
        total_seconds=settings(ROOT).getfloat('agent','total_seconds')))
    for name,argv in [('rustc',[cfg['rust_bin']+'/rustc','--version']),('cargo',[cfg['rust_bin']+'/cargo','--version']),
                      ('verus',[cfg['verus'],'--version'])]:
        r=run('version-'+name,argv);require(r.status=='COMPLETED' and r.exit_code==0,'BLOCKED: unavailable '+name)
    seed=work/'unfinished';prepare('election-start',seed)
    original=(ROOT/'madraft/src/raft/raft.rs').read_text();prepared=(seed/'madraft/src/raft/raft.rs').read_text()
    (out/'preparation.patch').write_text(''.join(difflib.unified_diff(original.splitlines(True),prepared.splitlines(True),'root/raft.rs','candidate/raft.rs')))
    r=run('fixed-definitions',[cfg['verus'],ROOT/'tasks/election_start/api.rs','--crate-type','lib','--no-cheating'])
    require(r.exit_code==0 and 'verified, 0 errors' in r.stdout,'fixed definitions failed')
    for goal in task.GOALS:
        argv=proof_command(seed,goal);argv.remove('--no-cheating')
        r=run('typecheck-'+goal,argv+['--no-verify'],meaning='TYPECHECKED ONLY; zero implementation proof credit')
        require(r.exit_code==0,'caller does not typecheck: '+goal)
        r=run('conditional-'+goal,argv+['--verify-root'],meaning='CONDITIONAL CALLER ONLY; all candidate contracts/lemmas remain unproved; never task acceptance')
        require(r.exit_code==0,'caller cannot close under declared candidate contracts: '+goal)
    for scope,source in [('election-start',seed),('request-vote',work/'old-receiver-input')]:
        if scope=='request-vote':prepare(scope,source)
        output=work/scope/'independent'
        report=check.evaluate(source,authority=source,tool_root=ROOT,output=output,timeout=300,seeds=[1],task=scope,mode='final')
        logs=out/scope;logs.mkdir()
        for p in output.glob('*.log'):shutil.copy2(p,logs/p.name)
        save_json(logs/'results.json',report)
        require(report['checks']['build']['status']=='PASS','unfinished '+scope+' must compile')
        require(report['checks']['inventory']['status']=='PASS','missing fixed tests for '+scope)
        require(report['status']=='INCOMPLETE','unfinished '+scope+' unexpectedly accepted')
        module=task if scope=='election-start' else request_vote
        require(all(report['checks'][module.PREFIX+g]['reason']=='verification_failure' for g in module.GOALS),
            'unfinished entries must remain actual verification failures, not compilation/missing-tool failures')
        results[scope+'-seed']=dict(status='REJECTED_UNIMPLEMENTED',meaning='Unfinished seed, not a semantic mutation',report=str(logs/'results.json'))
        print(scope,'UNFINISHED_REJECTED',flush=True)
    flags=env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f')
    cases={'term':'must enter next term','split':'mixed history must include self vote','candidate':'must choose self',
        'request_term':'request must belong to new round','log':'request must keep actual log','early':'no unmatched send authorization',
        'recovery_vote':'restore retained self vote','recovery_next':'do not reuse retained operation ID'}
    for case,diagnostic in cases.items():
        source=ROOT/'tasks/election_start/fixtures.rs';binary=work/('wrong-'+case)
        r=run(case+'-compile',[cfg['rust_bin']+'/rustc',source,'--edition','2021','--test','--cfg','bad_'+case,*flags,'-o',binary])
        require(r.exit_code==0,'fixture failed to compile: '+case)
        r=run(case+'-functional',[binary,'--nocapture'])
        require(r.status=='COMPLETED' and r.exit_code!=0 and diagnostic in r.stdout+r.stderr,'wrong Rust rejection: '+case)
        r=run(case+'-verus',[cfg['verus'],source,'--crate-type','lib','--cfg','bad_'+case,'--no-cheating'])
        require(r.status=='COMPLETED' and r.exit_code!=0 and re.search('postcondition not satisfied|assertion failed',r.stdout+r.stderr)
            and not re.search(r'error\[E\d+\]|aborting due to .*previous errors',r.stderr),'wrong Verus rejection: '+case)
    # Wrong core in the ACTUAL MadRaft host. It is never a successful solution.
    host=work/'host-fixture';copy_files(seed,host,files_under(seed))
    shutil.copy2(ROOT/'tasks/election_start/bad_core.rs',host/'madraft/src/raft/vote_core.rs')
    core_before=(host/'madraft/src/raft/vote_core.rs').read_bytes()
    r=run('host-core-before',proof_command(host,'campaign'),meaning='Deliberately wrong core: semantic verification FAILURE expected, not a proof baseline')
    require(r.status=='COMPLETED' and r.exit_code!=0 and 'postcondition not satisfied' in r.stderr,'host fixture core must expose semantic failures')
    def integration(name,should_pass,case='rpc_campaign_delayed_save',diagnostic='no campaign RPC before save'):
        extra={'CARGO_TARGET_DIR':str(work/(name+'-build')),'MADSIM_TEST_SEED':'1','MADSIM_TEST_NUM':'1'}
        base=[cfg['rust_bin']+'/cargo','test','--manifest-path',host/'madraft/Cargo.toml','--locked','--offline','--release','--lib']
        r=run(name+'-compile',base+['--no-run'],host,extra,timeout=180);require(r.exit_code==0,'host fixture compile failed')
        r=run(name+'-rpc',base+[task.TEST_FILTER+case,'--','--exact','--nocapture'],host,extra,
            meaning='REAL three-node RPC test with an intentionally wrong one-round fixture, not completed Raft')
        text=r.stdout+r.stderr
        require(r.status=='COMPLETED' and 'running 1 test' in text,'host test did not run')
        require((r.exit_code==0) if should_pass else (r.exit_code!=0 and diagnostic in text),'unexpected host diagnostic')
    integration('host-wiring-control',True)
    actual=host/'madraft/src/raft/raft.rs';before=actual.read_text()
    pos=before.index('    pub async fn start_election(');head,tail=before[:pos],before[pos:]
    marker='        let saved = vote_host::save(write, &hooks).await;'
    require(marker in tail,'cannot locate actual campaign save')
    tail=tail.replace(marker,'''        // DEFECT: use prepared data before persistence/authorization.
        if let vote_api::Action::Send(send) = write.action { self.send_vote_request(send).await?; }
'''+marker,1)
    after=head+tail;actual.write_text(after)
    (out/'host-send-before-save.patch').write_text(''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),'host/raft.rs','bad-host/raft.rs')))
    integration('host-send-before-save',False)
    require((host/'madraft/src/raft/vote_core.rs').read_bytes()==core_before,'host mutation changed core')
    r=run('host-core-after',proof_command(host,'campaign'),meaning='Same wrong core, same semantic FAILURE; the host violation is detected by real RPC assertions')
    require(r.status=='COMPLETED' and r.exit_code!=0 and 'postcondition not satisfied' in r.stderr,'core result unexpectedly changed')
    actual.write_text(before);integration('host-wiring-recheck',True)
    # Exercise actual cancellation, file retention and fresh construction before
    # the bad fixture's reset next=0 is rejected. This is NOT successful recovery.
    for window in ('confirmed','written_pending'):
        integration('host-bad-recovery-'+window,False,'self_restart_'+window,'recover self vote and next from file')
    r=run('system-regression',[sys.executable,'-m','unittest','discover','-s','scripts/tests','-v'],timeout=240,
        meaning='Offline SIMULATED controller tests and existing real Rust/Verus support tests are identified in the test sources')
    require(r.exit_code==0,'system regression failed')
    r=run('definitions-recheck',[cfg['verus'],ROOT/'tasks/election_start/api.rs','--crate-type','lib','--no-cheating'])
    require(r.exit_code==0,'definition recheck failed')
    r=run('preview',[sys.executable,'scripts/run.py','new','runs/election-start-01','--task','election-start'])
    require(r.exit_code==0,'preview failed')
    results['summary']=dict(status='MAINTENANCE_CHECKS_PASSED',generation='NOT_RUN',local_goals='ALL_OPEN',
        full_madraft='INCOMPLETE',global_safety='ALL_OPEN',host='UNVERIFIED',
        note='Conditional callers and typechecking are not body proofs; wrong host-fixture core does not pass Verus. No old run is an input.')
    save_json(out/'results.json',results)
    print('MAINTENANCE_CHECKS_PASSED; generation NOT_RUN; evidence:',out,flush=True)


if __name__=='__main__':main()

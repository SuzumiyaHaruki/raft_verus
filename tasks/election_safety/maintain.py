#!/usr/bin/env python3
"""Real unfinished-input/observer/negative checks. Never invokes a generation agent."""
import argparse
import json
from pathlib import Path
import re
import shlex
import shutil
import sys
import time
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'scripts'))
import check
import election_safety as task
from inputs import copy_files,files_under,fingerprint,git_revision,input_files,save_json
from process import run_process
from tools import ROOT,environment,settings


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'review/election_safety'/str(time.time_ns()))
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    work=ROOT/'target/election-safety-maintenance'/out.name;work.mkdir(parents=True,exist_ok=False)
    cfg,env=environment(ROOT);results={}
    def require(ok,message):
        if not ok:raise RuntimeError(message)
    def run(name,argv,*,extra=None,timeout=120,meaning='REAL tool check'):
        argv=[str(a) for a in argv];e=env.copy();e.update(extra or {})
        r=run_process(argv,ROOT,env=e,timeout=timeout,max_log_bytes=16*1024*1024)
        log=out/(name+'.log');log.write_text('$ '+shlex.join(argv)+'\n'+r.stdout+r.stderr+f'\nprocess={r.status} exit={r.exit_code}\n')
        results[name]=dict(command=argv,process=r.status,exit=r.exit_code,seconds=r.elapsed_seconds,
            log=str(log),meaning=meaning,environment={k:e[k] for k in ('RUSTC','RUSTUP_HOME','CARGO_HOME','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_DIR','MADSIM_TEST_SEED') if k in e})
        save_json(out/'results.json',results);print(name,r.status,r.exit_code,flush=True);return r
    def verified(r):return r.status=='COMPLETED' and r.exit_code==0 and re.search(r'\d+ verified, 0 errors',r.stdout)
    names=input_files(ROOT,'election-safety')+['scripts/run.py','tasks/election_safety/maintain.py',
        'tasks/election_safety/fixtures.rs','scripts/tests/test_election_safety.py']
    save_json(out/'inputs.json',dict(git_revision=git_revision(ROOT),reference='81f5f7c96728ad184641eadf48f02c487137a09e',
        input_version=task.VERSION,files=names,input_fingerprint=fingerprint({n:ROOT/n for n in names}),
        agent_model=settings(ROOT)['agent']['model'],effort=settings(ROOT)['agent']['reasoning_effort'],
        budget=settings(ROOT).getint('agent','total_seconds'),generation_performed=False))
    try:
        for name,argv in [('rustc',[cfg['rust_bin']+'/rustc','--version']),('cargo',[cfg['rust_bin']+'/cargo','--version']),('verus',[cfg['verus'],'--version'])]:
            r=run('version-'+name,argv);require(r.status=='COMPLETED' and r.exit_code==0,'BLOCKED: unavailable '+name)
        seed=work/'unfinished';copy_files(ROOT,seed,input_files(ROOT,'election-safety'));task.prepare(seed)
        ini=settings(ROOT)
        for k in ('verus','rust_bin','rustup_home','cargo_home'):ini['tools'][k]=cfg[k]
        with (seed/'tools.ini').open('w') as f:ini.write(f)
        def proof(goal):
            argv=[cfg['verus'],seed/task.DIRECTORY/'caller.rs','--crate-type','lib','--cfg','madsim']
            return argv+(['--cfg','es_'+goal] if goal!='rules' else [])
        require(verified(run('definitions',[cfg['verus'],ROOT/task.DIRECTORY/'api.rs','--crate-type','lib','--no-cheating'])),'fixed definition failure')
        for goal in task.GOALS:
            r=run('typecheck-'+goal,proof(goal)+['--no-verify'],meaning='TYPECHECKED ONLY; no body-proof credit')
            require(r.status=='COMPLETED' and r.exit_code==0,'interface cannot typecheck: '+goal)
            r=run('conditional-'+goal,proof(goal)+['--verify-root'],meaning='CONDITIONAL caller; candidate contracts/lemmas UNPROVED; never acceptance')
            require(verified(r),'fixed caller cannot close under its unproved interfaces: '+goal)
        r=run('conditional-network',proof('provenance')+['--verify-module','vote_network','--no-cheating'],
            meaning='CONDITIONAL actual-call/record plumbing; candidate contracts unproved, no Raft invariant supplied')
        require(verified(r),'actual-call recording correspondence failure')
        # The independent checker really builds/tests/verifies the unfinished seed.
        output=work/'independent'
        report=check.evaluate(seed,authority=seed,tool_root=ROOT,output=output,timeout=300,
            seeds=[1],task='election-safety',mode='initial')
        raw=out/'unfinished';raw.mkdir()
        for p in output.glob('*.log'):shutil.copy2(p,raw/p.name)
        save_json(raw/'results.json',report)
        results['unfinished']=dict(status=report['status'],report=str(raw/'results.json'),meaning='Unfinished seed rejection, NOT a generated candidate')
        save_json(out/'results.json',results)
        require(report['checks']['build']['status']=='PASS','unfinished Rust input failed to compile')
        require(report['checks']['inventory']['status']=='PASS','fixed test inventory missing')
        require(report['status']=='INCOMPLETE','unfinished candidate must not be accepted')
        require(all(report['checks'][task.PREFIX+g]['reason']=='verification_failure' for g in task.GOALS),'expected semantic proof failures, not missing files/tools/compiler failures')
        # Obtain the exact Cargo-built test executable, with isolated build directory.
        extra={'CARGO_TARGET_DIR':str(work/'observer-build')}
        r=run('observer-build',[cfg['rust_bin']+'/cargo','test','--manifest-path',seed/'madraft/Cargo.toml',
            '--locked','--offline','--release','--lib','--no-run','--message-format=json'],extra=extra,timeout=180)
        require(r.exit_code==0,'observer build failed')
        artifacts=[json.loads(line) for line in r.stdout.splitlines() if line.startswith('{')]
        binary=next(x['executable'] for x in artifacts if x.get('executable'))
        cases=['observer_live_then_cancelled','observer_queued_reply_then_closed','observer_transport_error',
            'observer_panicked_producer','collection_observation_calibration','collection_queue_calibration','election_observation_calibration','promotion_schedule_calibration']
        for seed_number in check.configured_seeds(ROOT):
            for case in cases:
                r=run(f'observer-{seed_number}-{case}',[binary,task.TEST_FILTER+case,'--exact','--nocapture'],
                    extra={'MADSIM_TEST_SEED':str(seed_number),'MADSIM_TEST_NUM':'1'},meaning='REAL Rust/MadSim observation check; never a protocol-success result')
                require(r.exit_code==0 and '1 passed;' in r.stdout,'observation check failed: '+case)
        flags=env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f');fixture=ROOT/task.DIRECTORY/'fixtures.rs'
        cases={'bad_half':'half is not a strict majority','bad_unsaved':'unsaved self vote cannot authorize promotion',
            'bad_omission':'actual election was omitted','bad_erasure':'demotion erased election history','control':None}
        for case,diagnostic in cases.items():
            binary=work/('fixture-'+case)
            r=run(case+'-compile',[cfg['rust_bin']+'/rustc',fixture,'--edition','2021','--test','--cfg',case,*flags,'-o',binary])
            require(r.exit_code==0,'bad fixture must compile: '+case)
            r=run(case+'-rust',[binary,'--nocapture'],meaning='REAL intentionally wrong fragment' if diagnostic else 'REAL source-observation control; no protocol implementation')
            require(r.status=='COMPLETED' and ((r.exit_code!=0 and diagnostic in r.stdout+r.stderr) if diagnostic else r.exit_code==0),'unexpected fixture runtime result: '+case)
            selected=['--no-cheating']
            r=run(case+'-verus',[cfg['verus'],fixture,'--crate-type','lib','--cfg',case,*selected],
                meaning='REAL observation/specification control' if not diagnostic else 'REAL defective body proof; semantic FAILURE expected')
            require(verified(r) if not diagnostic else r.status=='COMPLETED' and r.exit_code!=0
                and re.search('postcondition not satisfied|assertion failed',r.stdout+r.stderr)
                and not re.search(r'error\[E\d+\]',r.stdout+r.stderr),'unexpected fixture proof diagnostic: '+case)
        r=run('system-regression',[sys.executable,'-m','unittest',*[p.stem for p in sorted((ROOT/'scripts/tests').glob('test_*.py')) if p.stem!='test_verification'],'-v'],
            extra={'PYTHONPATH':str(ROOT/'scripts/tests')},timeout=240,meaning='Offline system regressions: agent/checker results are SIMULATED; real tools are checked separately above')
        require(r.exit_code==0,'system regression failed')
        require(verified(run('definitions-recheck',[cfg['verus'],ROOT/task.DIRECTORY/'api.rs','--crate-type','lib','--no-cheating'])),'definition recheck failed')
        r=run('preview',[sys.executable,'scripts/run.py','new','runs/election-safety-01','--task','election-safety'])
        require(r.exit_code==0,'preview failed')
        results['summary']=dict(status='MAINTENANCE_CHECKS_PASSED',generation='NOT_RUN',candidate='UNIMPLEMENTED',
            local_proofs='OPEN',scoped_election_safety_no_restart='OPEN',cross_node_source='OPEN',global_safety='ALL_OPEN',full_madraft='INCOMPLETE',liveness='OPEN',host='UNVERIFIED')
    except Exception as e:
        results['summary']=dict(status='BLOCKED',reason=str(e),generation='NOT_RUN',local_proofs='OPEN',cross_node_source='OPEN')
        raise
    finally:save_json(out/'results.json',results)
    print('MAINTENANCE_CHECKS_PASSED; no generation; all candidate targets OPEN:',out,flush=True)

if __name__=='__main__':main()

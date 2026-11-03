"""Completed-write restart task, using the existing controller/checker."""
from request_vote import evaluate as evaluate_task, prepare as prepare_task
VERSION = 'madraft-election-recovery-1'
DIRECTORY = 'tasks/election_recovery'
HELPERS = ['scripts/request_vote.py', 'scripts/election_recovery.py']
INPUTS = ('AGENTS.md','TASK.md','start.md','continue.md','redesign.md','api.rs','caller.rs','network.rs',
          'observation.rs','promotion_tests.rs','core.rs','lemmas.rs','host.rs','driver.rs','prepare.py',
          'rpc_tests.rs','election_tests.rs','observer_tests.rs','collection_tests.rs',
          'recovery_network.rs','recovery_lemmas.rs','recovery_tests.rs')
TEST_FILES = ('rpc_tests.rs','election_tests.rs','observer_tests.rs','collection_tests.rs','promotion_tests.rs','recovery_tests.rs')
TEST_FILTER = 'raft::raft::vote_tests::'
GOALS = ('rules','campaign','authorization','history','reply_rules','self_vote','rounds','higher_reply',
         'provenance','nonempty','promotion','election_basis','election_history','election_nonempty',
         'receive_confirmed','receive_pending','self_confirmed','self_pending',
         'restart_source','restart_history','restart_nonempty')
PREFIX = 'recovery_'
CFG = 'es_'
SCOPE = 'election-recovery'
LABEL = 'ELECTION_SAFETY_COMPLETED_WRITE_RESTART_CHECKS_PASSED'
FUNCTIONS = ['try_become_leader','receive','start_election','complete','collect','vote_count','has_vote','recover']
BOUNDARY = ('Finite fixed-membership histories permit repeated node restarts only after a complete successful '
    'write, before or after its matching Core completion. Other nodes and old network messages continue. '
    'Truthful I/O, file retention, callback incarnation/cancellation, serialization and RPC forwarding are UNVERIFIED. '
    'Interrupted/failed-write recovery, full MadRaft, the other four properties and conditional liveness remain OPEN.')
UNCOVERED = ['interrupted/failed-write recovery','stale/corrupt retained storage','physical power-loss durability',
             'real log-to-summary correspondence','replication, commitment, application and snapshots',
             'complete Raft protocol and faults']
def prepare(authority):
    return prepare_task(authority,DIRECTORY)
def evaluate(checks,mode,seeds):
    from inputs import save_json
    report=evaluate_task(checks,mode,seeds)
    goals=report['local_goals']
    report['formula_status']={'no_restart_election_safety':goals['election_history'],'completed_write_restarts':goals['restart_history']}
    scoped='CHECKS_PASSED_REVIEW_REQUIRED' if report['status']==LABEL else 'OPEN'
    report['scoped_election_safety_no_restart']=scoped
    report['scoped_election_safety_completed_write_restarts']=scoped
    report['local_recovery_windows']={g:goals[g] for g in ('receive_confirmed','receive_pending','self_confirmed','self_pending')}
    report['arbitrary_crash_recovery']='OPEN'
    save_json(checks.output/'results.json',report)
    return report

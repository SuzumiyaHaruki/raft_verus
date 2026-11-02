"""One scoped extension; reuse the existing receiver checker and controller."""
from request_vote import evaluate as evaluate_task, prepare as prepare_task
VERSION = 'madraft-election-safety-1'
DIRECTORY = 'tasks/election_safety'
HELPERS = ['scripts/request_vote.py','scripts/election_safety.py']
INPUTS = ('AGENTS.md','TASK.md','start.md','continue.md','redesign.md','api.rs','caller.rs','network.rs',
          'observation.rs','promotion_tests.rs','core.rs','lemmas.rs','host.rs','driver.rs','prepare.py','rpc_tests.rs','election_tests.rs',
          'observer_tests.rs','collection_tests.rs')
TEST_FILES = ('rpc_tests.rs','election_tests.rs','observer_tests.rs','collection_tests.rs','promotion_tests.rs')
TEST_FILTER = 'raft::raft::vote_tests::'
GOALS = ('rules','campaign','authorization','history','reply_rules','self_vote','rounds','higher_reply',
         'provenance','nonempty','promotion','election_basis','election_history','election_nonempty','receive_confirmed','receive_pending','self_confirmed','self_pending')
PREFIX = 'election_'
CFG = 'es_'
SCOPE = 'election-safety'
LABEL = 'ELECTION_SAFETY_NO_RESTART_CHECKS_PASSED'
FUNCTIONS = ['try_become_leader','receive','start_election','complete','collect','vote_count','has_vote','recover']
BOUNDARY = ('Historical Election Safety uses the original targets::Event::Elect vocabulary for a single fixed group, '
    'finite executions from Followers, without node restart. Four local recovery windows are separate. '
    'RPC association, I/O/readback, serialization and forwarding remain UNVERIFIED. '
    'Complete Raft Election Safety, the other four properties, full MadRaft and conditional liveness remain OPEN.')
UNCOVERED = ['cross-node restart induction','arbitrary/interrupted/failed-write recovery',
    'real log-to-summary correspondence','replication and snapshots','complete Raft protocol and faults']
def prepare(authority):
    return prepare_task(authority,DIRECTORY)
def evaluate(checks,mode,seeds):
    from inputs import save_json
    report=evaluate_task(checks,mode,seeds)
    report['scoped_election_safety_no_restart'] = (
        'CHECKS_PASSED_REVIEW_REQUIRED' if report['status']==LABEL else 'OPEN')
    report['local_recovery_windows']={g:report['local_goals'][g] for g in
        ('receive_confirmed','receive_pending','self_confirmed','self_pending')}
    save_json(checks.output/'results.json',report)
    return report

"""One additional scoped task; reuse the receiver's actual checking loop."""
from request_vote import evaluate, prepare as prepare_task

VERSION = 'madraft-election-start-1'
DIRECTORY = 'tasks/election_start'
HELPERS = ['scripts/request_vote.py', 'scripts/election_start.py']
INPUTS = ('AGENTS.md','TASK.md','start.md','continue.md','redesign.md','api.rs','caller.rs',
          'core.rs','lemmas.rs','host.rs','driver.rs','rpc_tests.rs','election_tests.rs','prepare.py')
TEST_FILES = ('rpc_tests.rs','election_tests.rs')
TEST_FILTER = 'raft::raft::vote_tests::'
GOALS = ('rules','campaign','authorization','history','receive_confirmed','receive_pending',
         'self_confirmed','self_pending')
PREFIX = 'election_'
CFG = 'es_'
SCOPE = 'election-start'
LABEL = 'ELECTION_START_CHECKS_PASSED'
FUNCTIONS = ['receive','start_election','complete','recover']
BOUNDARY = ('Local choices include actual self-selections and affirmative replies; a self-choice is not I/O or majority evidence. '
            'Serialization, faithful I/O/readback, file retention and unchanged RPC forwarding remain unverified host premises. '
            'Only two bounded recovery windows are covered. All five global safety properties, full 2A and conditional liveness remain OPEN.')
UNCOVERED = ['arbitrary/interrupted-write crashes','failed-write recovery','repeated-crash induction',
             'real log-to-summary correspondence','reply provenance and higher reply terms',
             'candidate tally and majority','leader promotion','replication and snapshots']


def prepare(authority):
    return prepare_task(authority,DIRECTORY)

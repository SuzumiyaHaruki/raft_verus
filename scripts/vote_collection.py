"""One scoped extension; reuse the existing receiver checker and controller."""
from request_vote import evaluate, prepare as prepare_task
VERSION = 'madraft-vote-collection-1'
DIRECTORY = 'tasks/vote_collection'
HELPERS = ['scripts/request_vote.py','scripts/vote_collection.py']
INPUTS = ('AGENTS.md','TASK.md','start.md','continue.md','redesign.md','api.rs','caller.rs','network.rs',
          'core.rs','lemmas.rs','host.rs','driver.rs','prepare.py','rpc_tests.rs','election_tests.rs',
          'observer_tests.rs','collection_tests.rs')
TEST_FILES = ('rpc_tests.rs','election_tests.rs','observer_tests.rs','collection_tests.rs')
TEST_FILTER = 'raft::raft::vote_tests::'
GOALS = ('rules','campaign','authorization','history','reply_rules','self_vote','rounds','higher_reply',
         'provenance','nonempty','receive_confirmed','receive_pending','self_confirmed','self_pending')
PREFIX = 'collection_'
CFG = 'vc_'
SCOPE = 'vote-collection'
LABEL = 'VOTE_COLLECTION_CHECKS_PASSED'
FUNCTIONS = ['receive','start_election','complete','collect','vote_count','has_vote','recover']
BOUNDARY = ('Actual per-prefix vote provenance without node restart and separate bounded local recovery are required. '
    'These do not establish Election Safety. Host RPC association, serialization, truthful I/O/readback and forwarding '
    'remain UNVERIFIED. Majority election, all five global properties, full MadRaft and conditional liveness remain OPEN.')
UNCOVERED = ['leader promotion','majority intersection','arbitrary/interrupted/failed-write recovery',
             'cross-node restart induction','replicated log correspondence','replication and snapshots']
def prepare(authority):
    return prepare_task(authority,DIRECTORY)

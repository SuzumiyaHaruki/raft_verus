#!/usr/bin/env python3
"""Simulated verifier protocol, deliberately distinct from real Verus."""
import argparse
import json
from pathlib import Path
import sys
import time

p = argparse.ArgumentParser()
p.add_argument('--scenario', default='ok')
p.add_argument('--target', action='append', default=[])
p.add_argument('--touch')
a = p.parse_args()
if a.scenario == 'timeout':
    time.sleep(120)
if a.touch:
    Path(a.touch).write_text('modified during check')
if a.scenario == 'crash':
    print('fixture crash', file=sys.stderr)
    sys.exit(2)
if a.scenario == 'unsupported':
    print('unsupported output')
    sys.exit(0)
if a.scenario == 'empty':
    sys.exit(0)
print(json.dumps({'protocol': 'raft-check-v1',
                  'checked_targets': [] if a.scenario == 'no_targets' else a.target,
                  'verified': 0 if a.scenario in ('no_targets', 'fail') else len(a.target),
                  'errors': 1 if a.scenario == 'fail' else 0}))
sys.exit(1 if a.scenario == 'fail' else 0)

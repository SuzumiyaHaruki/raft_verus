#!/usr/bin/env python3
"""Text adapter fixture, not Verus; callers must label it SIMULATED."""
import argparse
import sys

if '--version' in sys.argv:
    print('verus FIXTURE 1')
    sys.exit(0)
p = argparse.ArgumentParser()
p.add_argument('--verified', type=int, default=1)
p.add_argument('--errors', type=int, default=0)
a = p.parse_args()
print(f'fixture summary: {a.verified} verified; {a.errors} errors')
sys.exit(1 if a.errors else 0)

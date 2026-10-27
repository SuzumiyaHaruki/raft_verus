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
p.add_argument('--compiler-error', action='store_true')
a = p.parse_args()
if a.compiler_error:
    print('error[E0425]: cannot find value; compiler fixture failure')
elif a.errors:
    print('postcondition not satisfied: wrong_increment at fixture.rs:3')
print(f'fixture summary: {a.verified} verified; {a.errors} errors')
sys.exit(1 if a.errors else 0)

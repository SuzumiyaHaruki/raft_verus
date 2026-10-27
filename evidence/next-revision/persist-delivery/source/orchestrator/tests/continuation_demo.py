#!/usr/bin/env python3
"""DRAFT offline controller examples with retained artifacts; never calls real Codex."""
import argparse
import json
from pathlib import Path
import sys
from types import SimpleNamespace
from unittest.mock import patch
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from orchestrator.tests.test_continuation import ContinuationTests
from orchestrator.tests.test_performance import PerformanceTests


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=False)
    cases = [
        (ContinuationTests, 'test_exhaustion_resume_and_explicit_extension'),
        (ContinuationTests, 'test_interrupt_after_launch_is_charged_and_candidate_retained'),
        (ContinuationTests, 'test_cross_task_regression_repaired_in_current_task_scope'),
        (ContinuationTests, 'test_regression_after_interruption_repairs_later_task'),
        (ContinuationTests, 'test_rewind_preserves_abandoned_patch_and_rechecks'),
        (PerformanceTests, 'test_optimization_patch_returns_through_correctness_before_measurement'),
    ]
    results = []
    for cls, name in cases:
        directory = destination / name.removeprefix('test_')
        directory.mkdir()
        case = cls(name)
        # Retain the exact test's workspace instead of TemporaryDirectory cleanup.
        with patch('orchestrator.tests.test_scheduler.tempfile.TemporaryDirectory',
                   return_value=SimpleNamespace(name=str(directory), cleanup=lambda: None)):
            case.setUp()
        try:
            getattr(case, name)()
            result = dict(case=name, status='PASS', artifacts=str(directory))
        except Exception as error:
            result = dict(case=name, status='FAIL', reason=str(error), artifacts=str(directory))
        finally:
            case.tearDown()
        results.append(result)
    summary = dict(status='PASS' if all(r['status'] == 'PASS' for r in results) else 'FAIL',
        generation_kind='SIMULATED', checking='Real Python fixture predicates; synthetic benchmark samples; no Rust/Verus or Raft proof claim',
        full_acceptance=False, cases=results)
    (destination/'summary.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary, indent=2))
    return 0 if summary['status'] == 'PASS' else 1


if __name__ == '__main__':
    raise SystemExit(main())

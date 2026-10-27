"""Optional DRAFT comparison of declared, repeated, comparable workloads."""
import json
import statistics
from pathlib import Path


def compare(measurements, rule, root):
    if not rule:
        return {"performance_verdict": "INCONCLUSIVE", "comparison_reason": "measurement only; no reference/rule configured"}
    if not rule.get("reference_candidate"):
        raise ValueError("comparison requires an identified reference candidate")
    path = Path(rule["reference_path"])
    if not path.is_absolute():
        path = root / path
    reference = json.loads(path.read_text())
    reference = reference.get("measurements", []) if isinstance(reference, dict) else reference
    threshold, noise = rule["max_regression_fraction"], rule["max_relative_dispersion"]
    minimum = rule.get("minimum_samples", 5)
    if not 0 <= threshold or not 0 <= noise or minimum < 2:
        raise ValueError("invalid explicit comparison rule")
    key_fields = rule.get("workload_fields", ["workload"])
    comparable = rule.get("comparable_fields", ["build_mode", "environment", "boundary"])
    def index(rows):
        result = {}
        for row in rows:
            key = tuple(json.dumps(row[f], sort_keys=True) for f in key_fields)
            if key in result:
                raise ValueError("duplicate workload; declare its dimensions in workload_fields")
            result[key] = row
        return result
    current, prior = index(measurements), index(reference)
    if not current or current.keys() != prior.keys():
        raise ValueError("comparison workloads differ or are missing")
    comparisons, inconclusive = [], False
    for key, row in current.items():
        old = prior[key]
        if any(row[f] != old[f] for f in comparable):
            raise ValueError("comparison environment/build/workload configuration mismatch")
        sample_key = next(k for k in ("samples_ns_per_operation", "samples_ns", "samples") if k in row)
        if sample_key not in old or (sample_key == "samples" and row["unit"] != old["unit"]):
            raise ValueError("comparison units mismatch")
        a, b = row[sample_key], old[sample_key]
        import math
        if any(type(x) not in (int, float) or not math.isfinite(x) or x <= 0 for x in a + b):
            raise ValueError("comparison requires finite positive samples")
        med_a, med_b = statistics.median(a), statistics.median(b)
        spread_a = (max(a) - min(a)) / med_a
        spread_b = (max(b) - min(b)) / med_b
        stable = min(len(a), len(b)) >= minimum and max(spread_a, spread_b) <= noise
        inconclusive |= not stable
        # Every repeated sample must exceed the reference range, not just one median.
        regression = stable and min(a) > max(b) * (1 + threshold)
        comparisons.append(dict(workload=list(key), statistic="median", ratio=med_a / med_b,
            samples=[len(a), len(b)], relative_range=[spread_a, spread_b],
            reference_median=med_b, candidate_median=med_a, regression=regression))
    verdict = "INCONCLUSIVE" if inconclusive else "REGRESSION" if any(c["regression"] for c in comparisons) else "NO_REGRESSION"
    return dict(performance_verdict=verdict, comparisons=comparisons,
                reference=dict(path=str(path), candidate=rule["reference_candidate"]), comparison_rule=rule)

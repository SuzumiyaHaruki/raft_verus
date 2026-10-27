//! DRAFT local helper replay. No network, timer, crash or durability simulation.
use raft_verified::{reconcile_log, Command, Entry, ReconcileError};
use std::{env, fs};

fn terms(text: &str) -> Result<Vec<u64>, String> {
    if text == "-" {
        return Ok(vec![]);
    }
    text.split(',')
        .map(|n| n.parse().map_err(|_| format!("invalid term: {n}")))
        .collect()
}

fn entries(terms: &[u64]) -> Vec<Entry> {
    terms
        .iter()
        .map(|&term| Entry {
            term,
            command: Command::Noop,
        })
        .collect()
}

/// Returns the actual per-event observations as JSON, including failures.
pub fn run_trace(trace: &str) -> Result<(bool, usize, String), String> {
    let mut seed = None;
    let mut log = None;
    let mut events = vec![];
    let mut passed = true;
    for raw in trace.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["seed", value] if seed.is_none() => {
                seed = Some(value.parse::<u64>().map_err(|_| "invalid seed")?);
            }
            ["initial", value] if log.is_none() => {
                log = Some(entries(&terms(value)?));
            }
            ["append", previous, incoming, "=>", expected] => {
                let previous = previous
                    .parse::<usize>()
                    .map_err(|_| "invalid predecessor")?;
                let incoming = terms(incoming)?;
                let current = log.as_mut().ok_or("initial log missing")?;
                let before: Vec<_> = current.iter().map(|e| e.term).collect();
                let result = reconcile_log(current, previous, entries(&incoming));
                let after: Vec<_> = current.iter().map(|e| e.term).collect();
                let okay = if *expected == "INVALID_INDEX" {
                    result == Err(ReconcileError::InvalidIndex) && before == after
                } else {
                    result == Ok(()) && after == terms(expected)?
                };
                passed &= okay;
                let expected_json = if *expected == "INVALID_INDEX" {
                    "\"INVALID_INDEX\"".to_owned()
                } else {
                    format!("{:?}", terms(expected)?)
                };
                events.push(format!(
                    "{{\"previous\":{previous},\"incoming\":{incoming:?},\"before\":{before:?},\"expected\":{expected_json},\"after\":{after:?},\"returned\":\"{result:?}\",\"matched\":{okay}}}"
                ));
            }
            _ => return Err(format!("invalid trace line: {line}")),
        }
    }
    let seed = seed.ok_or("seed missing")?;
    if log.is_none() || events.is_empty() {
        return Err("empty scenario".to_owned());
    }
    let count = events.len();
    let result = if passed { "PASS" } else { "FAIL" };
    Ok((passed, count, format!(
        "{{\"kind\":\"LOCAL_RECONCILIATION_REPLAY\",\"evidence_kind\":\"REAL_LOCAL\",\"result\":\"{result}\",\"seed\":{seed},\"selected_count\":{count},\"trace\":[{}]}}", events.join(",")
    )))
}

#[allow(dead_code)]
fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/raft-verified/examples/reconcile.trace".to_owned());
    let outcome = fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|s| run_trace(&s));
    match outcome {
        Ok((passed, count, report)) => {
            println!("{report}");
            if passed {
                println!("REPLAY_PASS steps={count}");
            } else {
                eprintln!("Replay mismatch; complete event and state observations are above. Source trace: {path}");
            }
            if !passed {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("Replay infrastructure/trace error: {error}");
            std::process::exit(2);
        }
    }
}

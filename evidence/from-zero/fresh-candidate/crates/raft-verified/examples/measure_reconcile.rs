//! DRAFT core microbenchmark, not Raft throughput or a regression verdict.
use raft_verified::{reconcile_log, Command, Entry, RequestId};
use std::{env, fs, hint::black_box, time::Instant};

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('\"');
    out
}

fn entry(term: u64) -> Entry {
    Entry {
        term,
        command: Command::Client {
            request: RequestId {
                client: 1,
                sequence: 1,
            },
            bytes: vec![42; 32],
        },
    }
}

fn main() {
    assert!(
        !cfg!(debug_assertions),
        "Run this measurement with --release."
    );
    let version = env::var("RAFT_CODE_VERSION").unwrap_or_else(|_| "DRAFT working tree".to_owned());
    let machine = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let cpu = machine
        .lines()
        .find_map(|line| line.strip_prefix("model name\t: "))
        .unwrap_or("unknown");
    let kernel =
        fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_else(|_| "unknown".to_owned());
    let started = Instant::now();
    for log_size in [0, 16, 256, 4096] {
        for workload in ["matching_tail", "first_conflict", "missing_suffix"] {
            let mut samples = Vec::new();
            for _ in 0..5 {
                let mut total_ns = 0;
                for _ in 0..32 {
                    // Workload allocation is outside the measurement. The call owns
                    // its incoming vector, so consuming/dropping it is inside.
                    let mut log = vec![entry(1); log_size];
                    let incoming = match workload {
                        "matching_tail" => vec![entry(1); log_size / 2],
                        "first_conflict" => vec![entry(2); log_size.max(1) / 2 + 1],
                        _ => vec![entry(1); log_size + 1],
                    };
                    let begin = Instant::now();
                    let result = reconcile_log(black_box(&mut log), 0, black_box(incoming));
                    total_ns += begin.elapsed().as_nanos();
                    assert_eq!(result, Ok(()));
                    black_box(&log);
                }
                samples.push(total_ns as f64 / 32.0);
            }
            let mut ordered = samples.clone();
            ordered.sort_by(|a, b| a.total_cmp(b));
            println!(
                "{{\"kind\":\"CORE_MICROBENCHMARK\",\"result\":\"MEASURED\",\"evidence_kind\":\"REAL_LOCAL\",\"code_version\":{},\"source\":\"crates/raft-verified/src/reconcile.rs\",\"build_mode\":\"release\",\"workload\":\"{workload}\",\"repetitions\":5,\"operations_per_sample\":32,\"log_length\":{log_size},\"command_bytes\":32,\"batch_size\":1,\"concurrency\":1,\"machine\":{},\"kernel\":{},\"architecture\":{},\"boundary\":\"reconcile_log plus timer overhead; incoming consumption included; input construction and final log destruction excluded\",\"samples_ns_per_operation\":{samples:?},\"median_ns_per_operation\":{},\"threshold\":null,\"reference_version\":null,\"comparison\":\"INCONCLUSIVE_NO_REFERENCE\"}}",
                json_string(&version), json_string(cpu), json_string(kernel.trim()),
                json_string(env::consts::ARCH), ordered[2]
            );
        }
    }
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let rss = status
        .lines()
        .find(|line| line.starts_with("VmHWM:"))
        .unwrap_or("VmHWM unavailable");
    println!("{{\"kind\":\"MEASUREMENT_RESOURCES\",\"elapsed_seconds\":{},\"peak_rss\":{},\"scope\":\"measurement process including workload setup\"}}", started.elapsed().as_secs_f64(), json_string(rss));
}

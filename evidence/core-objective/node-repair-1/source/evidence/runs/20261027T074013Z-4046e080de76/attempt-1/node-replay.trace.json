//! DRAFT initialization replay and measurement, not cluster faults or throughput.
use raft_verified::{InitError, Node, Role};
use std::{hint::black_box, time::Instant};

fn main() {
    if std::env::args().any(|a| a == "--measure") {
        assert!(!cfg!(debug_assertions), "Use a release build for measurement");
        for size in [1, 3, 5, 31] {
            let mut samples = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                for _ in 0..1000 {
                    let n = Node::new(0, black_box((0..size).collect())).unwrap();
                    black_box(n);
                }
                samples.push(start.elapsed().as_nanos() as f64 / 1000.0);
            }
            println!("{{\"kind\":\"CORE_MICROBENCHMARK\",\"result\":\"MEASURED\",\"evidence_kind\":\"REAL_LOCAL\",\"code_version\":\"DRAFT candidate bound by checker\",\"source\":\"crates/raft-verified/src/node.rs\",\"build_mode\":\"release\",\"workload\":\"initialize_{size}_members\",\"repetitions\":5,\"operations_per_sample\":1000,\"boundary\":\"membership allocation, validation, construction and destruction; includes loop/timer overhead\",\"samples_ns_per_operation\":{samples:?},\"threshold\":null,\"reference_version\":null,\"comparison\":\"INCONCLUSIVE_NO_REFERENCE\"}}");
        }
    } else {
        // Reproducible independent initial states; no protocol event/fault claim.
        for id in [1, 2, 3] {
            let n = Node::new(id, vec![1, 2, 3]).unwrap();
            assert_eq!(n.role, Role::Follower);
            assert_eq!(n.hard.term, 0);
            println!("node={id} role=Follower term=0 log_length={}", n.hard.log.len());
        }
        assert_eq!(Node::new(4, vec![1, 2, 3]), Err(InitError::InvalidMembership));
        println!("NODE_REPLAY_PASS steps=4");
    }
}

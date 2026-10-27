//! DRAFT deterministic host driver for the actual partial dispatcher; not cluster recovery.
use raft_verified::*;
use std::{hint::black_box, time::Instant};

fn image(term: u64) -> DurableState { DurableState { term, voted_for: Some(1), log: vec![] } }
fn reply(term: u64) -> Message {
    Message { from: 1, to: 2, term, rpc: OperationId { incarnation: 0, sequence: term },
              body: MessageBody::VoteResponse { granted: true } }
}
fn main() {
    if std::env::args().any(|a| a == "--measure") {
        assert!(!cfg!(debug_assertions));
        let mut samples = Vec::new();
        let mut n = Node::new(1, vec![1, 2, 3]).unwrap();
        for _ in 0..5 {
            let start = Instant::now();
            for _ in 0..1000 {
                let token = n.submit_persist(black_box(image(1)), Some(reply(1))).unwrap();
                black_box(n.step(Event::Complete { operation: token, kind: CompletionKind::Persist, outcome: CompletionOutcome::Success }).unwrap());
            }
            samples.push(start.elapsed().as_nanos() as f64 / 1000.0);
        }
        println!("{{\"kind\":\"CORE_MICROBENCHMARK\",\"workload\":\"coordinator_submit_complete\",\"build_mode\":\"release\",\"environment\":{{\"architecture\":\"{}\",\"os\":\"{}\"}},\"boundary\":\"in-memory submit plus callback; no disk, network or initialization\",\"operations_per_sample\":1000,\"samples_ns_per_operation\":{samples:?}}}", std::env::consts::ARCH, std::env::consts::OS);
        return;
    }
    let mut n = Node::new(1, vec![1, 2, 3]).unwrap();
    let mut disk = n.hard.clone();
    let mut durable = None;
    let mut steps = 0;
    let path = std::env::args().nth(1);
    let trace = path.map(std::fs::read_to_string).transpose().unwrap()
        .unwrap_or_else(|| include_str!("persistence.trace").to_owned());
    for line in trace.lines().filter(|s| !s.is_empty() && !s.starts_with('#')) {
        let words: Vec<_> = line.split_whitespace().collect();
        let output = match words[0] {
            "submit" => {
                let term = words[1].parse().unwrap();
                let token = n.submit_persist(image(term), Some(reply(term))).unwrap();
                assert!(n.pending_write.is_some());
                format!("submitted={token:?}; reply withheld")
            },
            "durable" => {
                let token = n.pending_write.as_ref().unwrap().operation;
                assert_eq!(token.sequence, words[1].parse::<u64>().unwrap());
                disk = n.hard.clone(); durable = Some(token);
                format!("disk term={}; reply still withheld", disk.term)
            },
            "complete" => {
                let operation = OperationId { incarnation: words[1].parse().unwrap(), sequence: words[2].parse().unwrap() };
                let kind = match words[3] { "Persist" => CompletionKind::Persist, "Send" => CompletionKind::Send, "Apply" => CompletionKind::Apply, _ => panic!("invalid kind") };
                let outcome = match words[4] { "Success" => CompletionOutcome::Success, "Unknown" => CompletionOutcome::Unknown, "NotPerformed" => CompletionOutcome::NotPerformed, _ => panic!("invalid outcome") };
                let result = n.step(Event::Complete { operation, kind, outcome });
                let observed = match &result {
                    Ok(Some(message)) => {
                        assert_eq!(durable, Some(operation), "host cannot acknowledge an undurable image");
                        assert_eq!(message.term, disk.term); "released".to_owned()
                    },
                    Ok(None) => "no_reply".to_owned(),
                    Err(error) => format!("{error:?}"),
                };
                assert_eq!(observed, words[5], "exact failing event: {line}");
                observed
            },
            _ => panic!("invalid event: {line}"),
        };
        steps += 1;
        println!("event={line}; output={output}; hard={}; disk={}; pending={}; recovery_required={}", n.hard.term, disk.term, n.pending_write.is_some(), n.recovery_required);
    }
    println!("PERSIST_REPLAY_PASS steps={steps}");
}

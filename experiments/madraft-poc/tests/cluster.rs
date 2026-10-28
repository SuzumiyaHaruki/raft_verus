use raft_host_madsim::cluster::{run, CrashPoint, History, Record, Scenario};
use raft_verified::*;
fn client(h: &History, id: u64, sequence: u64, bytes: &[u8]) -> bool {
    h.disks[&id].applied.iter().any(|e|matches!(&e.command,Command::Client{request,bytes:b} if request.client==7 && request.sequence==sequence && b==bytes))
}
fn show(h: &History) {
    for r in &h.records {
        println!("{r:?}");
    }
    println!(
        "TRACE_END elections={} applications={}",
        h.elections.len(),
        h.applications.len()
    );
}
#[test]
fn nominal_one_three_five() {
    for n in [1, 3, 5] {
        let h = run(Scenario::Nominal(n), 42);
        show(&h);
        assert!(!h.elections.is_empty());
        for id in 1..=n {
            for seq in 1..=3 {
                assert!(
                    client(&h, id, seq, &[seq as u8, 42]),
                    "missing real command {seq} on {id}"
                );
            }
        }
    }
}
#[test]
fn partition_replacement_and_divergent_catchup() {
    let h = run(Scenario::Partition, 43);
    show(&h);
    assert!(h.elections.iter().map(|e| e.1).max().unwrap() > h.elections[0].1);
    for id in 1..=3 {
        assert!(client(&h, id, 50, &[50]));
        assert!(!client(&h, id, 40, &[40]));
    }
    let a = &h.nodes[&1].hard.log;
    assert_eq!(a, &h.nodes[&2].hard.log);
    assert_eq!(a, &h.nodes[&3].hard.log);
}
#[test]
fn restart_at_write_and_application_boundaries() {
    for point in [
        CrashPoint::BeforeWrite,
        CrashPoint::AfterWrite,
        CrashPoint::AfterApply,
    ] {
        let h = run(Scenario::Restart(point), 44);
        show(&h);
        for id in 1..=3 {
            assert!(client(&h, id, 70, &[70]));
            for seq in 1..=3 {
                assert!(client(&h, id, seq, &[seq as u8, 42]));
            }
        }
        let restored = h
            .records
            .iter()
            .find_map(|r| {
                if let Record::Restart(id) = r {
                    Some(*id)
                } else {
                    None
                }
            })
            .unwrap();
        assert!(h.disks[&restored].incarnation > 0);
    }
}
#[test]
fn faults_and_fixed_seed_repeat() {
    let a = run(Scenario::Duplicate, 771);
    let b = run(Scenario::Duplicate, 771);
    show(&a);
    show(&b);
    assert_eq!(a, b);
    for id in 1..=3 {
        assert!(client(&a, id, 3, &[3, 42]));
    }
}
#[test]
fn competing_elections_progress() {
    let h = run(Scenario::Competing, 45);
    show(&h);
    assert!(h.elections[0].1 >= 2);
    for id in 1..=3 {
        assert!(client(&h, id, 3, &[3, 42]));
    }
}
#[test]
fn codec_preserves_commands_and_rejects_damage() {
    use raft_host_madsim::cluster::{decode, encode};
    let m = Message {
        from: 1,
        to: 2,
        term: 5,
        rpc: OperationId {
            incarnation: 2,
            sequence: 88,
        },
        body: MessageBody::AppendEntries {
            previous_index: 3,
            previous_term: 2,
            leader_commit: 4,
            entries: vec![Entry {
                term: 5,
                command: Command::Client {
                    request: RequestId {
                        client: 6,
                        sequence: 7,
                    },
                    bytes: vec![0, 255, 3],
                },
            }],
        },
    };
    let bytes = encode(&m);
    assert_eq!(decode(&bytes), Some(m));
    for len in 0..bytes.len() {
        assert!(decode(&bytes[..len]).is_none());
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(decode(&extra).is_none());
}

#[test]
fn election_one_three_five() {
    use madsim::{
        runtime::Runtime,
        time::{sleep, Duration},
    };
    use raft_host_madsim::cluster::Cluster;
    for count in [1, 3, 5] {
        Runtime::with_seed_and_config(53, Default::default()).block_on(async {
            let c = Cluster::new(count).await;
            sleep(Duration::from_millis(1500)).await;
            let h = c.history();
            show(&h);
            assert!(!h.elections.is_empty(), "no actual node became leader");
            assert!(h.nodes.values().any(|n| n.role == Role::Leader));
        });
    }
}
#[test]
fn election_competition_and_replacement() {
    use madsim::{
        runtime::Runtime,
        time::{sleep, Duration},
    };
    use raft_host_madsim::cluster::Cluster;
    Runtime::with_seed_and_config(54, Default::default()).block_on(async {
        let c = Cluster::new(3).await;
        for id in 1..=3 {
            c.event(
                id,
                Event::ElectionTimeout {
                    incarnation: 0,
                    generation: 1,
                },
            )
            .await;
        }
        sleep(Duration::from_millis(1500)).await;
        let old = c.leader(None);
        let term = c.history().nodes[&old].hard.term;
        c.partition(old, true);
        sleep(Duration::from_millis(1000)).await;
        let next = c.leader(Some(old));
        assert_ne!(old, next);
        assert!(c.history().nodes[&next].hard.term > term);
        c.partition(old, false);
        sleep(Duration::from_millis(700)).await;
        show(&c.history());
    });
}

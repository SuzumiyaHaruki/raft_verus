//! Independent STATE-INIT acceptance tests. The generation seed must fail these.
use raft_verified::{InitError, Node, Role};

fn check_fresh(id: u64, members: Vec<u64>) {
    let n = Node::new(id, members.clone()).expect("valid membership must construct a node");
    assert_eq!(n.id, id);
    assert_eq!(n.members, members);
    assert_eq!(n.role, Role::Follower);
    assert!(n.pending_write.is_none());
    assert!(!n.recovery_required);
    assert_eq!(n.hard.term, 0);
    assert_eq!(n.hard.voted_for, None);
    assert!(n.hard.log.is_empty());
    assert_eq!((n.commit, n.delivered, n.application_cursor), (0, 0, 0));
    assert_eq!((n.incarnation, n.serial, n.deadline), (0, 0, 1));
}

#[test]
fn one_member() { check_fresh(0, vec![0]); }
#[test]
fn several_members() { check_fresh(2, vec![1, 2, 3, 4, 5]); }
#[test]
fn membership_order_is_not_a_precondition() { check_fresh(3, vec![3, 1, 2]); }
#[test]
fn maximum_identity_is_not_a_sentinel() { check_fresh(u64::MAX, vec![0, u64::MAX]); }
#[test]
fn empty_membership() { assert_eq!(Node::new(0, vec![]), Err(InitError::InvalidMembership)); }
#[test]
fn unknown_identity() { assert_eq!(Node::new(9, vec![1, 2, 3]), Err(InitError::InvalidMembership)); }
#[test]
fn duplicate_identity() { assert_eq!(Node::new(1, vec![1, 1]), Err(InitError::InvalidMembership)); }
#[test]
fn nonadjacent_duplicate_peer() { assert_eq!(Node::new(1, vec![1, 2, 3, 2]), Err(InitError::InvalidMembership)); }

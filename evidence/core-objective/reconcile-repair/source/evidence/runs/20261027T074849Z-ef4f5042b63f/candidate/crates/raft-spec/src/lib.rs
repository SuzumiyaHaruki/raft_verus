//! DRAFT, PARTIAL: semantic candidate, not a proved Raft implementation.
//! Direct Verus entrypoint; see input/model.md for semantics and open proof obligations.
use vstd::prelude::*;

verus! {
pub type NodeId = nat;
pub type RequestId = (nat, nat);

pub enum Role { Follower, Candidate, Leader }
pub enum Command { Client { request: RequestId, value: Seq<u8> }, Noop }
pub struct Entry { pub term: nat, pub command: Command }
pub struct Token { pub incarnation: nat, pub serial: nat }
pub struct HardState { pub term: nat, pub voted_for: Option<NodeId>, pub log: Seq<Entry> }
pub enum Body {
    RequestVote { last_index: nat, last_term: nat },
    VoteResponse { granted: bool },
    AppendEntries { previous: nat, previous_term: nat, entries: Seq<Entry>, commit: nat },
    AppendResponse { accepted: bool, matched: nat },
}
pub struct Message { pub from: NodeId, pub to: NodeId, pub term: nat, pub rpc: Token, pub body: Body }
pub struct Write { pub token: Token, pub image: HardState, pub reply: Option<Message> }
pub struct Flight { pub token: Token, pub term: nat, pub previous: nat, pub last: nat }
pub struct Delivery { pub node: NodeId, pub incarnation: nat, pub index: nat, pub entry: Entry, pub token: Token }
pub struct Election { pub node: NodeId, pub term: nat, pub log: Seq<Entry> }
pub struct Commit { pub node: NodeId, pub term: nat, pub prefix: Seq<Entry> }
pub struct Acceptance { pub node: NodeId, pub term: nat, pub index: nat, pub request: RequestId }

pub struct Node {
    pub alive: bool,
    pub hard: HardState,
    pub disk: HardState,
    pub role: Role,
    pub votes: Set<NodeId>,
    pub next_index: Map<NodeId, nat>,
    pub matched: Map<NodeId, nat>,
    pub flight: Map<NodeId, Flight>,
    pub commit: nat,
    pub delivered: nat,
    pub application_cursor: nat,
    pub incarnation: nat,
    pub serial: nat,
    pub deadline: nat,
    pub pending_write: Option<Write>,
    pub pending_apply: Option<Delivery>,
    pub outbox: Seq<Message>,
}

pub struct Cluster {
    pub members: Set<NodeId>,
    pub nodes: Map<NodeId, Node>,
    pub now: nat,
    pub network: Seq<Message>,
    pub sent: Seq<Message>,
    pub received: Seq<Message>,
    pub requests: Map<RequestId, Seq<u8>>,
    pub accepted: Seq<Acceptance>,
    pub elections: Seq<Election>,
    pub commits: Seq<Commit>,
    pub deliveries: Seq<Delivery>,
    pub applications: Seq<Delivery>,
}

pub open spec fn empty_hard() -> HardState {
    HardState { term: 0, voted_for: None, log: Seq::empty() }
}

pub open spec fn fresh_node() -> Node {
    Node {
        alive: true, hard: empty_hard(), disk: empty_hard(), role: Role::Follower,
        votes: Set::empty(), next_index: Map::empty(), matched: Map::empty(), flight: Map::empty(),
        commit: 0, delivered: 0, application_cursor: 0, incarnation: 0, serial: 0, deadline: 1,
        pending_write: None, pending_apply: None, outbox: Seq::empty(),
    }
}

pub open spec fn initial(members: Set<NodeId>) -> Cluster {
    Cluster {
        members, nodes: Map::new(|id: NodeId| members.contains(id), |id: NodeId| fresh_node()),
        now: 0, network: Seq::empty(), sent: Seq::empty(), received: Seq::empty(),
        requests: Map::empty(), accepted: Seq::empty(), elections: Seq::empty(),
        commits: Seq::empty(), deliveries: Seq::empty(), applications: Seq::empty(),
    }
}

pub open spec fn init(s: Cluster) -> bool {
    s.members.finite() && s.members.len() > 0 && s == initial(s.members)
}

// Concrete nonempty witness candidate; proving init(three_node_initial()) is PO-INIT.
pub open spec fn three_node_initial() -> Cluster {
    initial(Set::empty().insert(0).insert(1).insert(2))
}

pub open spec fn majority(members: Set<NodeId>, voters: Set<NodeId>) -> bool {
    members.finite() && voters.finite() && voters.subset_of(members) && 2 * voters.len() > members.len()
}
pub open spec fn term_at(log: Seq<Entry>, index: nat) -> nat {
    if index == 0 { 0 } else { log[(index - 1) as int].term }
}
pub open spec fn prefix(a: Seq<Entry>, b: Seq<Entry>) -> bool {
    a.len() <= b.len() && a == b.take(a.len() as int)
}
pub open spec fn clean(n: Node) -> bool { n.alive && n.pending_write.is_none() && n.hard == n.disk }
pub open spec fn token(n: Node) -> Token { Token { incarnation: n.incarnation, serial: n.serial } }
pub open spec fn set_node(s: Cluster, id: NodeId, n: Node) -> Cluster {
    Cluster { nodes: s.nodes.insert(id, n), ..s }
}
pub open spec fn consume(s: Cluster, p: int) -> Cluster {
    Cluster { network: s.network.remove(p), received: s.received.push(s.network[p]), ..s }
}
pub open spec fn emit(s: Cluster, message: Message) -> Cluster {
    Cluster { network: s.network.push(message), sent: s.sent.push(message), ..s }
}
pub open spec fn write(n: Node, hard: HardState, reply: Option<Message>) -> Node {
    Node { hard, pending_write: Some(Write { token: token(n), image: hard, reply }), serial: n.serial + 1, ..n }
}
pub open spec fn available(s: Cluster, id: NodeId) -> bool {
    s.members.contains(id) && clean(s.nodes[id])
}
pub open spec fn packet(s: Cluster, p: int) -> bool {
    0 <= p < s.network.len() && s.members.contains(s.network[p].from)
    && s.members.contains(s.network[p].to) && available(s, s.network[p].to)
}
pub open spec fn reply(request: Message, term: nat, body: Body) -> Message {
    Message { from: request.to, to: request.from, term, rpc: request.rpc, body }
}

pub open spec fn election_timeout(s: Cluster, t: Cluster, id: NodeId, deadline: nat) -> bool {
    let n = s.nodes[id];
    let hard = HardState { term: n.hard.term + 1, voted_for: Some(id), ..n.hard };
    available(s, id) && n.role != Role::Leader && s.now >= n.deadline && deadline > s.now
    && t == set_node(s, id, write(Node {
        role: Role::Candidate, votes: Set::empty().insert(id), deadline,
        flight: Map::empty(), outbox: Seq::empty(), ..n
    }, hard, None))
}

pub open spec fn send_vote(s: Cluster, t: Cluster, id: NodeId, peer: NodeId) -> bool {
    let n = s.nodes[id];
    let m = Message { from: id, to: peer, term: n.hard.term, rpc: token(n), body: Body::RequestVote {
        last_index: n.hard.log.len(), last_term: term_at(n.hard.log, n.hard.log.len()),
    }};
    available(s, id) && n.role == Role::Candidate && s.members.contains(peer) && peer != id
    && t == emit(set_node(s, id, Node { serial: n.serial + 1, ..n }), m)
}

// Observe a higher term before consuming the message; storage must finish first.
pub open spec fn higher_term(s: Cluster, t: Cluster, p: int) -> bool {
    let m = s.network[p]; let n = s.nodes[m.to];
    packet(s, p) && m.term > n.hard.term
    && t == set_node(s, m.to, write(Node {
        role: Role::Follower, votes: Set::empty(), flight: Map::empty(), outbox: Seq::empty(), ..n
    }, HardState { term: m.term, voted_for: None, ..n.hard }, None))
}

pub open spec fn receive_vote_request(s: Cluster, t: Cluster, p: int, deadline: nat) -> bool {
    let m = s.network[p]; let n = s.nodes[m.to];
    packet(s, p) && m.term <= n.hard.term && deadline > s.now
    && match m.body {
        Body::RequestVote { last_index, last_term } => {
            let fresh = last_term > term_at(n.hard.log, n.hard.log.len())
                || (last_term == term_at(n.hard.log, n.hard.log.len()) && last_index >= n.hard.log.len());
            let grant = m.term == n.hard.term && fresh
                && (n.hard.voted_for.is_none() || n.hard.voted_for == Some(m.from));
            let response = reply(m, n.hard.term, Body::VoteResponse { granted: grant });
            if grant {
                t == set_node(consume(s, p), m.to, write(Node { deadline, ..n },
                    HardState { voted_for: Some(m.from), ..n.hard }, Some(response)))
            } else {
                t == set_node(consume(s, p), m.to, Node { outbox: n.outbox.push(response), ..n })
            }
        },
        _ => false,
    }
}

pub open spec fn receive_vote_response(s: Cluster, t: Cluster, p: int) -> bool {
    let m = s.network[p]; let n = s.nodes[m.to];
    packet(s, p) && m.term <= n.hard.term
    && match m.body {
        Body::VoteResponse { granted } => {
            let votes = if m.term == n.hard.term && n.role == Role::Candidate && granted {
                n.votes.insert(m.from)
            } else { n.votes };
            t == set_node(consume(s, p), m.to, Node { votes, ..n })
        },
        _ => false,
    }
}

pub open spec fn become_leader(s: Cluster, t: Cluster, id: NodeId) -> bool {
    let n = s.nodes[id];
    let n2 = Node {
        role: Role::Leader,
        next_index: Map::new(|p: NodeId| s.members.contains(p), |p: NodeId| n.hard.log.len() + 1),
        matched: Map::new(|p: NodeId| s.members.contains(p), |p: NodeId| 0),
        flight: Map::empty(), ..n
    };
    available(s, id) && n.role == Role::Candidate && majority(s.members, n.votes)
    && t == Cluster { elections: s.elections.push(Election { node: id, term: n.hard.term, log: n.hard.log }),
        ..set_node(s, id, write(n2, HardState { log: n.hard.log.push(Entry { term: n.hard.term, command: Command::Noop }), ..n.hard }, None)) }
}

pub open spec fn introduce_request(s: Cluster, t: Cluster, request: RequestId, value: Seq<u8>) -> bool {
    (!s.requests.contains_key(request) || s.requests[request] == value)
    && t == Cluster { requests: s.requests.insert(request, value), ..s }
}
pub open spec fn propose(s: Cluster, t: Cluster, id: NodeId, request: RequestId) -> bool {
    let n = s.nodes[id];
    available(s, id) && n.role == Role::Leader && s.requests.contains_key(request)
    && t == Cluster { accepted: s.accepted.push(Acceptance { node: id, term: n.hard.term, index: n.hard.log.len() + 1, request }),
        ..set_node(s, id, write(n, HardState {
            log: n.hard.log.push(Entry { term: n.hard.term, command: Command::Client { request, value: s.requests[request] }}),
            ..n.hard
        }, None)) }
}

// entries may be empty (heartbeat) or contain the whole remaining suffix.
pub open spec fn send_append(s: Cluster, t: Cluster, id: NodeId, peer: NodeId) -> bool {
    let n = s.nodes[id]; let start = n.next_index[peer];
    let m = Message { from: id, to: peer, term: n.hard.term, rpc: token(n), body: Body::AppendEntries {
        previous: (start - 1) as nat, previous_term: term_at(n.hard.log, (start - 1) as nat),
        entries: n.hard.log.subrange((start - 1) as int, n.hard.log.len() as int), commit: n.commit,
    }};
    available(s, id) && n.role == Role::Leader && s.members.contains(peer) && peer != id
    && n.next_index.contains_key(peer) && 1 <= start <= n.hard.log.len() + 1
    && t == emit(set_node(s, id, Node { serial: n.serial + 1,
        flight: n.flight.insert(peer, Flight { token: token(n), term: n.hard.term, previous: (start - 1) as nat, last: n.hard.log.len() }), ..n }), m)
}

// k is the first incoming conflict/missing position; a shorter matching heartbeat never truncates.
pub open spec fn merged(old: Seq<Entry>, previous: nat, incoming: Seq<Entry>, result: Seq<Entry>) -> bool {
    previous <= old.len() && (
        (previous + incoming.len() <= old.len()
            && (forall |j: int| 0 <= j < incoming.len() ==> old[previous as int + j].term == incoming[j].term)
            && result == old)
        || exists |k: nat| k < incoming.len()
            && (forall |j: int| 0 <= j < k ==> previous + j < old.len() && old[previous as int + j].term == incoming[j].term)
            && (previous + k >= old.len() || old[(previous + k) as int].term != incoming[k as int].term)
            && result == old.take((previous + k) as int).add(incoming.subrange(k as int, incoming.len() as int))
    )
}

pub open spec fn receive_append(s: Cluster, t: Cluster, p: int, log: Seq<Entry>, deadline: nat) -> bool {
    let m = s.network[p]; let n = s.nodes[m.to];
    packet(s, p) && m.term <= n.hard.term && deadline > s.now
    && match m.body {
        Body::AppendEntries { previous, previous_term, entries, commit } => {
            let ok = m.term == n.hard.term && previous <= n.hard.log.len() && term_at(n.hard.log, previous) == previous_term;
            let base = if m.term == n.hard.term { Node { role: Role::Follower, deadline, ..n } } else { n };
            if ok {
                let end = previous + entries.len();
                let new_commit = if commit < end { commit } else { end };
                let c = if n.commit > new_commit { n.commit } else { new_commit };
                merged(n.hard.log, previous, entries, log)
                && t == Cluster { commits: if c > n.commit { s.commits.push(Commit { node: m.to, term: n.hard.term, prefix: log.take(c as int) }) } else { s.commits },
                    ..set_node(consume(s, p), m.to, write(Node { commit: c, ..base }, HardState { log, ..n.hard },
                    Some(reply(m, n.hard.term, Body::AppendResponse { accepted: true, matched: end })))) }
            } else {
                t == set_node(consume(s, p), m.to, Node { outbox: n.outbox.push(reply(m, n.hard.term,
                    Body::AppendResponse { accepted: false, matched: 0 })), ..base })
            }
        },
        _ => false,
    }
}

pub open spec fn receive_append_response(s: Cluster, t: Cluster, p: int) -> bool {
    let m = s.network[p]; let n = s.nodes[m.to];
    packet(s, p) && m.term <= n.hard.term
    && match m.body {
        Body::AppendResponse { accepted, matched } => {
            let relevant = n.role == Role::Leader && m.term == n.hard.term
                && n.flight.contains_key(m.from) && n.flight[m.from].token == m.rpc && n.flight[m.from].term == m.term;
            if relevant && accepted && matched == n.flight[m.from].last {
                let old_match = n.matched[m.from];
                let high = if old_match > matched { old_match } else { matched };
                t == set_node(consume(s, p), m.to, Node { matched: n.matched.insert(m.from, high),
                    next_index: n.next_index.insert(m.from, high + 1), flight: n.flight.remove(m.from), ..n })
            } else if relevant && !accepted {
                let back = if n.next_index[m.from] > 1 { (n.next_index[m.from] - 1) as nat } else { 1 };
                t == set_node(consume(s, p), m.to, Node { next_index: n.next_index.insert(m.from, back),
                    flight: n.flight.remove(m.from), ..n })
            } else { t == consume(s, p) }
        },
        _ => false,
    }
}

pub open spec fn advance_commit(s: Cluster, t: Cluster, id: NodeId, index: nat) -> bool {
    let n = s.nodes[id];
    let replicated = s.members.filter(|peer: NodeId|
        peer == id || (n.matched.contains_key(peer) && n.matched[peer] >= index));
    available(s, id) && n.role == Role::Leader && n.commit < index <= n.disk.log.len()
    && term_at(n.disk.log, index) == n.hard.term && majority(s.members, replicated)
    && t == Cluster { commits: s.commits.push(Commit { node: id, term: n.hard.term, prefix: n.hard.log.take(index as int) }),
        ..set_node(s, id, Node { commit: index, ..n }) }
}

pub open spec fn storage_durable(s: Cluster, t: Cluster, id: NodeId, operation: Token) -> bool {
    let n = s.nodes[id];
    s.members.contains(id) && n.alive && n.pending_write.is_some()
    && n.pending_write.unwrap().token == operation && operation.incarnation == n.incarnation
    && t == set_node(s, id, Node { disk: n.pending_write.unwrap().image, ..n })
}
pub open spec fn storage_ack(s: Cluster, t: Cluster, id: NodeId, operation: Token) -> bool {
    let n = s.nodes[id]; let w = n.pending_write.unwrap();
    s.members.contains(id) && n.alive && n.pending_write.is_some()
    && w.token == operation && operation.incarnation == n.incarnation && n.disk == w.image
    && t == set_node(s, id, Node { pending_write: None,
        outbox: match w.reply { Some(m) => n.outbox.push(m), None => n.outbox }, ..n })
}
pub open spec fn flush_message(s: Cluster, t: Cluster, id: NodeId) -> bool {
    let n = s.nodes[id];
    available(s, id) && n.outbox.len() > 0
    && t == emit(set_node(s, id, Node { outbox: n.outbox.remove(0), ..n }), n.outbox[0])
}
pub open spec fn deliver(s: Cluster, t: Cluster, id: NodeId) -> bool {
    let n = s.nodes[id]; let index = n.delivered + 1;
    let d = Delivery { node: id, incarnation: n.incarnation, index, entry: n.hard.log[(index - 1) as int], token: token(n) };
    available(s, id) && n.pending_apply.is_none() && index <= n.commit && index <= n.disk.log.len()
    && t == Cluster { deliveries: s.deliveries.push(d), ..set_node(s, id, Node {
        delivered: index, pending_apply: Some(d), serial: n.serial + 1, ..n }) }
}
// Optional application contract: apply + durable cursor is one host transaction.
pub open spec fn application_performed(s: Cluster, t: Cluster, id: NodeId, operation: Token) -> bool {
    let n = s.nodes[id]; let d = n.pending_apply.unwrap();
    s.members.contains(id) && n.alive && n.pending_apply.is_some() && d.token == operation
    && d.index == n.application_cursor + 1
    && t == Cluster { applications: s.applications.push(d),
        ..set_node(s, id, Node { application_cursor: d.index, ..n }) }
}
pub open spec fn application_ack(s: Cluster, t: Cluster, id: NodeId, operation: Token) -> bool {
    let n = s.nodes[id]; let d = n.pending_apply.unwrap();
    s.members.contains(id) && n.alive && n.pending_apply.is_some() && d.token == operation
    && operation.incarnation == n.incarnation && n.application_cursor == d.index
    && t == set_node(s, id, Node { pending_apply: None, ..n })
}
pub open spec fn crash(s: Cluster, t: Cluster, id: NodeId) -> bool {
    let n = s.nodes[id];
    s.members.contains(id) && n.alive
    && t == set_node(s, id, Node { alive: false, hard: n.disk, role: Role::Follower,
        commit: 0, delivered: n.application_cursor, pending_write: None, pending_apply: None,
        outbox: Seq::empty(), votes: Set::empty(), matched: Map::empty(),
        next_index: Map::empty(), flight: Map::empty(), ..n })
}
pub open spec fn recover(s: Cluster, t: Cluster, id: NodeId, deadline: nat) -> bool {
    let n = s.nodes[id];
    s.members.contains(id) && !n.alive && deadline > s.now
    && t == set_node(s, id, Node { alive: true, hard: n.disk, incarnation: n.incarnation + 1,
        serial: 0, deadline, ..n })
}
// Fail-stop storage error retains the last durable image; no fictitious success.
pub open spec fn storage_error(s: Cluster, t: Cluster, id: NodeId) -> bool {
    s.nodes[id].pending_write.is_some() && crash(s, t, id)
}
pub open spec fn stale_completion(s: Cluster, t: Cluster, id: NodeId, operation: Token) -> bool {
    s.members.contains(id) && operation.incarnation != s.nodes[id].incarnation && t == s
}

pub enum Action {
    Tick, Stutter,
    ElectionTimeout { node: NodeId, deadline: nat }, SendVote { node: NodeId, peer: NodeId },
    HigherTerm { packet: int }, VoteRequest { packet: int, deadline: nat }, VoteResponse { packet: int },
    BecomeLeader { node: NodeId }, Introduce { request: RequestId, value: Seq<u8> }, Propose { node: NodeId, request: RequestId },
    SendAppend { node: NodeId, peer: NodeId }, Append { packet: int, log: Seq<Entry>, deadline: nat }, AppendResponse { packet: int },
    Commit { node: NodeId, index: nat }, StorageDurable { node: NodeId, token: Token }, StorageAck { node: NodeId, token: Token },
    Flush { node: NodeId }, Deliver { node: NodeId }, ApplicationPerformed { node: NodeId, token: Token }, ApplicationAck { node: NodeId, token: Token },
    Crash { node: NodeId }, Recover { node: NodeId, deadline: nat }, StorageError { node: NodeId }, StaleCompletion { node: NodeId, token: Token },
    Drop { packet: int }, Duplicate { packet: int },
}

pub open spec fn step(s: Cluster, a: Action, t: Cluster) -> bool {
    match a {
        Action::Tick => t == Cluster { now: s.now + 1, ..s },
        Action::Stutter => t == s,
        Action::ElectionTimeout { node, deadline } => election_timeout(s, t, node, deadline),
        Action::SendVote { node, peer } => send_vote(s, t, node, peer),
        Action::HigherTerm { packet } => higher_term(s, t, packet),
        Action::VoteRequest { packet, deadline } => receive_vote_request(s, t, packet, deadline),
        Action::VoteResponse { packet } => receive_vote_response(s, t, packet),
        Action::BecomeLeader { node } => become_leader(s, t, node),
        Action::Introduce { request, value } => introduce_request(s, t, request, value),
        Action::Propose { node, request } => propose(s, t, node, request),
        Action::SendAppend { node, peer } => send_append(s, t, node, peer),
        Action::Append { packet, log, deadline } => receive_append(s, t, packet, log, deadline),
        Action::AppendResponse { packet } => receive_append_response(s, t, packet),
        Action::Commit { node, index } => advance_commit(s, t, node, index),
        Action::StorageDurable { node, token } => storage_durable(s, t, node, token),
        Action::StorageAck { node, token } => storage_ack(s, t, node, token),
        Action::Flush { node } => flush_message(s, t, node),
        Action::Deliver { node } => deliver(s, t, node),
        Action::ApplicationPerformed { node, token } => application_performed(s, t, node, token),
        Action::ApplicationAck { node, token } => application_ack(s, t, node, token),
        Action::Crash { node } => crash(s, t, node),
        Action::Recover { node, deadline } => recover(s, t, node, deadline),
        Action::StorageError { node } => storage_error(s, t, node),
        Action::StaleCompletion { node, token } => stale_completion(s, t, node, token),
        Action::Drop { packet } => 0 <= packet < s.network.len() && t == Cluster { network: s.network.remove(packet), ..s },
        Action::Duplicate { packet } => 0 <= packet < s.network.len() && t == Cluster { network: s.network.push(s.network[packet]), ..s },
    }
}
pub open spec fn next(s: Cluster, t: Cluster) -> bool { exists |a: Action| step(s, a, t) }

pub open spec fn well_formed(s: Cluster) -> bool {
    s.members.finite() && s.members.len() > 0 && s.nodes.dom() == s.members
    && forall |id: NodeId| s.members.contains(id) ==> {
        let n = s.nodes[id];
        n.commit <= n.hard.log.len() && n.delivered <= n.hard.log.len()
        && n.application_cursor <= n.delivered && n.disk.term <= n.hard.term
        && n.votes.subset_of(s.members)
        && (n.hard.voted_for.is_some() ==> s.members.contains(n.hard.voted_for.unwrap()))
        && (forall |i: int| 0 <= i < n.hard.log.len() ==> n.hard.log[i].term <= n.hard.term)
    }
}
pub open spec fn election_safety(s: Cluster) -> bool {
    forall |i: int, j: int| 0 <= i < s.elections.len() && 0 <= j < s.elections.len()
        && s.elections[i].term == s.elections[j].term ==> s.elections[i].node == s.elections[j].node
}
pub open spec fn leader_append_only(s: Cluster, t: Cluster) -> bool {
    forall |id: NodeId| s.members.contains(id) && t.members.contains(id)
        && s.nodes[id].alive && s.nodes[id].role == Role::Leader && t.nodes[id].alive
        ==> prefix(s.nodes[id].hard.log, t.nodes[id].hard.log)
}
pub open spec fn matching(a: Seq<Entry>, b: Seq<Entry>) -> bool {
    forall |i: int| 0 <= i < a.len() && i < b.len() && a[i].term == b[i].term
        ==> a.take(i + 1) == b.take(i + 1)
}
pub open spec fn log_matching(s: Cluster) -> bool {
    forall |a: NodeId, b: NodeId| s.members.contains(a) && s.members.contains(b)
        ==> matching(s.nodes[a].hard.log, s.nodes[b].hard.log)
}
pub open spec fn leader_completeness(s: Cluster) -> bool {
    forall |c: int, e: int| 0 <= c < s.commits.len() && 0 <= e < s.elections.len()
        && s.commits[c].term < s.elections[e].term
        ==> prefix(s.commits[c].prefix, s.elections[e].log)
}
pub open spec fn delivery_consistency(s: Cluster) -> bool {
    forall |i: int, j: int| 0 <= i < s.deliveries.len() && 0 <= j < s.deliveries.len()
        && s.deliveries[i].index == s.deliveries[j].index ==> s.deliveries[i].entry == s.deliveries[j].entry
}
pub open spec fn application_safety(s: Cluster) -> bool {
    forall |i: int, j: int| 0 <= i < s.applications.len() && 0 <= j < s.applications.len()
        && s.applications[i].index == s.applications[j].index ==> s.applications[i].entry.command == s.applications[j].entry.command
}
pub open spec fn command_origin(s: Cluster, command: Command) -> bool {
    match command {
        Command::Client { request, value } => s.requests.contains_key(request) && s.requests[request] == value,
        Command::Noop => true,
    }
}
pub open spec fn interface_safety(s: Cluster) -> bool {
    delivery_consistency(s)
    && (forall |i: int| #![trigger s.deliveries[i]] 0 <= i < s.deliveries.len() ==> {
        let d = s.deliveries[i];
        d.index > 0 && command_origin(s, d.entry.command)
        && exists |c: int| 0 <= c < s.commits.len() && d.index <= s.commits[c].prefix.len()
            && s.commits[c].prefix[(d.index - 1) as int] == d.entry
    })
    && (forall |i: int, j: int| 0 <= i < j < s.deliveries.len()
        && s.deliveries[i].node == s.deliveries[j].node
        && s.deliveries[i].incarnation == s.deliveries[j].incarnation
        ==> s.deliveries[i].index < s.deliveries[j].index)
}

// Plain, fully defined trajectory predicates; no temporal axioms or library claims.
pub type Trace = spec_fn(nat) -> Cluster;
pub type Actions = spec_fn(nat) -> Action;
pub open spec fn valid_trace(ex: Trace, actions: Actions) -> bool {
    init(ex(0)) && forall |time: nat| #[trigger] step(ex(time), actions(time), ex(time + 1))
}
pub open spec fn always(ex: Trace, p: spec_fn(Cluster) -> bool) -> bool {
    forall |time: nat| #[trigger] p(ex(time))
}
pub open spec fn eventually_from(ex: Trace, from: nat, p: spec_fn(Cluster) -> bool) -> bool {
    exists |time: nat| time >= from && #[trigger] p(ex(time))
}
pub open spec fn enabled_from(ex: Trace, action: Action, from: nat) -> bool {
    forall |time: nat| #![trigger ex(time)] time >= from
        ==> exists |after: Cluster| #[trigger] step(ex(time), action, after)
}
pub open spec fn action_occurs_from(actions: Actions, action: Action, from: nat) -> bool {
    exists |time: nat| time >= from && #[trigger] actions(time) == action
}
pub open spec fn weak_fair(ex: Trace, actions: Actions, action: Action) -> bool {
    forall |from: nat| #[trigger] enabled_from(ex, action, from)
        ==> action_occurs_from(actions, action, from)
}
pub open spec fn stable_leader_from(ex: Trace, time: nat, leader: NodeId, term: nat) -> bool {
    forall |later: nat| #![trigger ex(later)] later >= time ==> ex(later).nodes[leader].alive
        && ex(later).nodes[leader].role == Role::Leader && ex(later).nodes[leader].hard.term == term
}

// Targets only. Precise environment-to-target theorem remains OPEN (ADR-04).
pub open spec fn l_election(ex: Trace, from: nat, quorum: Set<NodeId>) -> bool {
    exists |time: nat, leader: NodeId, term: nat| time >= from && quorum.contains(leader)
        && #[trigger] stable_leader_from(ex, time, leader, term)
}
pub open spec fn request_committed(s: Cluster, request: RequestId) -> bool {
    exists |c: int, i: int| #![trigger s.commits[c].prefix[i]]
        0 <= c < s.commits.len() && 0 <= i < s.commits[c].prefix.len()
        && match s.commits[c].prefix[i].command {
            Command::Client { request: found, value } => found == request && s.requests[request] == value,
            Command::Noop => false,
        }
}
pub open spec fn l_commit(ex: Trace, from: nat, request: RequestId) -> bool {
    eventually_from(ex, from, |s: Cluster| request_committed(s, request))
}
pub open spec fn l_deliver(ex: Trace, from: nat, node: NodeId, index: nat, entry: Entry) -> bool {
    eventually_from(ex, from, |s: Cluster| exists |i: int| 0 <= i < s.deliveries.len()
        && s.deliveries[i].node == node && s.deliveries[i].index == index && s.deliveries[i].entry == entry)
}
// Observable representation relation; coordinator/internal-state refinement is PO-REPRESENTATION/PO-IO.
pub open spec fn observation_refines(concrete_log: Seq<Entry>, concrete_term: nat,
    concrete_commit: nat, abstract_node: Node) -> bool {
    concrete_log == abstract_node.hard.log && concrete_term == abstract_node.hard.term
        && concrete_commit == abstract_node.commit
}
}

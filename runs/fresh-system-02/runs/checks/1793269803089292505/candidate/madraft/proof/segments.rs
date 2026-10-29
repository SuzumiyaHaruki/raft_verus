//! Partial histories of election decision segments, not complete host executions.
use super::election::*;
use crate::support::*;
use vstd::prelude::*;
verus! {
pub enum Message { Request(Vote), Response(Reply), Heartbeat(u64), HeartbeatReply(u64) }
pub enum Input {
    Timeout { node: u64 },
    SendHeartbeat { group: nat, node: nat, peer: nat },
    HeartbeatDelivery { delivery: Delivery<Message> },
    VoteReplyDelivery { delivery: Delivery<Message>, started: u64, votes: usize, members: usize },
    SendVote { group: nat, node: u64, peer: nat, last_term: u64, last_index: u64 },
    Deliver { delivery: Delivery<Message>, last_term: u64, last_index: u64 },
}
pub type Output = Seq<Envelope<Message>>;
pub open spec fn receive(i: Input) -> Option<Delivery<Message>> {
    match i {
        Input::Deliver { delivery, .. } | Input::HeartbeatDelivery { delivery }
        | Input::VoteReplyDelivery { delivery, .. } => Some(delivery), _ => None }
}
pub open spec fn emit(o: Output) -> Seq<Envelope<Message>> { o }
pub open spec fn initial(s: Election) -> bool { s == fresh() }
pub open spec fn step(s: Election, i: Input, t: Election, o: Output) -> bool {
    match i {
        Input::SendHeartbeat { group,node,peer } => t == s && o == seq![Envelope {
            group, sender: node, recipient: peer, body: Message::Heartbeat(s.term) }],
        Input::HeartbeatDelivery { delivery: d } => match d.message.body {
            Message::Heartbeat(term) => t == heartbeat_result(s,term) && o == seq![Envelope {
                group: d.message.group, sender: d.message.recipient, recipient: d.message.sender,
                body: Message::HeartbeatReply(t.term) }],
            Message::HeartbeatReply(term) => t == advanced(s,term) && o == Seq::<Envelope<Message>>::empty(),
            _ => false,
        },
        Input::VoteReplyDelivery { delivery: d, started,votes,members } => match d.message.body {
            Message::Response(r) => t == reply_result(s,r.term,started,r.granted,votes,members)
                && o == Seq::<Envelope<Message>>::empty(),
            _ => false,
        },
        Input::Timeout { node } => t == timeout_result(s,node) && o == Seq::<Envelope<Message>>::empty(),
        Input::SendVote { group, node, peer, last_term, last_index } =>
            t == s && o == seq![Envelope { group, sender: node as nat, recipient: peer,
                body: Message::Request(Vote { term: s.term, candidate: node, last_term, last_index }) }],
        Input::Deliver { delivery: d, last_term, last_index } => match d.message.body {
            Message::Request(v) => t == vote_result(s,v,last_term,last_index)
                && o == seq![Envelope { group: d.message.group, sender: d.message.recipient,
                    recipient: d.message.sender, body: Message::Response(Reply {
                        term: t.term, granted: grant(s,v,last_term,last_index) }) }],
            _ => false, // Different RPC segments use their corresponding input variant.
        },
    }
}
// A concrete send is eligible for receipt, and the real handler produces the
// response specified by the modeled step. No vote correctness is assumed by exchange.
pub fn actual_request_path(sender: Election, receiver: Election, group: Ghost<nat>,
    node: u64, peer: Ghost<nat>, last_term: u64, last_index: u64)
    -> (r: (Election, Reply))
    ensures step(receiver, Input::Deliver { delivery: Delivery { send: 0,
        message: Envelope { group: group@, sender: node as nat, recipient: peer@,
        body: Message::Request(Vote { term: sender.term, candidate: node, last_term, last_index }) } },
        last_term, last_index }, r.0,
        seq![Envelope { group: group@, sender: peer@, recipient: node as nat, body: Message::Response(r.1) }]),
{
    let vote = make_vote(sender,node,last_term,last_index);
    let r = request_vote(receiver,vote,last_term,last_index);
    proof {
        let request = Envelope { group: group@, sender: node as nat, recipient: peer@, body: Message::Request(vote) };
        let sent = seq![request];
        let reply = Envelope { group: group@, sender: peer@, recipient: node as nat, body: Message::Response(r.1) };
        assert(exchange(Seq::empty(), None, sent, sent));
        assert(exchange(sent, Some(Delivery { send: 0, message: request }), seq![reply], sent + seq![reply]));
    }
    r
}
pub proof fn segment_vote_stable(s: Election,i: Input,t: Election,o: Output)
    requires step(s,i,t,o),
    ensures t.term >= s.term,
        t.term == s.term && s.voted_for.is_some() ==> t.voted_for == s.voted_for,
{
    match i {
        Input::Timeout { .. } => {},
        Input::SendVote { .. } => {},
        Input::SendHeartbeat { .. } => {},
        Input::HeartbeatDelivery { delivery } => { match delivery.message.body { _ => {} } },
        Input::VoteReplyDelivery { delivery, .. } => { match delivery.message.body { _ => {} } },
        Input::Deliver { delivery, .. } => { match delivery.message.body { _ => {} } },
    }
}

}

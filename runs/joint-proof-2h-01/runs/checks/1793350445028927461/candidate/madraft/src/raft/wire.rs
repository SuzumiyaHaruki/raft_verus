//! Exact values submitted by the real RPC caller. No Raft-correctness filter.
use super::{election::VoteCall, state::{VoteRequest, VoteReply}};
use vstd::prelude::*;
verus! {
pub struct RequestSend { sender: usize, recipient: usize, request: VoteRequest }
pub struct ResponseSend { sender: usize, recipient: usize, request: VoteRequest, reply: VoteReply }
pub struct RequestView { pub sender: usize, pub recipient: usize, pub request: VoteRequest }
pub struct ResponseView { pub sender: usize, pub recipient: usize, pub request: VoteRequest, pub reply: VoteReply }
impl RequestSend {
    pub closed spec fn view(&self) -> RequestView {
        RequestView { sender: self.sender, recipient: self.recipient, request: self.request }
    }
    pub fn from_call(call: &VoteCall) -> (r: Self)
        ensures r.view().sender == call.request().candidate,
            r.view().recipient == call.peer(), r.view().request == call.request(),
    {
        let request = call.args();
        Self { sender: request.candidate, recipient: call.destination(), request }
    }
    pub fn args(&self) -> (r: VoteRequest) ensures r == self.view().request, { self.request }
    pub fn destination(&self) -> (r: usize) ensures r == self.view().recipient, { self.recipient }
}
impl ResponseSend {
    pub closed spec fn view(&self) -> ResponseView {
        ResponseView { sender: self.sender, recipient: self.recipient, request: self.request, reply: self.reply }
    }
    pub fn new(sender: usize, recipient: usize, request: VoteRequest, reply: VoteReply) -> (r: Self)
        ensures r.view().sender == sender, r.view().recipient == recipient,
            r.view().request == request, r.view().reply == reply,
    { Self { sender, recipient, request, reply } }
    pub fn body(&self) -> (r: VoteReply) ensures r == self.view().reply, { self.reply }
}
#[derive(Clone, Copy)]
pub struct InboundVote { sender: usize, recipient: usize, members: usize, request: VoteRequest }
pub struct InboundView { pub sender: usize, pub recipient: usize, pub members: usize, pub request: VoteRequest }
pub open spec fn inbound_valid(v: InboundView) -> bool {
    v.sender < v.members && v.recipient < v.members && v.request.candidate == v.sender
}
impl InboundVote {
    pub closed spec fn view(&self) -> InboundView {
        InboundView { sender: self.sender, recipient: self.recipient, members: self.members, request: self.request }
    }
    pub fn new(sender: usize, recipient: usize, members: usize, request: VoteRequest) -> (r: Option<Self>)
        ensures r.is_some() == (sender < members && recipient < members && request.candidate == sender),
            r.is_some() ==> r.unwrap().valid() && inbound_valid(r.unwrap().view())
                && r.unwrap().view() == (InboundView { sender, recipient, members, request }),
    {
        if sender >= members || recipient >= members || request.candidate != sender { return None; }
        Some(Self { sender, recipient, members, request })
    }
    pub proof fn evidence(&self) requires self.valid(), ensures inbound_valid(self.view()), {}
    pub closed spec fn valid(&self) -> bool { inbound_valid(self.view()) }
    pub fn request(&self) -> (q: VoteRequest) ensures q == self.view().request, { self.request }
    pub fn sender(&self) -> (p: usize) ensures p == self.view().sender, { self.sender }
    pub fn recipient(&self) -> (p: usize) ensures p == self.view().recipient, { self.recipient }
}

}

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
    pub fn new(sender: usize, request: VoteRequest, reply: VoteReply) -> (r: Self)
        ensures r.view().sender == sender, r.view().recipient == request.candidate,
            r.view().request == request, r.view().reply == reply,
    { Self { sender, recipient: request.candidate, request, reply } }
    pub fn body(&self) -> (r: VoteReply) ensures r == self.view().reply, { self.reply }
}
}

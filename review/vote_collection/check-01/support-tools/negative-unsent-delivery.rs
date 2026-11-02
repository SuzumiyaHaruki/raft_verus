#[path = "/home/nitro/Desktop/raft_verus/requirements/support.rs"] mod support;
use support::*;
use vstd::prelude::*;
verus! { proof fn wrong_network_claim() {
    let m = Envelope { group: 1, sender: 0, recipient: 1, body: 7u64 };
    assert(delivered(Seq::empty(), Delivery { send: 0, message: m }));
} }

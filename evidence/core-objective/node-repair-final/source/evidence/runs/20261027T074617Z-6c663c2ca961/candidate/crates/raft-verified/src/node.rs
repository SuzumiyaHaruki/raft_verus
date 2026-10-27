//! SIMULATED offline generator output. Never supplied as experiment context.
use crate::state::{DurableState, InitError, Node, Role};
#[cfg(verus_keep_ghost)]
use crate::initialization::init_result;
use vstd::prelude::*;

verus! {
impl Node {
    pub fn new(id: u64, members: Vec<u64>) -> (result: Result<Self, InitError>)
        ensures init_result(result, id, members@),
    {
        let mut found = false;
        let mut i: usize = 0;
        while i < members.len()
            invariant
                i <= members.len(),
                found == (exists |k: int| 0 <= k < i && members@[k] == id),
                forall |a: int, b: int| 0 <= a < b < i ==> members@[a] != members@[b],
            decreases members.len() - i,
        {
            let mut j: usize = 0;
            while j < i
                invariant
                    i < members.len(), j <= i,
                    forall |a: int| 0 <= a < j ==> members@[a] != members@[i as int],
                decreases i - j,
            {
                if members[j] == members[i] {
                    return Err(InitError::InvalidMembership);
                }
                j += 1;
            }
            if members[i] == id { found = true; }
            i += 1;
        }
        if !found { return Err(InitError::InvalidMembership); }
        proof {
            assert forall |a: int, b: int| 0 <= a < members.len() && 0 <= b < members.len()
                && a != b implies members@[a] != members@[b] by {
                if a < b {} else { assert(b < a); }
            };
        }
        let n = Node {
            id, members,
            hard: DurableState { term: 0, voted_for: None, log: Vec::new() },
            role: Role::Follower,
            commit: 0, delivered: 0, application_cursor: 0,
            incarnation: 0, serial: 0, deadline: 1,
        };
        proof {
            assert(crate::representation::log_view(n.hard.log@) =~= Seq::empty());
        }
        Ok(n)
    }
}
}

use vstd::prelude::*;
verus! {
pub open spec fn agreeing(values: Seq<u64>, through: u64, n: nat) -> nat
    recommends n <= values.len(),
    decreases n,
{
    if n == 0 { 0 } else {
        agreeing(values, through, (n - 1) as nat) + if values[n as int - 1] >= through { 1nat } else { 0nat }
    }
}
pub(super) fn authorize(current: u64, proposed: u64, entry_term: Option<u64>, term: u64, matches: &Vec<u64>) -> (next: u64)
    ensures next == current || next == proposed, next >= current,
        next > current ==> entry_term == Some(term) && matches.len() > 0
            && agreeing(matches@, proposed, matches.len() as nat) > matches.len() / 2,
{
    if proposed <= current || entry_term != Some(term) || matches.len() == 0 { return current; }
    let mut count = 0usize;
    let mut i = 0usize;
    while i < matches.len()
        invariant i <= matches.len(), count <= i,
            count == agreeing(matches@, proposed, i as nat),
        decreases matches.len() - i,
    {
        if matches[i] >= proposed { count += 1; }
        i += 1;
    }
    if count > matches.len() / 2 { proposed } else { current }
}
}

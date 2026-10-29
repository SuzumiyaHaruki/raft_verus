//! Distinct-member vote counting, shared by host and proof.
use vstd::prelude::*;
verus! {
pub open spec fn tally(v: Seq<bool>, n: nat) -> nat
    recommends n <= v.len(), decreases n,
{ if n == 0 {0} else {tally(v,(n-1) as nat) + if v[n as int-1] {1nat} else {0nat}} }
pub fn count(v:&Vec<bool>) -> (n:usize)
    ensures n == tally(v@,v.len() as nat),n <= v.len(),
{
    let mut i=0usize;let mut n=0usize;
    while i<v.len()
        invariant i<=v.len(),n<=i,n == tally(v@,i as nat),
        decreases v.len()-i,
    {
        if v[i] {n+=1;}
        i+=1;
    }
    n
}
pub fn record(v:&mut Vec<bool>,peer:usize) -> (n:usize)
    ensures v@ == (if peer < old(v).len() {old(v)@.update(peer as int,true)} else {old(v)@}),
        n == tally(v@,v.len() as nat),n<=v.len(),
{
    if peer<v.len() {v.set(peer,true);}
    count(v)
}
pub fn empty(members:usize) -> (v:Vec<bool>)
    ensures v.len()==members,forall|j:int|0<=j<v.len() ==> !#[trigger] v@[j],
{
    let mut v=Vec::new();
    while v.len()<members
        invariant v.len()<=members,forall|j:int|0<=j<v.len() ==> !#[trigger] v@[j],
        decreases members-v.len(),
    {v.push(false);}
    v
}
}

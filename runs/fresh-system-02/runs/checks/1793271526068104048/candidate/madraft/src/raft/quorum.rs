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
    let mut v:Vec<bool>=Vec::new();
    while v.len()<members
        invariant v.len()<=members,forall|j:int|0<=j<v.len() ==> !#[trigger] v@[j],
        decreases members-v.len(),
    {v.push(false);}
    v
}
pub proof fn disjoint_bound(a:Seq<bool>,b:Seq<bool>,n:nat)
    requires n<=a.len(),n<=b.len(),forall|j:int|0<=j<n ==> !(#[trigger] a[j] && b[j]),
    ensures tally(a,n)+tally(b,n)<=n,
    decreases n,
{
    if n>0 {
        disjoint_bound(a,b,(n-1) as nat);
        assert(!(a[n as int-1] && b[n as int-1]));
    }
}
pub proof fn intersection(a:Seq<bool>,b:Seq<bool>)
    requires a.len()==b.len(),tally(a,a.len())>a.len()/2,tally(b,b.len())>b.len()/2,
    ensures exists|j:int|0<=j<a.len() && #[trigger] a[j] && b[j],
{
    if !(exists|j:int|0<=j<a.len() && #[trigger] a[j] && b[j]) {
        assert forall|j:int|0<=j<a.len() implies !(#[trigger] a[j] && b[j]) by {}
        disjoint_bound(a,b,a.len());
    }
}
// Conditional theorem: callers still owe the history-backed vote predicate.
pub proof fn certificate_unique(a:Seq<bool>,b:Seq<bool>,ca:nat,cb:nat,
    voted:spec_fn(nat)->Option<nat>)
    requires a.len()==b.len(),tally(a,a.len())>a.len()/2,tally(b,b.len())>b.len()/2,
        forall|j:int|0<=j<a.len() && #[trigger] a[j] ==> voted(j as nat)==Some(ca),
        forall|j:int|0<=j<b.len() && #[trigger] b[j] ==> voted(j as nat)==Some(cb),
    ensures ca==cb,
{
    intersection(a,b);
    let j=choose|j:int|0<=j<a.len() && a[j] && b[j];
}

}

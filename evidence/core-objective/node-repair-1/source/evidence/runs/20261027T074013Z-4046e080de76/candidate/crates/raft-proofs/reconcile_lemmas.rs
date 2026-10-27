//! DRAFT derived local facts about the unchanged candidate `merged` relation.
//! These helpers introduce no assumptions and do not establish global log matching.
use crate::model;
use vstd::prelude::*;

verus! {
pub proof fn merged_empty_preserves_log(old: Seq<model::Entry>, previous: nat, result: Seq<model::Entry>)
    requires model::merged(old, previous, Seq::empty(), result),
    ensures result == old,
{
}

pub proof fn merged_matching_preserves_log(
    old: Seq<model::Entry>, previous: nat, incoming: Seq<model::Entry>, result: Seq<model::Entry>,
)
    requires
        model::merged(old, previous, incoming, result),
        previous + incoming.len() <= old.len(),
        forall |j: int| 0 <= j < incoming.len() ==>
            old[previous as int + j].term == incoming[j].term,
    ensures result == old,
{
}
}

//! Executable generation selection and repair-order guards. File I/O and
//! encoding remain host operations; this is not a disk/crash refinement proof.
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy)]
pub enum Source { Absent, Primary, Backup }

pub fn primary_coherent(base: u64, state_generation: u64, snapshot_generation: Option<u64>, empty_snapshot: bool) -> (r: bool)
    ensures r == (if base == 0 { empty_snapshot } else { snapshot_generation == Some(state_generation) }),
{ if base == 0 { empty_snapshot } else { snapshot_generation == Some(state_generation) } }

pub fn select(primary: Option<u64>, backup: Option<u64>) -> (r: Source)
    ensures
        r is Absent <==> primary.is_none() && backup.is_none(),
        r is Primary ==> primary.is_some() && (backup.is_none() || primary.unwrap() >= backup.unwrap()),
        r is Backup ==> backup.is_some() && (primary.is_none() || backup.unwrap() > primary.unwrap()),
{
    match primary {
        Some(p) => match backup { Some(b) => if b > p { Source::Backup } else { Source::Primary }, None => Source::Primary },
        None => match backup { Some(_) => Source::Backup, None => Source::Absent },
    }
}

pub fn backup_required(selected: u64, backup: Option<u64>) -> (r: bool)
    ensures r == (backup != Some(selected)),
{ backup != Some(selected) }

pub fn next_generation(generation: u64) -> (r: Option<u64>)
    ensures r.is_some() == (generation < u64::MAX),
        r.is_some() ==> r.unwrap() == generation + 1,
{ if generation == u64::MAX { None } else { Some(generation + 1) } }

// This is a conditional local lemma about the actual selection routine. The
// decoder/file-generation correspondence is a remaining host proof obligation.
pub proof fn repair_retains_an_authority(primary: Option<u64>, backup: Option<u64>, selected: u64, source: Source)
    requires
        source is Primary ==> primary == Some(selected),
        source is Backup ==> backup == Some(selected),
        !(source is Absent),
    ensures backup != Some(selected) ==> primary == Some(selected),
{}
}

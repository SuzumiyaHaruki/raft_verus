//! Checked storage sequencing. I/O events are supplied only by the real driver.
//! The filesystem effect of each successful host operation is an explicit
//! environment contract in the parent proof, not inferred from a token.
use super::protocol::{self, WriteVote, VoteImage, Machine};
use super::state::VoteReply;
use vstd::prelude::*;
verus! {
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Create, Write, Sync, Ready, Failed }
#[derive(Clone, Copy, Debug)]
pub enum Action { Create, Write(VoteImage), Sync, Release, Stop }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion { Created, Written, Synced, Failed }

pub struct VoteWrite {
    operation: WriteVote,
    phase: Phase,
}
// No public constructor: only the completed checked flow can produce a receipt.
pub struct WrittenVote { operation: WriteVote }

pub open spec fn next_phase(p: Phase, c: Completion) -> Phase {
    match (p, c) {
        (_, Completion::Failed) => Phase::Failed,
        (Phase::Create, Completion::Created) => Phase::Write,
        (Phase::Write, Completion::Written) => Phase::Sync,
        (Phase::Sync, Completion::Synced) => Phase::Ready,
        _ => Phase::Failed,
    }
}
pub open spec fn action_for(p: Phase, image: VoteImage) -> Action {
    match p {
        Phase::Create => Action::Create,
        Phase::Write => Action::Write(image),
        Phase::Sync => Action::Sync,
        Phase::Ready => Action::Release,
        Phase::Failed => Action::Stop,
    }
}
impl VoteWrite {
    pub closed spec fn phase(&self) -> Phase { self.phase }
    pub closed spec fn id(&self) -> u64 { self.operation.number() }
    pub closed spec fn image(&self) -> VoteImage { self.operation.contents() }
    pub fn new(operation: WriteVote) -> (r: Self)
        ensures r.phase() == Phase::Create, r.id() == operation.number(),
            r.image() == operation.contents(),
    { Self { operation, phase: Phase::Create } }

    pub fn action(&self) -> (r: Action)
        ensures r == action_for(self.phase(), self.image()),
    {
        match self.phase {
            Phase::Create => Action::Create,
            Phase::Write => Action::Write(self.operation.image()),
            Phase::Sync => Action::Sync,
            Phase::Ready => Action::Release,
            Phase::Failed => Action::Stop,
        }
    }
    pub fn complete(&mut self, event: Completion)
        ensures self.id() == old(self).id(), self.image() == old(self).image(),
            self.phase() == next_phase(old(self).phase(), event),
    {
        self.phase = match (self.phase, event) {
            (_, Completion::Failed) => Phase::Failed,
            (Phase::Create, Completion::Created) => Phase::Write,
            (Phase::Write, Completion::Written) => Phase::Sync,
            (Phase::Sync, Completion::Synced) => Phase::Ready,
            _ => Phase::Failed,
        };
    }
    pub fn release(self) -> (r: Option<WrittenVote>)
        ensures r.is_some() == (self.phase() == Phase::Ready),
            r.is_some() ==> r.unwrap().id() == self.id() && r.unwrap().image() == self.image(),
    {
        if matches!(self.phase, Phase::Ready) { Some(WrittenVote { operation: self.operation }) }
        else { None }
    }
}
impl WrittenVote {
    pub closed spec fn id(&self) -> u64 { self.operation.number() }
    pub closed spec fn image(&self) -> VoteImage { self.operation.contents() }
    pub fn matches(&self, id: u64, image: VoteImage) -> (r: bool)
        ensures r == (id == self.id() && image == self.image()),
    {
        let actual = self.operation.image();
        self.operation.sequence() == id && actual.version == image.version
            && actual.term == image.term && actual.voted_for == image.voted_for
            && actual.me == image.me && actual.members == image.members
    }
    pub fn release_reply(self, machine: &mut Machine) -> (r: Option<VoteReply>)
        requires old(machine).invariant(),
        ensures machine.invariant(), machine.state() == old(machine).state(),
            protocol::complete_step(old(machine).view(), self.id(), self.image(), machine.view(), r),
            r.is_some() ==> self.image() == protocol::image(machine.state()),
    { machine.complete_vote(self.operation) }

}
}

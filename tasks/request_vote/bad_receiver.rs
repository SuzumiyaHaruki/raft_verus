//! MAINTENANCE FIXTURE ONLY: always grants, accepts illegal identities, ignores
//! stale terms/logs/operation completion identity. NO proof or correctness claim.
//! Used inside the actual prepared MadRaft handler, never a second protocol host.
use super::vote_api::*;
pub struct Core { state: View }
impl Core {
    pub fn new(c: Config, i: Initial) -> Option<Self> {
        Some(Self{state:View{config:c,state:i.state,role:i.role,next:0,pending:None,halted:false}})
    }
    pub fn recover(c: Config, r: Record) -> Option<Self> {
        let mut x=Self::new(c,Initial{state:r.state,role:Role::Follower}).unwrap();
        x.state.next=r.op.saturating_add(1); Some(x)
    }
    pub fn inspect(&self) -> View { self.state }
    pub fn receive(&mut self,r: Request) -> Receive {
        self.state.state.term=r.term;
        self.state.state.voted_for=Some(r.candidate); // intentionally overwrites votes
        let w=Write{record:Record{op:self.state.next,state:self.state.state},request:r,
            reply:Reply{node:self.state.config.node,term:r.term,candidate:r.candidate,granted:true}};
        self.state.next=self.state.next.saturating_add(1); self.state.pending=Some(w); Receive::Save(w)
    }
    pub fn complete(&mut self,_n: Completion) -> Option<Reply> {
        self.state.pending.take().map(|w|w.reply) // intentionally ignores success and ID
    }
}

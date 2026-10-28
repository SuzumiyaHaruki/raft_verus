//! DRAFT generation slot. No election, replication, commit or recovery implementation is supplied.
use crate::{io::*, state::Node};
use vstd::prelude::*;
verus! {
impl Node {
    /// The cluster host calls only this serial Event -> Effects interface.
    /// `step` remains the older coordinator-only compatibility API.
    pub fn handle_event(&mut self,event:Event)->(result:Result<Vec<Effect>,InputError>)
        requires old(self).wf(),
        ensures self.wf(),
    {
        let _=event;
        Err(InputError::UnsupportedEvent)
    }
}
}

//! DRAFT representation used by the independent local contract consumer.
//! This preserves every entry term, command tag, request field and command byte.
use crate::model;
use crate::reconcile::{Command, Entry};
use vstd::prelude::*;

verus! {
pub open spec fn entry_view(entry: Entry) -> model::Entry {
    model::Entry { term: entry.term as nat, command: match entry.command {
        Command::Noop => model::Command::Noop,
        Command::Client { request, bytes } => model::Command::Client {
            request: (request.client as nat, request.sequence as nat), value: bytes@,
        },
    } }
}

pub open spec fn log_view(log: Seq<Entry>) -> Seq<model::Entry> {
    log.map(|i: int, e: Entry| entry_view(e))
}
}

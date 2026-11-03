//! Checks the invariant of the actual Core and its cross-instance composition.
#[path="../../tasks/election_recovery/api.rs"] mod vote_api;
#[path="../src/raft/vote_core.rs"] mod vote_core;
#[path="vote_lemmas.rs"] mod vote_lemmas;

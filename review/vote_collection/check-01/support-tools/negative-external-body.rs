
use vstd::prelude::*;
verus! {
#[verifier::external_body]
pub fn incorrect(n:u64) -> (r:u64) ensures r == n, { 7 }
}

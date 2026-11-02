
use vstd::prelude::*;
verus! {
pub fn increment(n: u64) -> (r: u64)
    requires n < u64::MAX,
    ensures r == n + 1,
{ n + 1 }
}
#[cfg(test)] mod tests {
    #[test] fn actual_increment() { assert_eq!(super::increment(2),3); }
}

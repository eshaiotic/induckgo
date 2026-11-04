//! Allows one redirect per request, so two policies that undo each other cannot loop forever.

use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct RedirectBudget {
    capacity: usize,
    spent: Mutex<SpentRequests>,
}

#[derive(Default)]
struct SpentRequests {
    ids: HashSet<u64>,
    oldest_first: VecDeque<u64>,
}

impl RedirectBudget {
    /// Remembers the last `capacity` requests that were redirected; older ones are forgotten.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self { capacity: capacity.max(1), spent: Mutex::new(SpentRequests::default()) }
    }

    /// True the first time it is asked about `request_id`, false afterwards.
    pub fn allow(&self, request_id: u64) -> bool {
        let mut spent = self.spent.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !spent.ids.insert(request_id) {
            return false;
        }
        spent.oldest_first.push_back(request_id);
        if spent.oldest_first.len() > self.capacity
            && let Some(forgotten) = spent.oldest_first.pop_front()
        {
            spent.ids.remove(&forgotten);
        }
        true
    }
}

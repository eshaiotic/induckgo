//! One redirect per request, so two policies that undo each other cannot loop.

use induckgo_core::network::redirect_budget::RedirectBudget;

struct BudgetScenario {
    budget: RedirectBudget,
}

impl BudgetScenario {
    fn given_budget_remembering(capacity: usize) -> Self {
        Self { budget: RedirectBudget::new(capacity) }
    }

    fn when_asked_for(&self, request_id: u64) -> bool {
        self.budget.allow(request_id)
    }
}

#[test]
fn should_allow_the_first_redirect_of_a_request() {
    assert!(BudgetScenario::given_budget_remembering(8).when_asked_for(1));
}

#[test]
fn should_refuse_a_second_redirect_of_the_same_request() {
    let scenario = BudgetScenario::given_budget_remembering(8);
    scenario.when_asked_for(1);
    assert!(!scenario.when_asked_for(1));
}

#[test]
fn should_allow_a_redirect_for_a_different_request() {
    let scenario = BudgetScenario::given_budget_remembering(8);
    scenario.when_asked_for(1);
    assert!(scenario.when_asked_for(2));
}

#[test]
fn should_forget_the_oldest_request_when_the_memory_is_full() {
    let scenario = BudgetScenario::given_budget_remembering(2);
    scenario.when_asked_for(1);
    scenario.when_asked_for(2);
    scenario.when_asked_for(3);
    assert!(scenario.when_asked_for(1), "request 1 should have been forgotten");
}

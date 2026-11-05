//! Guard's page script runs where the page cannot reach it and hides each selector as a rule of its own.

use induckgo_core::engine::injection_point::InjectionPoint;
use induckgo_core::engine::script_world::ScriptWorld;
use induckgo_core::guard::guard_page_script::GuardPageScript;

#[test]
fn should_run_before_the_page_in_an_isolated_world_of_every_frame() {
    let script = GuardPageScript::page_script("/* client */");
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
    assert_eq!(script.world, ScriptWorld::Isolated);
    assert!(script.runs_in_all_frames);
}

#[test]
fn should_place_the_channel_client_in_front_of_the_script() {
    let script = GuardPageScript::page_script("/* client */");
    assert!(script.source.starts_with("/* client */\n"));
}

#[test]
fn should_check_every_selector_before_inserting_it_as_a_rule_of_its_own() {
    let source = GuardPageScript::source();
    assert!(source.contains("CSS.supports(`selector(${selector})`)"));
    assert!(source.contains("sheet.insertRule("));
}

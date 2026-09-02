//! Every event the client documents in Blizzard_APIDocumentationGenerated must
//! be known to the simulator's event validator. A documented event that the
//! tables lack turns into `Frame:RegisterEvent(): Attempt to register unknown
//! event` at the Blizzard call site, and when that call sits in an OnLoad --
//! `GameMenuFrame` registers `EXTERNAL_EVENT_LAUNCH_URL_FAILED` there -- the
//! rest of that handler is lost.

use std::fs;

use wow_ui_sim::event::{is_callback_event, is_restricted_event, is_valid_event};

fn documented_events() -> Option<Vec<String>> {
    let dir = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .ok()?
        .join("Blizzard_APIDocumentationGenerated");
    let entries = fs::read_dir(&dir).ok()?;
    let mut events = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("lua") {
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        // Event entries look like:  LiteralName = "EXTERNAL_EVENT_LAUNCH_URL_FAILED",
        for line in source.lines() {
            let Some(rest) = line.trim_start().strip_prefix("LiteralName = \"") else {
                continue;
            };
            if let Some(name) = rest.split('"').next() {
                events.push(name.to_string());
            }
        }
    }
    events.sort();
    events.dedup();
    Some(events)
}

#[test]
fn every_documented_client_event_is_known_to_the_validator() {
    let Some(events) = documented_events() else {
        eprintln!("Skipping: Blizzard_APIDocumentationGenerated not in the Blizzard UI cache");
        return;
    };
    assert!(
        events.len() > 1000,
        "expected the generated documentation to list well over a thousand events, found {}",
        events.len()
    );
    // Restricted and callback-only events are known to the client but closed to
    // Frame:RegisterEvent; they count as known here.
    let unknown: Vec<&String> = events
        .iter()
        .filter(|e| !is_valid_event(e) && !is_restricted_event(e) && !is_callback_event(e))
        .collect();
    assert!(
        unknown.is_empty(),
        "{} documented events are unknown to src/event/valid_events*.rs: {unknown:?}",
        unknown.len()
    );
}

#[test]
fn game_menu_frame_external_url_event_is_registerable() {
    // GameMenuFrame.lua:54 registers this in OnLoad; without it the handler aborts.
    assert!(
        wow_ui_sim::event::is_registerable_event("EXTERNAL_EVENT_LAUNCH_URL_FAILED"),
        "EXTERNAL_EVENT_LAUNCH_URL_FAILED is documented in ExternalEventURLDocumentation.lua"
    );
}

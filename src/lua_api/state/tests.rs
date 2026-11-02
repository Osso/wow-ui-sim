use super::*;
use std::time::Duration;

#[test]
fn empty_runtime_state_new_seeds_expected_runtime_defaults() {
    let state = EmptyRuntimeState::new();

    assert_eq!(state.next_report_token, INITIAL_REPORT_TOKEN);
    assert_eq!(state.next_anim_group_id, INITIAL_ANIM_GROUP_ID);
    assert_eq!(state.next_cast_id, INITIAL_CAST_ID);
    assert_eq!(state.screen_width, DEFAULT_SCREEN_WIDTH);
    assert_eq!(state.screen_height, DEFAULT_SCREEN_HEIGHT);
    assert_eq!(state.screen_kind, ScreenKind::Game);
    assert!(!state.is_logged_in);
    assert!(!state.screen_first_displayed);
    assert!(state.focused_frame_id.is_none());
    assert!(state.hovered_frame.is_none());
    assert!(state.saved_account_name.is_empty());
    assert!(state.saved_account_list.is_empty());
    assert!(state.start_time.elapsed() < Duration::from_secs(1));
}

#[test]
fn post_event_workaround_marker_is_one_shot() {
    let session = tempfile::tempdir().unwrap();
    let mut state = SimState::with_session_wtf_path(session.path().to_path_buf());

    assert!(!state.post_event_workarounds_applied);
    assert!(state.mark_post_event_workarounds_applied());
    assert!(state.post_event_workarounds_applied);
    assert!(!state.mark_post_event_workarounds_applied());
}

#[test]
fn session_directory_selects_cvar_reads_and_writes() {
    let session = tempfile::tempdir().unwrap();
    let path = session.path().join("cvars.json");
    std::fs::write(&path, r#"{"sessionstorageprobe":"37"}"#).unwrap();

    let state = SimState::with_session_wtf_path(session.path().to_path_buf());

    assert_eq!(state.session_wtf_path, session.path());
    assert_eq!(
        state.cvars.get("sessionStorageProbe").as_deref(),
        Some("37")
    );
    assert!(state.cvars.set("sessionStorageProbe", "41"));
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(persisted["sessionstorageprobe"], "41");
}

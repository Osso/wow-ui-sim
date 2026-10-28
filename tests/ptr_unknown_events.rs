#![cfg(feature = "client-ptr")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn ptr_original_consumers_register_documented_events() {
    let env = WowLuaEnv::new().unwrap();
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    for (_, toc) in wow_ui_sim::loader::discover_blizzard_addons(&ui) {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    }
    for (frame, event) in [
        (
            "ActionBarButtonActionCombatRangeCheckFrame",
            "ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE",
        ),
        (
            "EventRegistry.frameEventFrame",
            "EXTERNAL_EVENT_LAUNCH_URL_RESULT",
        ),
        ("StatusTrackingBarManager", "GAME_MODE_EXP_BAR_DATA_UPDATED"),
        ("MicroMenuContainer", "GAME_RULES_CHANGED"),
        ("PlayerFrame", "PLAYER_PVP_FLAG_CHANGED"),
        (
            "SecureTransferDialog",
            "SECURE_TRANSFER_CONFIRM_VC_PRODUCT_PURCHASE",
        ),
        ("ShardTransferImminentFrame", "SHARD_TRANSFER_CANCELLED"),
        (
            "ShardTransferImminentMinimizeButton",
            "SHARD_TRANSFER_CANCELLED",
        ),
        ("ClassTrainerFrame", "UNIT_PET_TRAINING_POINTS"),
        ("ChannelFrame", "VOICE_CHAT_VOICE_PROVIDER_CHANGED"),
    ] {
        env.exec(&format!("assert({frame}:IsEventRegistered('{event}'), '{frame}: {event}'); {frame}:RegisterEvent('{event}'); assert({frame}:IsEventRegistered('{event}'))"))
            .unwrap_or_else(|error| panic!("{frame}: {event}: {error}"));
    }
    let error = env
        .exec("CreateFrame('Frame'):RegisterEvent('PTR_INVENTED_UNDECLARED_EVENT')")
        .unwrap_err();
    assert!(error.to_string().contains("unknown event"), "{error}");
}

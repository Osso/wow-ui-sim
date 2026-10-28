#![cfg(feature = "client-mists")]

use std::path::PathBuf;
use std::process::Command;
use wow_ui_sim::loader::{discover_blizzard_startup_addons_for_screen, load_startup_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

fn assert_watched_reputation_bar(faction_id: u32, name: &str, value: u32, max: u32) {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = WowLuaEnv::new().unwrap();
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        addons
            .iter()
            .any(|addon| addon.toc_path.ends_with("Blizzard_ActionBar_Classic.toc"))
    );
    for addon in addons {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env.state()
        .borrow_mut()
        .major_factions
        .get_mut(&(faction_id as i64))
        .expect("seeded major faction")
        .renown_level_threshold = max as i32;
    let other_id = if faction_id == 2590 { 2570 } else { 2590 };
    env.exec(&format!(
        r#"
        C_Reputation.SetWatchedFactionByID({other_id})
        assert(C_Reputation.GetWatchedFactionData().factionID == {other_id}, 'initial watch')
        C_Reputation.SetWatchedFactionByID({faction_id})
        local bar = CreateFrame('Frame', nil, UIParent, 'ReputationStatusBarTemplate')
        assert(bar.Update == ReputationStatusBarMixin.Update)
        local info = debug.getinfo(bar.Update, 'S')
        assert(info.source:gsub('\\', '/'):match('Blizzard_ActionBar/Shared/ReputationBar.lua$'))
        bar:Update()
        assert(bar.factionID == {faction_id} and bar.value == {value} and bar.max == {max},
            ('bar: id=%s value=%s max=%s'):format(tostring(bar.factionID), tostring(bar.value), tostring(bar.max)))
        assert(bar.name == '{name} %d / %d', 'bar name: '..tostring(bar.name))
        local data = C_Reputation.GetWatchedFactionData()
        assert(data.factionID == {faction_id} and data.isAccountWide == false, 'watched data')
        assert(C_Reputation.GetFactionDataByID({faction_id}).isAccountWide == false, 'faction data')
        assert(C_Reputation.IsAccountWideReputation({faction_id}) == false, 'account wide query')
        assert(REPUTATION_STATUS_BAR_LABEL_ACCOUNT_WIDE == nil, 'unexpected label')
    "#
    ))
    .unwrap();
}

#[test]
fn mists_reputation_bar_watches_dornogal() {
    assert_watched_reputation_bar(2590, "Council of Dornogal", 8200, 12000);
    assert_watched_reputation_bar(2590, "Council of Dornogal", 8200, 21000);
}

#[test]
fn mists_reputation_bar_watches_hallowfall() {
    assert_watched_reputation_bar(2570, "Hallowfall Arathi", 4500, 12000);
    assert_watched_reputation_bar(2570, "Hallowfall Arathi", 4500, 21000);
}

#[test]
fn mists_character_and_reputation_panels_have_backing_data() {
    let lua = r#"
        ToggleCharacter("PaperDollFrame")
        local head = CharacterHeadSlot
        if not head or head:GetID() ~= 1 then
            error("CharacterHeadSlot was not wired to inventory slot 1")
        end
        if not head.icon or not head.icon:GetTexture() then
            error("CharacterHeadSlot has no icon texture")
        end

        ToggleCharacter("ReputationFrame")
        if ReputationFrame_Update then
            ReputationFrame_Update()
        end
        if not ReputationBar1 or not ReputationBar1:IsShown() then
            error("ReputationBar1 was not shown")
        end
        local name = ReputationBar1FactionName and ReputationBar1FactionName:GetText()
        if type(name) ~= "string" or name == "" then
            error("ReputationBar1 has no faction name")
        end
    "#;

    let output = Command::new(wow_sim_binary())
        .env("WOW_SIM_NO_SAVED_VARS", "1")
        .env("WOW_SIM_NO_ADDONS", "1")
        .arg("--exec-lua")
        .arg(lua)
        .arg("dump-tree")
        .output()
        .expect("wow-sim dump-tree should run");

    assert!(
        output.status.success(),
        "Mists character/reputation probe failed with status {:?}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Lua error"),
        "Mists character/reputation probe emitted Lua errors:\n{stderr}"
    );
}

//! Integration tests for legacy talent/skill probe and spell-tab globals.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── Talent tab / talent info ──────────────────────────────────────────────────

#[test]
fn get_num_talent_tabs_returns_zero() {
    let env = env();
    let n: i32 = env.eval("return GetNumTalentTabs()").unwrap();
    assert_eq!(n, 0);
}

#[test]
fn get_talent_info_returns_nil() {
    let env = env();
    let v: Option<String> = env.eval("return GetTalentInfo(1, 1)").unwrap();
    assert_eq!(v, None);
}

#[test]
fn get_talent_info_by_specialization_returns_nil() {
    let env = env();
    let v: Option<String> = env
        .eval("return GetTalentInfoBySpecialization(1, 1, 1)")
        .unwrap();
    assert_eq!(v, None);
}

// ── Spellbook tabs ────────────────────────────────────────────────────────────

#[test]
fn get_num_spell_tabs_returns_one() {
    let env = env();
    let n: i32 = env.eval("return GetNumSpellTabs()").unwrap();
    assert_eq!(n, 1);
}

#[test]
fn get_spell_tab_info_returns_class_name_and_spec_id() {
    let env = env();
    // Seeded player is Paladin (class_index 2), Retribution spec
    // (active_spec_index 2) by default — set Retribution specifically.
    env.state().borrow_mut().player.active_spec_index = 70;
    let (name, _icon, offset, num_spells, is_guild, spec_id): (
        String,
        String,
        i32,
        i32,
        bool,
        i32,
    ) = env.eval("return GetSpellTabInfo(1)").unwrap();
    assert_eq!(name, "Paladin");
    assert_eq!(offset, 0);
    assert_eq!(num_spells, 0);
    assert!(!is_guild);
    assert_eq!(spec_id, 70);
}

#[test]
fn get_spell_tab_info_nil_for_out_of_range_index() {
    let env = env();
    let v: Option<String> = env.eval("return GetSpellTabInfo(5)").unwrap();
    assert_eq!(v, None);
}

#[test]
fn get_specialization_info_for_class_id_stops_after_class_specs() {
    let env = env();
    let (first_spec_id, fourth_spec_id): (i32, Option<i32>) = env
        .eval("return GetSpecializationInfoForClassID(2, 1), GetSpecializationInfoForClassID(2, 4)")
        .unwrap();

    assert_eq!(first_spec_id, 65);
    assert_eq!(fourth_spec_id, None);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn specialization_class_id_selects_class_without_mutating_player() {
    let env = env();
    env.state().borrow_mut().player.class_index = 2;
    env.state().borrow_mut().player.active_spec_index = 3;
    env.exec(
        r#"
        local function check(classID, expectedID, expectedName)
            local function query()
                return C_SpecializationInfo.GetSpecializationInfo(1, false, false, nil, nil, nil, classID)
            end
            assert(select('#', query()) == 10)
            local id, name, description, icon, role, stat, points, background, preview, unlocked = query()
            assert(id == expectedID and name == expectedName)
            assert(type(description) == 'string' and type(icon) == 'number')
            assert(type(role) == 'string' and type(stat) == 'number')
            assert(points == 0 and background == nil and preview == 0 and unlocked == true)
        end
        check(8, 62, 'Arcane')
        check(2, 65, 'Holy')
        check(8, 62, 'Arcane')
        assert(C_SpecializationInfo.GetSpecializationInfo(1) == 65)
        assert(C_SpecializationInfo.GetSpecializationInfo(1, false, false, nil, nil, nil, nil) == 65)
        assert(C_SpecializationInfo.GetSpecializationInfo(99) == 70)
        assert(C_SpecializationInfo.GetSpecialization() == 3)
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.player.class_index, 2);
    assert_eq!(state.player.active_spec_index, 3);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn specialization_class_id_invalid_explicit_selection_returns_default_tuple() {
    let env = env();
    env.exec(
        r#"
        local function check(index, classID)
            local function query()
                return C_SpecializationInfo.GetSpecializationInfo(index, false, false, nil, nil, nil, classID)
            end
            assert(select('#', query()) == 10)
            local id, name, description, icon, role, stat, points, background, preview, unlocked = query()
            assert(id == 0 and name == nil and description == nil and icon == nil)
            assert(role == nil and stat == nil and points == 0)
            assert(background == nil and preview == 0 and unlocked == true)
        end
        check(1, 999)
        check(1, 0)
        check(1, -1)
        check(1, 8.5)
        check(0, 8)
        check(-1, 8)
        check(4, 8)
        check(1.5, 8)
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn specialization_class_id_cooldown_viewer_uses_cross_class_tag() {
    let env = env();
    env.state().borrow_mut().player.class_index = 2;
    let addons = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ));
    let source =
        std::fs::read_to_string(addons.join("Blizzard_CooldownViewer/CooldownViewerUtil.lua"))
            .expect("cached Blizzard CooldownViewerUtil source");
    env.exec(&source)
        .expect("unmodified CooldownViewerUtil loads");
    let mage: String = env
        .eval("return CooldownViewerUtil.GetClassAndSpecTagText(81)")
        .unwrap();
    let paladin: String = env
        .eval("return CooldownViewerUtil.GetClassAndSpecTagText(21)")
        .unwrap();
    assert_eq!(mage, "Mage - Arcane");
    assert_eq!(paladin, "Paladin - Holy");
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn specialization_class_id_preserves_non_retail_player_selection() {
    let env = env();
    env.state().borrow_mut().player.class_index = 2;
    let id: i32 = env
        .eval(
            "return C_SpecializationInfo.GetSpecializationInfo(1, false, false, nil, nil, nil, 8)",
        )
        .unwrap();
    assert_eq!(id, 65);
}

// ── PvP talents ───────────────────────────────────────────────────────────────

#[test]
fn get_pvp_talent_slot_info_returns_table_for_valid_slots() {
    let env = env();
    let (slot_index, enabled, locked, selected): (i32, bool, bool, i32) = env
        .eval(
            r#"
            local t = GetPvpTalentSlotInfo(2)
            return t.slotIndex, t.enabled, t.locked, t.selectedTalentID
            "#,
        )
        .unwrap();
    assert_eq!(slot_index, 2);
    assert!(enabled);
    assert!(!locked);
    assert_eq!(selected, 0);
}

#[test]
fn get_pvp_talent_slot_info_nil_for_out_of_range_slot() {
    let env = env();
    let v: Option<String> = env.eval("return GetPvpTalentSlotInfo(4)").unwrap();
    assert_eq!(v, None);
}

// ── Arena opponent spec ───────────────────────────────────────────────────────

#[test]
fn get_arena_opponent_spec_zero() {
    let env = env();
    let spec_id: i32 = env.eval("return GetArenaOpponentSpec(1)").unwrap();
    assert_eq!(spec_id, 0);
}

// ── Skill lines (legacy) ──────────────────────────────────────────────────────

#[test]
fn get_num_skill_lines_zero() {
    let env = env();
    let n: i32 = env.eval("return GetNumSkillLines()").unwrap();
    assert_eq!(n, 0);
}

#[test]
fn get_skill_line_info_nil() {
    let env = env();
    let v: Option<String> = env.eval("return GetSkillLineInfo(1)").unwrap();
    assert_eq!(v, None);
}

#[test]
fn get_selected_skill_zero() {
    let env = env();
    let n: i32 = env.eval("return GetSelectedSkill()").unwrap();
    assert_eq!(n, 0);
}

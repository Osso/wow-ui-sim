//! Behavior pin: `SpellFlyout:Toggle(button, flyoutID, ...)` opens the
//! hidden flyout, asks the flyout API for slot data, creates one
//! `SpellFlyoutPopupButton` per visible known spell, and closes again when
//! toggled from the same button.

use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::loader::BlizzardAddonOverride;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{SpellFlyoutInfo, SpellFlyoutSlot};

const ROOT: &str = "Blizzard_ActionBar";
#[cfg(feature = "client-ptr")]
const STARTUP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[BlizzardAddonOverride {
    addon: ROOT,
    extra_roots: &["Blizzard_GamepadSharedUtility"],
}];
#[cfg(not(feature = "client-ptr"))]
const STARTUP_OVERRIDES: &[BlizzardAddonOverride<'static>] = &[];
const FLYOUT_ID: u32 = 7001;
const FIRST_SPELL_ID: u32 = 19750;
const SECOND_SPELL_ID: u32 = 35395;

post_wrath_ui_fixture! {
#[test]
fn spell_flyout_toggle_shows_populates_and_hides_from_same_button() {
    test_timeout! {
    with_blizzard_addon_startup_shape(&[ROOT], STARTUP_OVERRIDES, |env, _loaded| {
        seed_spell_flyout(env);
        install_test_flyout_button(env);

        assert_flyout_hidden_by_default(env);
        toggle_spell_flyout(env);
        assert_flyout_shown_with_seeded_spells(env);
        toggle_spell_flyout(env);
        assert_flyout_hidden_after_second_toggle(env);
    });
    }
}
}

fn seed_spell_flyout(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.known_spells.insert(FIRST_SPELL_ID);
    state.known_spells.insert(SECOND_SPELL_ID);
    state.spell_flyouts.insert(
        FLYOUT_ID,
        SpellFlyoutInfo {
            name: "Test flyout".to_string(),
            description: "Seeded ActionBar flyout".to_string(),
            is_known: true,
            slots: vec![
                SpellFlyoutSlot {
                    spell_id: FIRST_SPELL_ID,
                    override_spell_id: FIRST_SPELL_ID,
                    is_known: true,
                    spell_name: "Flash of Light".to_string(),
                    spec_id: 0,
                },
                SpellFlyoutSlot {
                    spell_id: SECOND_SPELL_ID,
                    override_spell_id: SECOND_SPELL_ID,
                    is_known: true,
                    spell_name: "Crusader Strike".to_string(),
                    spec_id: 0,
                },
            ],
        },
    );
}

fn install_test_flyout_button(env: &WowLuaEnv) {
    let installed: bool = env
        .eval(
            r#"
            local button = CreateFrame(
                "Button",
                "SpellFlyoutBehaviorButton",
                UIParent,
                "FlyoutButtonTemplate"
            )
            button:SetSize(36, 36)
            button:SetPoint("CENTER")
            button:SetPopupDirection("RIGHT")
            button:SetPopup(SpellFlyout)
            return SpellFlyoutBehaviorButton ~= nil
                and button:GetPopup() == SpellFlyout
                and button:GetPopupDirection() == "RIGHT"
            "#,
        )
        .expect("test flyout button installation must run cleanly");
    assert!(installed, "test flyout button must be available globally");
}

fn toggle_spell_flyout(env: &WowLuaEnv) {
    let toggled: bool = env
        .eval(&format!(
            "SpellFlyout:Toggle(SpellFlyoutBehaviorButton, {FLYOUT_ID}, true, 0, false); return true"
        ))
        .expect("SpellFlyout:Toggle must run cleanly");
    assert!(
        toggled,
        "SpellFlyout:Toggle must complete without Lua errors"
    );
}

fn assert_flyout_hidden_by_default(env: &WowLuaEnv) {
    let is_shown: bool = env
        .eval("return SpellFlyout:IsShown() == true")
        .expect("SpellFlyout default visibility probe must run cleanly");
    assert!(!is_shown, "SpellFlyout must start hidden");
}

fn assert_flyout_shown_with_seeded_spells(env: &WowLuaEnv) {
    let (is_shown, first_id, second_id, third_visible): (bool, i64, i64, bool) = env
        .eval(
            r#"
            local first = SpellFlyoutPopupButton1
            local second = SpellFlyoutPopupButton2
            local third = SpellFlyoutPopupButton3
            return SpellFlyout:IsShown() == true,
                   first and first:IsShown() and first.spellID or -1,
                   second and second:IsShown() and second.spellID or -1,
                   third and third:IsShown() == true or false
            "#,
        )
        .expect("SpellFlyout populated-button probe must run cleanly");

    assert!(is_shown, "first toggle must show SpellFlyout");
    assert_eq!(first_id, FIRST_SPELL_ID as i64);
    assert_eq!(second_id, SECOND_SPELL_ID as i64);
    assert!(
        !third_visible,
        "flyout must render exactly two seeded spell buttons"
    );
}

fn assert_flyout_hidden_after_second_toggle(env: &WowLuaEnv) {
    let is_shown: bool = env
        .eval("return SpellFlyout:IsShown() == true")
        .expect("SpellFlyout post-toggle visibility probe must run cleanly");
    assert!(
        !is_shown,
        "second toggle from the same flyout button must hide SpellFlyout"
    );
}

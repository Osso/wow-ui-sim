//! Bounded supplied-state contracts, not native capability or cooldown policy.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SpellCooldownState;

#[test]
fn item_cooldown_projects_supplied_records_on_ptr_and_forever() {
    let env = WowLuaEnv::new().unwrap();
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        assert!(
            env.eval::<bool>("return rawget(C_Spell, 'GetItemCooldown') == nil")
                .unwrap()
        );
        return;
    }
    assert_populated_item_cooldown(&env);
    assert_zero_item_cooldown(&env);
    assert_removed_item_cooldown_and_isolation(&env);
}

fn assert_populated_item_cooldown(env: &WowLuaEnv) {
    env.exec("assert(C_Spell.GetItemCooldown(999999) == nil)")
        .unwrap();
    env.state().borrow_mut().item_cooldowns.insert(
        6948,
        SpellCooldownState {
            start: 11.0,
            duration: 31.0,
        },
    );
    env.exec(
        r#"
        assert(select('#', C_Spell.GetItemCooldown(6948)) == 1)
        local info = C_Spell.GetItemCooldown(6948)
        assert(info.startTime == 11 and info.duration == 31)
        assert(info.isEnabled == true and info.isActive == true)
        assert(info.modRate == 1)
        assert(info.activeCategory == nil and info.isOnGCD == nil)
        info.duration = 99
        assert(C_Spell.GetItemCooldown(6948).duration == 31)
    "#,
    )
    .unwrap();
}

fn assert_zero_item_cooldown(env: &WowLuaEnv) {
    env.state().borrow_mut().item_cooldowns.insert(
        6948,
        SpellCooldownState {
            start: 0.0,
            duration: 0.0,
        },
    );
    env.exec(
        r#"
        local info = C_Spell.GetItemCooldown(6948)
        assert(info.startTime == 0 and info.duration == 0)
        assert(info.isActive == false)
    "#,
    )
    .unwrap();
}

fn assert_removed_item_cooldown_and_isolation(env: &WowLuaEnv) {
    env.state().borrow_mut().item_cooldowns.remove(&6948);
    env.exec("assert(C_Spell.GetItemCooldown(6948) == nil)")
        .unwrap();
    let other = WowLuaEnv::new().unwrap();
    other
        .exec("assert(C_Spell.GetItemCooldown(6948) == nil)")
        .unwrap();
}

#[test]
fn graveyard_query_projects_supplied_availability_without_cross_environment_leaks() {
    let env = WowLuaEnv::new().unwrap();
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        assert!(env.eval::<bool>("return CanPortGraveyard == nil").unwrap());
        return;
    }
    env.exec("assert(CanPortGraveyard() == false and select('#', CanPortGraveyard()) == 1)")
        .unwrap();
    env.state().borrow_mut().player.can_port_graveyard = true;
    env.exec("assert(CanPortGraveyard() == true)").unwrap();
    let other = WowLuaEnv::new().unwrap();
    other.exec("assert(CanPortGraveyard() == false)").unwrap();
    env.state().borrow_mut().player.can_port_graveyard = false;
    env.exec("assert(CanPortGraveyard() == false)").unwrap();
}

#[cfg(feature = "client-ptr")]
mod original_consumers {
    use super::*;
    use sha2::{Digest, Sha256};

    fn load_original_consumer(addon: &str, relative: &str, digest: &str) -> WowLuaEnv {
        let root = crate::common::blizzard_addon_harness::blizzard_ui_dir();
        let bytes = std::fs::read(root.join(relative)).expect("read original PTR consumer");
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), digest);
        let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
            &root,
            &[addon],
            &[],
        );
        assert!(
            loaded.iter().any(|name| name == addon),
            "Game root not selected"
        );
        env
    }

    #[test]
    fn collections_original_item_cooldown_tracks_supplied_records() {
        let env = load_original_consumer(
            "Blizzard_Collections",
            "Blizzard_Collections/Shared/Blizzard_CollectionTemplates.lua",
            "99aa2918d9a923c421ec33660863e34b5528416b53422c903437c1b6bb591148",
        );
        env.exec(r#"
            local info = debug.getinfo(CollectionsSpellButton_UpdateCooldown, "S")
            assert(info.what == "Lua")
            assert(info.source:gsub("\\", "/"):match("Blizzard_Collections/Shared/Blizzard_CollectionTemplates.lua$"))
            I4ItemButton = CreateFrame("Button", nil, UIParent)
            I4ItemButton.itemID = 6948
            I4ItemButton.cooldown = CreateFrame("Cooldown", nil, I4ItemButton)
            I4ItemButton.cooldown:Show()
            CollectionsSpellButton_UpdateCooldown(I4ItemButton)
            assert(not I4ItemButton.cooldown:IsShown())
        "#).unwrap();
        env.state().borrow_mut().item_cooldowns.insert(
            6948,
            SpellCooldownState {
                start: 11.0,
                duration: 31.0,
            },
        );
        env.exec(
            r#"
            CollectionsSpellButton_UpdateCooldown(I4ItemButton)
            assert(I4ItemButton.cooldown:IsShown())
            local start, duration = I4ItemButton.cooldown:GetCooldownTimes()
            assert(start == 11000 and duration == 31000)
        "#,
        )
        .unwrap();
        env.state().borrow_mut().item_cooldowns.insert(
            6948,
            SpellCooldownState {
                start: 17.0,
                duration: 43.0,
            },
        );
        env.exec(
            r#"
            CollectionsSpellButton_UpdateCooldown(I4ItemButton)
            local start, duration = I4ItemButton.cooldown:GetCooldownTimes()
            assert(start == 17000 and duration == 43000)
        "#,
        )
        .unwrap();
        env.state().borrow_mut().item_cooldowns.remove(&6948);
        env.exec(
            r#"
            CollectionsSpellButton_UpdateCooldown(I4ItemButton)
            assert(not I4ItemButton.cooldown:IsShown())
        "#,
        )
        .unwrap();
    }

    #[test]
    fn ghost_original_visibility_tracks_supplied_availability() {
        let env = load_original_consumer(
            "Blizzard_FrameXML",
            "Blizzard_FrameXML/GhostFrame.lua",
            "8221dce827f17000e772f4341b13f899b311123df143993e71cdfad984391bb1",
        );
        env.exec(
            r#"
            local info = debug.getinfo(SetGhostFrameShown, "S")
            assert(info.what == "Lua")
            assert(info.source:gsub("\\", "/"):match("Blizzard_FrameXML/GhostFrame.lua$"))
            assert(GhostFrame, "original XML GhostFrame required")
            GhostFrame:Show()
            SetGhostFrameShown(true)
            assert(not GhostFrame:IsShown())
        "#,
        )
        .unwrap();
        env.state().borrow_mut().player.can_port_graveyard = true;
        env.exec(
            r#"
            SetGhostFrameShown(true)
            assert(GhostFrame:IsShown())
            SetGhostFrameShown(false)
            assert(not GhostFrame:IsShown())
            SetGhostFrameShown(true)
            assert(GhostFrame:IsShown())
        "#,
        )
        .unwrap();
        env.state().borrow_mut().player.can_port_graveyard = false;
        env.exec(
            r#"
            SetGhostFrameShown(true)
            assert(not GhostFrame:IsShown())
        "#,
        )
        .unwrap();
    }
}

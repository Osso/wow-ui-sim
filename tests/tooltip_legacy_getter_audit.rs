#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::state::{AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo, BagItem};

const ESSENCE_ASSERTIONS: &str = r#"
local previous
for _, essenceID in ipairs({2, 3}) do
    local direct = C_TooltipInfo.GetAzeriteEssence(essenceID, 1)
    assert(direct and direct.id == essenceID and #direct.lines > 0,
        'getter selects supplied essence')
    GameTooltip:SetAzeriteEssence(essenceID, 1)
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == essenceID and #shown.lines > 0,
        'setter publishes supplied essence')
    assert(not previous or previous ~= shown.id, 'essence entries must differ')
    previous = shown.id
end
"#;

const ESSENCE_SLOT_ASSERTIONS: &str = r#"
local previous
for _, case in ipairs({{0, 2}, {1, 3}}) do
    local direct = C_TooltipInfo.GetAzeriteEssenceSlot(case[1])
    assert(direct and direct.id == case[2] and #direct.lines > 0,
        'getter selects supplied essence slot')
    GameTooltip:SetAzeriteEssenceSlot(case[1])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == case[2] and #shown.lines > 0,
        'setter publishes supplied essence slot')
    assert(not previous or previous ~= shown.id, 'slot entries must differ')
    previous = shown.id
end
"#;

const CONDUIT_ASSERTIONS: &str = r#"
for _, case in ipairs({{5, 1}, {7, 2}}) do
    local direct = C_TooltipInfo.GetConduit(case[1], case[2])
    assert(direct and #direct.lines > 0, 'getter publishes selected conduit')
    GameTooltip:SetConduit(case[1], case[2])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and #shown.lines > 0, 'setter publishes selected conduit')
    assert(shown.lines[1].leftText == direct.lines[1].leftText,
        'setter publishes the selected conduit')
end
local first = C_TooltipInfo.GetConduit(5, 1)
local second = C_TooltipInfo.GetConduit(7, 2)
assert(first.lines[1].leftText ~= second.lines[1].leftText,
    'conduit entries must differ')
"#;

const ENHANCED_CONDUIT_ASSERTIONS: &str = r#"
for _, case in ipairs({{5, 1}, {7, 2}}) do
    local direct = C_TooltipInfo.GetEnhancedConduit(case[1], case[2])
    assert(direct and #direct.lines > 0, 'getter publishes selected enhanced conduit')
    GameTooltip:SetEnhancedConduit(case[1], case[2])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and #shown.lines > 0, 'setter publishes selected enhanced conduit')
    assert(shown.lines[1].leftText == direct.lines[1].leftText,
        'setter publishes the selected enhanced conduit')
end
local first = C_TooltipInfo.GetEnhancedConduit(5, 1)
local second = C_TooltipInfo.GetEnhancedConduit(7, 2)
assert(first.lines[1].leftText ~= second.lines[1].leftText,
    'enhanced conduit entries must differ')
"#;

const RUNEFORGE_ASSERTIONS: &str = r#"
for _, case in ipairs({
    {'Item-0-1-6948', 6948, 52},
    {'Item-0-2-1728', 1728, 61},
}) do
    local direct = C_TooltipInfo.GetRuneforgeResultItem(case[1], case[3])
    assert(direct and direct.id == case[2] and direct.overrideItemLevel == case[3],
        'getter selects supplied runeforge item')
    GameTooltip:SetRuneforgeResultItem(case[1], case[3])
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == case[2] and shown.overrideItemLevel == case[3],
        'setter publishes supplied runeforge item')
end
"#;

const COMPANION_ASSERTIONS: &str = r#"
local previous
for _, petID in ipairs({'AuditPet-1', 'AuditPet-2'}) do
    local direct = C_TooltipInfo.GetCompanionPet(petID)
    assert(direct and direct.id == petID and direct.lines[1].leftText ~= '',
        'getter selects supplied companion pet')
    GameTooltip:SetCompanionPet(petID)
    local shown = GameTooltip:GetPrimaryTooltipData()
    assert(shown and shown.id == petID and shown.lines[1].leftText == direct.lines[1].leftText,
        'setter publishes supplied companion pet')
    assert(not previous or previous ~= shown.id, 'pet entries must differ')
    previous = shown.id
end
"#;

fn item_env() -> wow_ui_sim::lua_api::WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec("GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')")
        .unwrap();
    env
}

fn seed_azerite(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.player.present = true;
    for (slot, essence_id) in [(0, 2), (1, 3)] {
        state.azerite_essence.essences.insert(
            essence_id,
            AzeriteEssenceInfo {
                id: essence_id,
                name: format!("Supplied essence {essence_id}"),
                rank: 1,
                icon: 1,
                unlocked: true,
                valid: true,
                access_rank: 1,
                has_never_activated: false,
            },
        );
        state
            .azerite_essence
            .milestones
            .push(AzeriteEssenceMilestoneInfo {
                id: essence_id,
                required_level: 1,
                slot: Some(slot),
                unlocked: true,
                can_unlock: false,
                is_major_slot: slot == 0,
                swirl_scale: 1.0,
                requires_only_aura: false,
                spell_id: 0,
                rank: None,
                active_essence_id: Some(essence_id),
            });
    }
}

fn seed_runeforge_items(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    for (slot, item_id) in [(1, 6948), (2, 1728)] {
        state.bag_items.insert(
            (0, slot),
            BagItem {
                owned_identity: None,
                item_id,
                stack_count: 1,
                hyperlink: None,
                durability: None,
                applied_transmog: None,
            },
        );
    }
}

fn seed_companion_pets(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let mut first = state.world.pets.first().cloned().unwrap();
    first.pet_id = "AuditPet-1".into();
    first.name = "First supplied pet".into();
    first.species_id = 190;
    let mut second = first.clone();
    second.pet_id = "AuditPet-2".into();
    second.name = "Second supplied pet".into();
    second.species_id = 191;
    state.world.pets.clear();
    state.world.pets.extend([first, second]);
}

#[test]
fn selected_azerite_essence_getter_publishes_two_supplied_entries() {
    let env = item_env();
    seed_azerite(&env);
    env.exec(ESSENCE_ASSERTIONS).unwrap();
}

#[test]
fn selected_azerite_essence_slot_getter_publishes_two_supplied_entries() {
    let env = item_env();
    seed_azerite(&env);
    env.exec(ESSENCE_SLOT_ASSERTIONS).unwrap();
}

#[test]
fn selected_conduit_getter_publishes_two_catalog_entries() {
    let env = item_env();
    env.exec(CONDUIT_ASSERTIONS).unwrap();
}

#[test]
fn selected_enhanced_conduit_getter_publishes_two_catalog_entries() {
    let env = item_env();
    env.exec(ENHANCED_CONDUIT_ASSERTIONS).unwrap();
}

#[test]
fn selected_runeforge_getter_publishes_two_supplied_items() {
    let env = item_env();
    seed_runeforge_items(&env);
    env.exec(RUNEFORGE_ASSERTIONS).unwrap();
}

#[test]
fn selected_companion_pet_getter_publishes_two_supplied_pets() {
    let env = item_env();
    seed_companion_pets(&env);
    env.exec(COMPANION_ASSERTIONS).unwrap();
}

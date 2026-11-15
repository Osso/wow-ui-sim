//! INFERRED Forever applicability: declaration parity, Retail R88p/R88k arithmetic.
//! Execute the pinned original consumer, not a copied count implementation.
#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::discover_blizzard_startup_addons_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const ORIGINAL_SOURCE: &str = r#"
    local fn = CooldownViewerCooldownItemMixin.CacheChargeValues
    assert(type(fn) == 'function')
    local source = debug.getinfo(fn, 'S').source
    local suffix = 'Blizzard_CooldownViewer/CooldownViewer.lua'
    assert(source:sub(-#suffix) == suffix, 'unexpected consumer: '..tostring(source))
    assert(C_AddOns.IsAddOnLoaded('Blizzard_CooldownViewer'))
"#;

const OBSERVE_COUNTS: &str = r#"
    local function observe(id, expected)
        local item = {
            GetSpellChargeInfo = function() return nil end,
            GetSpellID = function() return id end,
            SetCachedChargeValues = function(self, count, shown)
                self.count, self.shown = count, shown
            end,
        }
        CooldownViewerCooldownItemMixin.CacheChargeValues(item)
        assert(item.count == expected,
            'spell '..id..' count='..tostring(item.count)..', expected='..expected)
        assert(item.shown == (expected > 0))
    end
    observe(131, 7)
    observe(467, 2)
    observe(880001, 0)
"#;

fn load_forever_consumer() -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::WowForever);
    let ui = blizzard_ui_dir();
    let toc = TocFile::from_file(&ui.join("Blizzard_CooldownViewer/Blizzard_CooldownViewer.toc"))
        .expect("pinned Forever CooldownViewer TOC");
    assert!(!toc.is_load_on_demand());
    let selected = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        selected
            .iter()
            .any(|addon| addon.name == "Blizzard_CooldownViewer")
    );
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_CooldownViewer"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_CooldownViewer"));
    env.exec(ORIGINAL_SOURCE)
        .expect("original selected Forever consumer");
    env
}

fn add_count_backing(env: &WowLuaEnv, spell: u32, item: u32, count: i32, slot: i32) {
    assert!(wow_ui_sim::spells::get_spell(spell).is_some());
    let mut sim = env.state().borrow_mut();
    sim.spell_reagent_pairs
        .insert(spell, vec![(item as i32, 1)]);
    sim.bag_items.insert(
        (0, slot),
        BagItem {
            owned_identity: None,
            item_id: item,
            stack_count: count,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
}

#[test]
fn forever_original_cooldown_viewer_observes_distinct_reagent_counts_and_absence() {
    let env = load_forever_consumer();
    assert!(wow_ui_sim::spells::get_spell(880001).is_none());
    assert_eq!(
        wow_ui_sim::spells::get_spell(131).unwrap().name,
        "Water Breathing"
    );
    assert_eq!(wow_ui_sim::spells::get_spell(467).unwrap().name, "Thorns");
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_cast_context_is_null = Some(false);
        sim.cooldowns_restricted = false;
        sim.spell_reagent_pairs.clear();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.spell_charges.clear();
    }
    add_count_backing(&env, 131, 31001, 7, 1);
    add_count_backing(&env, 467, 31002, 2, 2);
    env.exec(OBSERVE_COUNTS)
        .expect("original consumer distinguishes 7, 2 and absent 0");
}

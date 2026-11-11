//! Selected Forever Camelot constructors and original spellbook consumers.
#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::c_api::{c_input_interface_style::InputInterfaceStyle, outfit_entry::OutfitEntry};
use wow_ui_sim::lua_api::WowLuaEnv;

const AVAILABILITY_UNSET: &str =
    "C_TransmogOutfitInfo.IsTransmogEnabled: transmog availability has not been supplied";
const OUTFITS_UNSET: &str =
    "C_TransmogOutfitInfo.GetOutfitsInfo: outfit list has not been supplied";

fn entries(ids: &[i64]) -> Vec<OutfitEntry> {
    ids.iter()
        .map(|&id| OutfitEntry {
            outfit_id: id,
            name: format!("Supplied outfit {id}"),
            situation_categories: vec![format!("supplied-{id}"), "second".into()],
            icon: 135771 + id,
            is_event_outfit: id == 81,
            is_disabled: id == 23,
            player_facing_outfit_index: id + 10,
        })
        .collect()
}

fn supply(env: &WowLuaEnv, ids: Option<&[i64]>) {
    env.state().borrow_mut().forever_outfits = Some(ids.map(entries));
}

fn load_original(enabled: bool) -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    env.state().borrow_mut().transmog_availability.enabled = Some(enabled);
    supply(&env, Some(&[23, 81]));
    crate::common::panel_fixtures::load_panel_addons(&env);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_PlayerSpells"], &[]);
    env.state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Gamepad);
    eprintln!(
        "unfiltered load diagnostics: {:?}",
        env.state().borrow().lua_errors
    );
    env.exec(
        r#"
        local function original(fn, suffix, line)
            local info = debug.getinfo(fn, 'S')
            local path = info.source:gsub('\\', '/')
            assert(info.what == 'Lua' and path:sub(-#suffix) == suffix, path)
            assert(info.linedefined == line, info.linedefined)
            print('original consumer', path, info.linedefined)
        end
        original(SpellBookFrameMixin.CreateCategoryMixins,
            'Camelot/SpellBook/Blizzard_SpellBookFrame.lua', 115)
        original(SpellBookFrameMixin.UpdateAllSpellData,
            'SpellBook/Blizzard_SpellBookFrame.lua', 487)
        original(SpellBookTransmogCategoryMixin.IsAvailable,
            'SpellBook/Blizzard_SpellBookCategory.lua', 506)
        original(SpellBookTransmogCategoryMixin.PopulateSpellGroupsIndiciesByRange,
            'SpellBook/Blizzard_SpellBookCategory.lua', 463)
        Book = PlayerSpellsFrame.SpellBookFrame
        assert(Book.CreateCategoryMixins == SpellBookFrameMixin.CreateCategoryMixins)
    "#,
    )
    .expect("selected Camelot implementation and XML instance");
    env
}

fn construct(env: &WowLuaEnv) {
    env.exec(r#"
        Book:CreateCategoryMixins()
        Category = Book.categoryMixins[#Book.categoryMixins]
        assert(Category.IsAvailable == SpellBookTransmogCategoryMixin.IsAvailable)
        assert(Category.PopulateSpellGroupsIndiciesByRange == SpellBookTransmogCategoryMixin.PopulateSpellGroupsIndiciesByRange)
        assert(Category.spellBookFrame == Book)
    "#).expect("original constructor creates transmog category");
}

fn observe(env: &WowLuaEnv, expected: bool) {
    assert_eq!(
        env.eval::<bool>("return Category:IsAvailable()").unwrap(),
        expected
    );
    env.exec(
        "Book:UpdateAllSpellData(false); Category = Book.categoryMixins[#Book.categoryMixins]",
    )
    .unwrap();
    let shown: bool = env
        .eval("return Book.CategoryTabSystem:IsTabShown(Category:GetTabID())")
        .unwrap();
    assert_eq!(
        shown, expected,
        "original refresh projects availability to tab visibility"
    );
}

fn exercise_availability(initial: bool) {
    let first = load_original(initial);
    // Independent original query reaches line 507 even when construction would
    // first encounter the separate outfit producer during Init.
    assert_eq!(
        first
            .eval::<bool>("return SpellBookTransmogCategoryMixin:IsAvailable()")
            .unwrap(),
        initial
    );
    construct(&first);
    let second = load_original(!initial);
    construct(&second);
    for enabled in [initial, !initial, initial] {
        first.state().borrow_mut().transmog_availability.enabled = Some(enabled);
        observe(&first, enabled);
        observe(&second, !initial);
    }
    first.state().borrow_mut().transmog_availability.enabled = None;
    let error = first.exec("Category:IsAvailable()").unwrap_err();
    assert!(error.to_string().contains(AVAILABILITY_UNSET), "{error}");
    first.state().borrow_mut().transmog_availability.enabled = Some(true);
    for ids in [Some(&[][..]), None, Some(&[47][..])] {
        supply(&first, ids);
        observe(&first, ids.is_some_and(|ids| !ids.is_empty()));
    }
    first
        .state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Mkb);
    observe(&first, false);
    first
        .state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Gamepad);
    observe(&first, true);
    first.state().borrow_mut().forever_outfits = None;
    let error = first.exec("Category:IsAvailable()").unwrap_err();
    assert!(error.to_string().contains(OUTFITS_UNSET), "{error}");
    supply(&first, Some(&[23]));
    observe(&first, true);
    observe(&second, !initial);
    first
        .exec("Book:UpdateAllSpellData(false)")
        .expect("original refresh reaches category availability");
}

#[test]
fn availability_enabled_first_transitions_and_isolation() {
    exercise_availability(true);
}

#[test]
fn availability_disabled_first_transitions_and_prerequisites() {
    exercise_availability(false);
}

fn assert_groups(env: &WowLuaEnv, expected: &[i64]) {
    env.exec("Category:PopulateSpellGroupsIndiciesByRange()")
        .unwrap();
    let indices: String = env
        .eval("return table.concat(Category.spellGroups[1].orderedSpellBookItemSlotIndices, ',')")
        .unwrap();
    let expected = std::iter::once(100000)
        .chain(expected.iter().map(|id| 100000 + id))
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(indices, expected);
    env.exec(
        r#"
        local group = Category.spellGroups[1]
        local count = 0
        for key, value in pairs(group.spellBookItemSlotIndices) do
            assert(value == true)
            count = count + 1
            assert(tContains(group.orderedSpellBookItemSlotIndices, key))
        end
        assert(count == #group.orderedSpellBookItemSlotIndices)
    "#,
    )
    .unwrap();
}

#[test]
fn outfit_projection_original_sparse_groups_and_isolation() {
    let first = load_original(true);
    construct(&first);
    assert_groups(&first, &[23, 81]);
    let second = load_original(false);
    construct(&second);
    first
        .exec(
            r#"
        local a = C_TransmogOutfitInfo.GetOutfitsInfo()
        assert(select('#', C_TransmogOutfitInfo.GetOutfitsInfo()) == 1)
        assert(#a == 2 and a[1].outfitID == 23 and a[2].outfitID == 81)
        assert(a[1].name == 'Supplied outfit 23' and a[1].icon == 135794)
        assert(a[1].isDisabled and not a[1].isEventOutfit)
        assert(a[2].isEventOutfit and not a[2].isDisabled)
        assert(a[1].playerFacingOutfitIndex == 33)
        assert(a[1].situationCategories[1] == 'supplied-23')
        assert(a[1].situationCategories[2] == 'second')
        a[1].outfitID = 999
        a[1].situationCategories[1] = 'changed'
        a[2] = nil
        local b = C_TransmogOutfitInfo.GetOutfitsInfo()
        assert(#b == 2 and b[1].outfitID == 23)
        assert(b[1].situationCategories[1] == 'supplied-23')
    "#,
        )
        .unwrap();
    supply(&first, Some(&[47]));
    assert_groups(&first, &[47]);
    assert_groups(&second, &[23, 81]);
    supply(&first, Some(&[]));
    assert_groups(&first, &[]);
    first
        .exec("assert(select('#', C_TransmogOutfitInfo.GetOutfitsInfo()) == 1)")
        .unwrap();
    supply(&first, None);
    assert_groups(&first, &[]);
    first
        .exec("assert(select('#', C_TransmogOutfitInfo.GetOutfitsInfo()) == 0)")
        .unwrap();
    first.state().borrow_mut().forever_outfits = None;
    let error = first
        .exec("Category:PopulateSpellGroupsIndiciesByRange()")
        .unwrap_err();
    assert!(error.to_string().contains(OUTFITS_UNSET), "{error}");
    supply(&first, Some(&[81, 23]));
    assert_groups(&first, &[81, 23]);
}

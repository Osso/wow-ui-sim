#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn encounter_journal_toc() -> PathBuf {
    let addon = blizzard_ui_dir().join("Blizzard_EncounterJournal");
    if cfg!(feature = "client-wowforever") {
        addon.join("Blizzard_EncounterJournal.toc")
    } else {
        addon.join("Blizzard_EncounterJournal_Mainline.toc")
    }
}

fn load_loot_journal(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    env.state().borrow_mut().addon_base_paths = vec![ui];
    wow_ui_sim::xml::register_intrinsic_templates();
    load_addon(&env.loader_env(), &encounter_journal_toc())
        .expect("selected Encounter Journal TOC should load");
}

#[test]
fn encounter_journal_renders_supplied_item_sets_and_items() {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    load_loot_journal(&env);

    let rendered: String = env
        .eval(
            r#"
            A_Admin.SetLootJournalItemSets({
                { setID=501, name="Supplied Alpha", itemLevel=620, classID=2, specID=71,
                  items={{ itemID=41001, icon=91001, slot=1, invType=13 }} },
                { setID=502, name="Supplied Beta", itemLevel=630, classID=2, specID=71,
                  items={{ itemID=41002, icon=91002, slot=2, invType=14 }} },
                { setID=503, name="Filtered Out", itemLevel=640, classID=3, specID=73,
                  items={{ itemID=41003, icon=91003, slot=3, invType=15 }} },
            })
            local journal = EncounterJournal.LootJournalItems
            journal:SetClassAndSpecFilters(2, 71)
            local list = journal.ItemSetsFrame
            list.dirty = true
            list:UpdateList()

            local provider = list.ScrollBox:GetDataProvider()
            assert(provider:GetSize() == 2, "the loot-journal consumer must filter to two sets")
            local rendered = {}
            list.ScrollBox:ForEachFrame(function(button)
                local item = button.ItemButtons[1]
                rendered[#rendered + 1] = table.concat({
                    button.SetName:GetText(), item.itemID, item.Icon:GetTexture()
                }, ":")
            end)
            table.sort(rendered)
            return table.concat(rendered, "|")
            "#,
        )
        .expect("Encounter Journal loot item-set consumer should render supplied values");

    assert_eq!(
        rendered,
        "Supplied Alpha:41001:91001|Supplied Beta:41002:91002"
    );
}

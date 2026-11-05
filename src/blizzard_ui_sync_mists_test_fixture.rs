fn write_mists_required_cache_entries(root: &std::path::Path) {
    for entry in super::required_profile_cache_entries() {
        let path = root.join(entry);
        std::fs::create_dir_all(path.parent().expect("entry has parent"))
            .expect("create entry parent");
        std::fs::write(path, mists_required_cache_entry_contents(entry))
            .expect("write required cache entry");
    }
}

fn mists_required_cache_entry_contents(entry: &str) -> &'static str {
    mists_action_bar_entry_contents(entry)
        .or_else(|| mists_core_entry_contents(entry))
        .unwrap_or("placeholder")
}

fn mists_action_bar_entry_contents(entry: &str) -> Option<&'static str> {
    let contents = match entry {
        "Blizzard_ActionBar/Classic/ActionButtonTemplate.xml" => {
            r#"<CheckButton name="ActionBarButtonTemplate"><Cooldown parentKey="chargeCooldown"/></CheckButton>"#
        }
        "Blizzard_ActionBar/Classic/MainMenuBar.lua" => {
            "function MainMenuBar_SetMaxLevelBarShown(shown)\nend\n"
        }
        "Blizzard_ActionBar/Classic/MainMenuBar.xml" => {
            r#"<Ui><Frame name="MainMenuBarMaxLevelBar"/></Ui>"#
        }
        "Blizzard_ActionBar/Classic/PossessActionBar.xml" => {
            r#"<Frame name="PossessActionBar" mixin="PossessActionBarMixin"/>"#
        }
        _ => return None,
    };

    Some(contents)
}

fn mists_core_entry_contents(entry: &str) -> Option<&'static str> {
    let contents = match entry {
        "Blizzard_ChatFrame/Classic/ChatConfigFrame.xml" => {
            r#"<Ui><CheckButton name="ChatConfigCheckButtonTemplate"><FontString name="$parentText"/></CheckButton></Ui>"#
        }
        "Blizzard_MicroMenu/Blizzard_MicroMenu_Classic.toc" => {
            "Vanilla\\MicroMenuContainerOverrides.lua [AllowLoadGameType vanilla, tbc]\n\
             [Game]\\MicroMenuContainerOverrides.lua [AllowLoadGameType wrath, cata, mists]\n"
        }
        "Blizzard_MicroMenu/Shared/MicroMenuContainer.lua" => {
            "function MicroMenuMixin:AddButton(button)\n\tbutton:PostAddButtonCallback();\nend\n"
        }
        "Blizzard_Fonts_Shared/Classic/GameFonts.xml" => {
            r#"<Ui><FontFamily name="PriceFont"/></Ui>"#
        }
        "Blizzard_MoneyFrame/Blizzard_MoneyFrame_Classic.toc" => {
            "## AllowLoadGameType: classic\nClassic\\MoneyInputFrame.lua\n"
        }
        "Blizzard_MoneyFrame/Classic/MoneyInputFrame.xml" => {
            r#"<Ui><Frame name="MoneyInputFrameTemplate"><EditBox parentKey="copper"/></Frame></Ui>"#
        }
        "Blizzard_NamePlates/Blizzard_NamePlates.toc" => {
            "Mainline\\Blizzard_ClassNameplateBar.lua [AllowLoadGameType mainline]\n"
        }
        "Blizzard_SharedMapDataProviders/Blizzard_SharedMapDataProviders_Mists.toc" => {
            "## AllowLoadGameType: mists\nWrath\\BonusObjectiveDataProvider.lua\n"
        }
        "Blizzard_UIPanels_Game/Shared/CastingBarFrame.lua" => {
            "function PlayerCastingBarMixin:OnShow()\nend\n"
        }
        "Blizzard_WorldMap/Blizzard_WorldMap_Mists.toc" => {
            "## AllowLoadGameType: mists\nCata\\Blizzard_WorldMap.xml\n"
        }
        "Blizzard_WorldMap/Cata/Blizzard_WorldMap.xml" => {
            r#"<Ui><Button name="WorldMapTrackQuest"/><Frame parentKey="WorldMapOptionsDropDown"/></Ui>"#
        }
        _ => return None,
    };

    Some(contents)
}

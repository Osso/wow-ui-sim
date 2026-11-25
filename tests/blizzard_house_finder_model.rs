#![cfg(feature = "client-retail")]

use std::path::PathBuf;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn house_finder_toc() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
        .join("Blizzard_HousingHouseFinder/Blizzard_HousingHouseFinder.toc")
}

const SUPPLY_NEIGHBORHOODS: &str = r#"
A_Admin.SetHouseFinderNeighborhoods({
    { neighborhoodGUID="Neighborhood-A", neighborhoodName="Amber Vale", neighborhoodType=Enum.NeighborhoodType.Public,
      neighborhoodOwnerType=Enum.NeighborhoodOwnerType.None, suggestionReason=Enum.HouseFinderSuggestionReason.None,
      uiMapID=41, textureSuffix="amber", factionMatches=true,
      plots={{ mapPosition={x=0.2,y=0.3}, ownerType=Enum.HousingPlotOwnerType.None, plotID=17, plotCost=1250 }} },
    { neighborhoodGUID="Neighborhood-B", neighborhoodName="Cobalt Reach", neighborhoodType=Enum.NeighborhoodType.Public,
      neighborhoodOwnerType=Enum.NeighborhoodOwnerType.None, suggestionReason=Enum.HouseFinderSuggestionReason.None,
      uiMapID=82, textureSuffix="cobalt", factionMatches=false,
      plots={{ mapPosition={x=0.7,y=0.8}, ownerType=Enum.HousingPlotOwnerType.Friend, plotID=29, ownerName="Mira" }} },
})
"#;

const SELECT_SECOND_NEIGHBORHOOD: &str = r#"
local buttons = HouseFinderFrame.neighborhoodButtonPool:EnumerateActive()
local first = buttons()
local second = buttons()
if not first or not second then return "neighborhoods_not_rendered" end
local rendered = first.NeighborhoodName:GetText() .. "/" .. second.NeighborhoodName:GetText()
second:Click()
return rendered .. ";" .. tostring(HouseFinderFrame.mapID)
"#;

#[test]
fn supplied_house_finder_neighborhoods_and_plots_reach_original_consumer() {
    let env = WowLuaEnv::new().expect("environment initializes");
    env.exec(SUPPLY_NEIGHBORHOODS).expect("supply distinct neighborhoods");
    load_addon(&env.loader_env(), &house_finder_toc()).expect("original House Finder addon loads");
    env.exec("ShowUIPanel(HouseFinderFrame); C_Housing.HouseFinderRequestNeighborhoods()")
        .expect("open the original frame and request its list");
    env.process_timers().expect("neighborhood reply dispatches asynchronously");
    let selected: String = env.eval(SELECT_SECOND_NEIGHBORHOOD)
        .expect("original frame selects supplied neighborhood");
    assert_eq!(selected, "Amber Vale/Cobalt Reach;82");
    env.process_timers().expect("plot reply dispatches asynchronously");
    let plots: i64 = env.eval("return #HouseFinderFrame.mapDataProvider.houseMapData")
        .expect("original map provider stores selected plots");
    assert_eq!(plots, 1);
}

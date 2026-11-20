#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::trainer_service::{TrainerService, TrainerServiceType};
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

const ORIGINAL_TRAINER_CONSUMER: &str = r#"
local alphaName, alphaType, _, _, alphaRank = GetTrainerServiceInfo(1)
local betaName, betaType, _, _, betaRank = GetTrainerServiceInfo(2)
assert(alphaName == 'Supplied Trainer Alpha' and alphaType == 'available' and alphaRank == 'Rank I',
    'legacy trainer info reads the supplied Alpha service')
assert(betaName == 'Supplied Trainer Beta' and betaType == 'available' and betaRank == 'Rank II',
    'legacy trainer info reads the supplied Beta service')
SelectTrainerService(2)
assert(GetTrainerSelectionIndex() == 2, 'selection reports the selected service')
SelectTrainerService(GetNumTrainerServices() + 1)
assert(GetTrainerSelectionIndex() == 0, 'selection clears when the UI passes an out-of-range index')

ClassTrainerFrame:Show()
ClassTrainerFrame_Update(true)

local view = ClassTrainerFrame.ScrollBox:GetView()
local rows = view.frames
local found = {}
for index = 1, view:GetFrameCount() do
    local row = rows[index]
    local name = row and row.name and row.name:GetText()
    if name == 'Supplied Trainer Alpha' or name == 'Supplied Trainer Beta' then
        found[name] = row
    end
end
assert(found['Supplied Trainer Alpha'], 'original ClassTrainerFrame renders service Alpha')
assert(found['Supplied Trainer Beta'], 'original ClassTrainerFrame renders service Beta')

local alpha = found['Supplied Trainer Alpha']
local beta = found['Supplied Trainer Beta']
assert(alpha.money:IsShown() and alpha.money.CopperButton.Text:GetText() == '34',
    'original trainer row renders Alpha copper cost')
assert(alpha.money.SilverButton.Text:GetText() == '12', 'original trainer row renders Alpha silver cost')
assert(beta.money:IsShown() and beta.money.CopperButton.Text:GetText() == '78',
    'original trainer row renders Beta copper cost')
assert(beta.money.SilverButton.Text:GetText() == '56', 'original trainer row renders Beta silver cost')

local previousTooltipID
for _, row in ipairs({alpha, beta}) do
    row:GetScript('OnEnter')(row)
    local tooltip = GameTooltip:GetPrimaryTooltipData()
    assert(tooltip, 'original row hover publishes its service spell tooltip')
    assert(#tooltip.lines > 0, 'trainer spell tooltip has lines')
    assert(not previousTooltipID or previousTooltipID ~= tooltip.id, 'service tooltips must differ')
    assert((row == alpha and tooltip.id == 19750) or (row == beta and tooltip.id == 642),
        'each original trainer row selects its supplied spell')
    previousTooltipID = tooltip.id
end
"#;

fn load_trainer_env() -> WowLuaEnv {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.state().borrow_mut().trainer.services = supplied_services();
    load_original_trainer_ui(&env);
    env
}

fn load_original_trainer_ui(env: &WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let addon = ui.join("Blizzard_TrainerUI");
    let toc = find_toc_file(&addon).expect("pinned TrainerUI TOC should resolve");
    load_addon(&env.loader_env(), &toc).expect("original TrainerUI should load");
}

fn supplied_services() -> Vec<TrainerService> {
    vec![
        trainer_service(19750, "Supplied Trainer Alpha", "Rank I", 1234),
        trainer_service(642, "Supplied Trainer Beta", "Rank II", 5678),
    ]
}

fn trainer_service(spell_id: u32, name: &str, sub_text: &str, cost: u64) -> TrainerService {
    TrainerService {
        spell_id,
        name: name.to_string(),
        sub_text: sub_text.to_string(),
        service_type: TrainerServiceType::Available,
        icon: 134400,
        cost,
        required_level: None,
        required_skill: None,
        ability_requirements: Vec::new(),
        is_profession: false,
    }
}

#[test]
fn original_class_trainer_rows_show_supplied_services_costs_and_tooltips() {
    let env = load_trainer_env();
    env.exec(ORIGINAL_TRAINER_CONSUMER)
        .expect("original ClassTrainerFrame should display both supplied trainer services");
}

//! API-level selection for the `C_ProfSpecs` reminders (WM49 part A).
//! Blacksmithing (164) and Mining (186) are the simulator's stored
//! professions; related lines and tabs come from 70391 data, with WM55
//! answers to Q2–Q4.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::c_api::profession_spec_inputs::{ProfessionSpecInputs, SkillRank};
use wow_ui_sim::lua_api::WowLuaEnv;

const BLACKSMITHING: u32 = 164;
const DRAGON_ISLES_BLACKSMITHING: u32 = 2822;
const KHAZ_ALGAR_BLACKSMITHING: u32 = 2872;
const KHAZ_ALGAR_MINING: u32 = 2881;
const MINING: u32 = 186;
/// SkillLineXTraitTree 70391: tab 302 belongs to 2822, tab 983 to 2881.
const DRAGON_ISLES_ARMORSMITHING_TAB: u32 = 302;
const KHAZ_ALGAR_MINING_TAB: u32 = 983;
/// First point currency of each line's tabs (TraitTreeXTraitCurrency 70391).
const DRAGON_ISLES_BLACKSMITHING_KNOWLEDGE: i32 = 2023;
const KHAZ_ALGAR_BLACKSMITHING_KNOWLEDGE: i32 = 2786;
const MIDNIGHT_BLACKSMITHING_KNOWLEDGE: i32 = 3151;
const MIDNIGHT_MINING_KNOWLEDGE: i32 = 3158;

fn learned(current: i32) -> SkillRank {
    SkillRank { current, bonus: 0 }
}

fn env_with(inputs: ProfessionSpecInputs) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create profession environment");
    env.state().borrow_mut().profession_spec_inputs = inputs;
    env
}

fn set_quantity(env: &WowLuaEnv, currency: i32, quantity: i32) {
    let mut state = env.state().borrow_mut();
    state
        .currency_info
        .get_mut(&currency)
        .expect("CurrencyTypes row is seeded")
        .quantity = quantity;
}

fn reminder_name(env: &WowLuaEnv) -> (i32, Option<String>) {
    env.eval(
        "return select('#', C_ProfSpecs.GetNewSpecReminderProfName()), \
         C_ProfSpecs.GetNewSpecReminderProfName()",
    )
    .expect("GetNewSpecReminderProfName is callable")
}

fn points_reminder(env: &WowLuaEnv) -> (i32, bool) {
    env.eval(
        "return select('#', C_ProfSpecs.ShouldShowPointsReminder()), \
         C_ProfSpecs.ShouldShowPointsReminder()",
    )
    .expect("ShouldShowPointsReminder is callable")
}

/// Dragon Isles Blacksmithing learned and enabled, with tab 302 in `tab_state`.
fn blacksmithing_tab(tab_state: i32) -> ProfessionSpecInputs {
    let mut inputs = ProfessionSpecInputs::default();
    inputs
        .related_skill_ranks
        .insert(DRAGON_ISLES_BLACKSMITHING, learned(25));
    inputs
        .enabled_skill_lines
        .insert(DRAGON_ISLES_BLACKSMITHING);
    inputs
        .tab_states
        .insert(DRAGON_ISLES_ARMORSMITHING_TAB, tab_state);
    inputs
}

fn with_mining_tab(mut inputs: ProfessionSpecInputs) -> ProfessionSpecInputs {
    inputs
        .related_skill_ranks
        .insert(KHAZ_ALGAR_MINING, learned(40));
    inputs.enabled_skill_lines.insert(KHAZ_ALGAR_MINING);
    inputs.tab_states.insert(KHAZ_ALGAR_MINING_TAB, 2);
    inputs
}

#[test]
fn name_reminder_selects_the_outer_profession_of_an_unlockable_related_tab() {
    assert_eq!(
        reminder_name(&env_with(ProfessionSpecInputs::default())),
        (1, None)
    );
    let env = env_with(blacksmithing_tab(2));
    assert_eq!(reminder_name(&env), (1, Some("Blacksmithing".into())));

    // Stored order decides; Mining wins once Blacksmithing is unlearned.
    let both = env_with(with_mining_tab(blacksmithing_tab(2)));
    assert_eq!(reminder_name(&both), (1, Some("Blacksmithing".into())));
    both.state()
        .borrow_mut()
        .crafting
        .unlearned_profession_ids
        .insert(BLACKSMITHING as i32);
    assert_eq!(reminder_name(&both), (1, Some("Mining".into())));
}

#[test]
fn name_reminder_requires_state_2_the_enabled_flag_and_a_learned_related_line() {
    // State 1 (Unlocked) does not qualify.
    assert_eq!(reminder_name(&env_with(blacksmithing_tab(1))), (1, None));

    let mut disabled = blacksmithing_tab(2);
    disabled.enabled_skill_lines.clear();
    assert_eq!(reminder_name(&env_with(disabled)), (1, None));

    let mut unlearned = blacksmithing_tab(2);
    unlearned.related_skill_ranks.clear();
    assert_eq!(reminder_name(&env_with(unlearned)), (1, None));

    // Current-plus-bonus rank must also be nonzero.
    let mut cancelled = blacksmithing_tab(2);
    cancelled.related_skill_ranks.insert(
        DRAGON_ISLES_BLACKSMITHING,
        SkillRank {
            current: 25,
            bonus: -25,
        },
    );
    assert_eq!(reminder_name(&env_with(cancelled)), (1, None));
}

/// Blacksmithing enabled and eligible; no related line needs to be learned.
fn blacksmithing_points() -> ProfessionSpecInputs {
    let mut inputs = ProfessionSpecInputs::default();
    inputs.enabled_skill_lines.insert(BLACKSMITHING);
    inputs.points_eligible_skill_lines.insert(BLACKSMITHING);
    inputs
}

#[test]
fn points_reminder_needs_five_of_the_first_related_point_currency() {
    assert_eq!(
        points_reminder(&env_with(ProfessionSpecInputs::default())),
        (1, false)
    );

    // WM55-Q4: Midnight (tier 15) precedes Khaz Algar (14) and Dragon Isles (13).
    let env = env_with(blacksmithing_points());
    set_quantity(&env, KHAZ_ALGAR_BLACKSMITHING_KNOWLEDGE, 9);
    set_quantity(&env, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 5);
    assert_eq!(points_reminder(&env), (1, true));
    set_quantity(&env, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 4);
    assert_eq!(points_reminder(&env), (1, false));

    // Only the outer candidate needs the cached enabled flag.
    let mut disabled = blacksmithing_points();
    disabled.enabled_skill_lines.clear();
    let disabled = env_with(disabled);
    set_quantity(&disabled, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 9);
    assert_eq!(points_reminder(&disabled), (1, false));
}

#[test]
fn eligibility_of_an_unlearned_related_line_lets_another_line_supply_currency() {
    // WM55-Q2/Q3: no learned gate; an ineligible related line supplies currency.
    let mut inputs = blacksmithing_points();
    inputs.points_eligible_skill_lines.clear();
    inputs
        .points_eligible_skill_lines
        .insert(DRAGON_ISLES_BLACKSMITHING);
    let env = env_with(inputs);
    set_quantity(&env, DRAGON_ISLES_BLACKSMITHING_KNOWLEDGE, 0);
    set_quantity(&env, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 6);
    assert_eq!(points_reminder(&env), (1, true));
}

fn with_mining_points(mut inputs: ProfessionSpecInputs) -> ProfessionSpecInputs {
    inputs.enabled_skill_lines.insert(MINING);
    inputs.points_eligible_skill_lines.insert(MINING);
    inputs
}

#[test]
fn a_low_quantity_ends_only_its_own_candidate() {
    // WM55-Q2: Blacksmithing is eligible with 2, so Mining is still tested.
    let env = env_with(with_mining_points(blacksmithing_points()));
    set_quantity(&env, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 2);
    set_quantity(&env, MIDNIGHT_MINING_KNOWLEDGE, 9);
    assert_eq!(points_reminder(&env), (1, true));
    set_quantity(&env, MIDNIGHT_MINING_KNOWLEDGE, 3);
    assert_eq!(points_reminder(&env), (1, false));
}

#[test]
fn an_ineligible_candidate_reads_no_currency() {
    let mut inputs = with_mining_points(blacksmithing_points());
    inputs.points_eligible_skill_lines.remove(&BLACKSMITHING);
    let env = env_with(inputs);
    set_quantity(&env, MIDNIGHT_BLACKSMITHING_KNOWLEDGE, 9);
    set_quantity(&env, MIDNIGHT_MINING_KNOWLEDGE, 1);
    assert_eq!(points_reminder(&env), (1, false));
}

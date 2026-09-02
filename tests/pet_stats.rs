//! Integration tests for `src/lua_api/globals/real/pet_stats.rs`.

use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn source_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── GetPetExperience ──────────────────────────────────────────────────────────

#[test]
fn get_pet_experience_defaults_zero() {
    let env = env();
    let (xp, xp_max): (i32, i32) = env.eval("return GetPetExperience()").unwrap();
    assert_eq!(xp, 0);
    assert_eq!(xp_max, 0);
}

#[test]
fn get_pet_experience_reads_pet_state() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.pet.xp = 12_000;
        st.pet.xp_max = 50_000;
    }
    let (xp, xp_max): (i32, i32) = env.eval("return GetPetExperience()").unwrap();
    assert_eq!(xp, 12_000);
    assert_eq!(xp_max, 50_000);
}

// ── GetPetHappiness ───────────────────────────────────────────────────────────

#[test]
fn get_pet_happiness_defaults_zero() {
    let env = env();
    let (happiness, damage_pct, loyalty_rate): (i32, i32, i32) =
        env.eval("return GetPetHappiness()").unwrap();
    assert_eq!(happiness, 0);
    assert_eq!(damage_pct, 0);
    assert_eq!(loyalty_rate, 0);
}

#[test]
fn get_pet_happiness_reads_pet_state() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.pet.happiness = 3; // Happy
        st.pet.damage_percent = 125;
        st.pet.loyalty_rate = 5;
    }
    let (happiness, damage_pct, loyalty_rate): (i32, i32, i32) =
        env.eval("return GetPetHappiness()").unwrap();
    assert_eq!(happiness, 3);
    assert_eq!(damage_pct, 125);
    assert_eq!(loyalty_rate, 5);
}

// ── GetPetLoyalty ─────────────────────────────────────────────────────────────

#[test]
fn get_pet_loyalty_nil_when_label_empty() {
    let env = env();
    let v: Option<String> = env.eval("return GetPetLoyalty()").unwrap();
    assert_eq!(v, None);
}

#[test]
fn get_pet_loyalty_reads_pet_state() {
    let env = env();
    env.state().borrow_mut().pet.loyalty_label = "Devoted".into();
    let label: String = env.eval("return GetPetLoyalty()").unwrap();
    assert_eq!(label, "Devoted");
}

// ── GetPetTimeInCombat ────────────────────────────────────────────────────────

#[test]
fn get_pet_time_in_combat_defaults_zero() {
    let env = env();
    let seconds: i32 = env.eval("return GetPetTimeInCombat()").unwrap();
    assert_eq!(seconds, 0);
}

#[test]
fn get_pet_time_in_combat_reads_pet_state() {
    let env = env();
    env.state().borrow_mut().pet.time_in_combat = 45;
    let seconds: i32 = env.eval("return GetPetTimeInCombat()").unwrap();
    assert_eq!(seconds, 45);
}

#[test]
fn pet_stats_globals_live_under_real_globals_boundary() {
    assert!(
        !source_path("src/lua_api/globals/pet_stats.rs").exists(),
        "pet-stat globals are modeled through SimState and belong under globals::real",
    );
    assert!(
        source_path("src/lua_api/globals/real/pet_stats.rs").exists(),
        "pet-stat globals should stay classified as real modeled Lua globals",
    );
}

// ── GetPetSpellBonusDamage ───────────────────────────────────────────────────

#[test]
fn get_pet_spell_bonus_damage_defaults_zero() {
    let env = env();
    let bonus: i32 = env.eval("return GetPetSpellBonusDamage()").unwrap();
    assert_eq!(bonus, 0);
}

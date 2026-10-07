//! Integration tests for `src/lua_api/globals/close_frames.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

fn fired(env: &WowLuaEnv, name: &str) -> bool {
    env.state()
        .borrow()
        .events
        .pending()
        .iter()
        .any(|e| e.name == name)
}

#[test]
fn close_bank_frame_clears_flag_and_fires_bankframe_closed() {
    let env = env();
    env.state().borrow_mut().bank_frame_open = true;
    env.exec("CloseBankFrame()").unwrap();
    assert!(!env.state().borrow().bank_frame_open);
    assert!(fired(&env, "BANKFRAME_CLOSED"));
}

#[test]
fn close_guild_bank_frame_fires_guild_bank_closed() {
    let env = env();
    env.state().borrow_mut().guild_bank_frame_open = true;
    env.exec("CloseGuildBankFrame()").unwrap();
    assert!(!env.state().borrow().guild_bank_frame_open);
    assert!(fired(&env, "GUILDBANKFRAME_CLOSED"));
}

#[test]
fn close_merchant_fires_merchant_closed() {
    let env = env();
    env.state().borrow_mut().merchant_frame_open = true;
    env.exec("CloseMerchant()").unwrap();
    assert!(!env.state().borrow().merchant_frame_open);
    assert!(fired(&env, "MERCHANT_CLOSED"));
}

#[test]
fn close_tabard_creation_fires_tabard_canceled() {
    let env = env();
    env.state().borrow_mut().tabard_frame_open = true;
    env.exec("CloseTabardCreation()").unwrap();
    assert!(!env.state().borrow().tabard_frame_open);
    assert!(fired(&env, "TABARD_CANCELED"));
}

#[test]
fn close_trainer_frame_fires_trainer_closed() {
    let env = env();
    env.state().borrow_mut().trainer_frame_open = true;
    env.exec("CloseTrainerFrame()").unwrap();
    assert!(!env.state().borrow().trainer_frame_open);
    assert!(fired(&env, "TRAINER_CLOSED"));
}

#[test]
fn close_socket_info_fires_socket_info_close() {
    let env = env();
    env.state().borrow_mut().socket_frame_open = true;
    env.exec("CloseSocketInfo()").unwrap();
    assert!(!env.state().borrow().socket_frame_open);
    assert!(fired(&env, "SOCKET_INFO_CLOSE"));
}

#[test]
fn close_loot_fires_loot_closed() {
    let env = env();
    env.state().borrow_mut().loot_frame_open = true;
    env.exec("CloseLoot()").unwrap();
    assert!(!env.state().borrow().loot_frame_open);
    assert!(fired(&env, "LOOT_CLOSED"));
}

#[test]
fn close_guild_registrar_fires_petition_closed() {
    let env = env();
    env.state().borrow_mut().guild_registrar_open = true;
    env.exec("CloseGuildRegistrar()").unwrap();
    assert!(!env.state().borrow().guild_registrar_open);
    assert!(fired(&env, "PETITION_CLOSED"));
}

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn close_pet_stables_fires_pet_stable_closed() {
    let env = env();
    env.state().borrow_mut().pet_stables_open = true;
    env.exec("ClosePetStables()").unwrap();
    assert!(!env.state().borrow().pet_stables_open);
    assert!(fired(&env, "PET_STABLE_CLOSED"));
}

#[test]
fn close_already_closed_frame_still_fires_event() {
    // Idempotency: flag starts false, close is called anyway.
    let env = env();
    env.exec("CloseBankFrame()").unwrap();
    assert!(!env.state().borrow().bank_frame_open);
    assert!(fired(&env, "BANKFRAME_CLOSED"));
}

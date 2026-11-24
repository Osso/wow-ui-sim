#[cfg(target_os = "linux")]
extern crate wow_ui_sim_dylib;

#[cfg(feature = "client-wowforever")]
#[path = "wowforever_battle_net_account_consumer.rs"]
mod wowforever_battle_net_account_consumer;

wow_ui_sim_test_harness::integration_tests!();

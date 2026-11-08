#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

fn wow_sim_binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_wow-sim")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("debug")
                .join("wow-sim")
        })
}

#[test]
fn mists_trade_window_supports_money_input_flow() {
    let output = Command::new("timeout")
        .arg("90")
        .arg(wow_sim_binary())
        .args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.SetMoney(100000)
            InitiateTrade("NPC")
            FireEvent("TRADE_SHOW")
            if not (TradeFrame and TradeFrame:IsShown()) then
                error("TradeFrame did not open")
            end

            if type(TradePlayerInputMoneyFrame.copper) ~= "table" then
                error("TradePlayerInputMoneyFrame missing copper child")
            end

            MoneyInputFrame_SetCopper(TradePlayerInputMoneyFrame, 1234)
            TradeFrame_UpdateMoney()
            if GetPlayerTradeMoney() ~= 1234 then
                error("trade money did not update")
            end

            FireEvent("TRADE_CLOSED")
            if TradeFrame:IsShown() then
                error("TradeFrame did not close")
            end
            "#,
            "lua-errors",
        ])
        .output()
        .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("trade panel flow", &stdout, &stderr);
}


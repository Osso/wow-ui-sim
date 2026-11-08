#![cfg(feature = "client-mists")]

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_trade_window_supports_money_input_flow() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
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
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "wow-sim failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_no_lua_errors(&stdout, &stderr);
}

fn assert_no_lua_errors(stdout: &str, stderr: &str) {
    assert!(
        !stdout.contains("Lua error") && !stderr.contains("Lua error"),
        "trade panel flow emitted Lua errors\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

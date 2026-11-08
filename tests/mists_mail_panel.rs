#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::path::PathBuf;
use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

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
fn mists_mail_panel_supports_inbox_send_attachments_and_cod() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            r#"
            A_Admin.ClearInbox()
            A_Admin.AddMail("Auction House", "Auction won", "Body text", 12345, {
                { item_id = 6948, count = 1 },
            })
            A_Admin.ClearBags()
            A_Admin.AddBagItem(0, 1, 6948, 1)

            MailFrame_Show()
            if not (MailFrame and MailFrame:IsShown()) then
                error("MailFrame did not open")
            end
            if not (InboxFrame and InboxFrame:IsShown()) then
                error("InboxFrame did not open")
            end

            InboxFrame_Update()
            MailItem1Button:SetChecked(true)
            InboxFrame_OnClick(MailItem1Button, 1)
            if not (OpenMailFrame and OpenMailFrame:IsShown()) then
                error("OpenMailFrame did not open")
            end

            TakeInboxMoney(1)
            local _, _, _, _, money = GetInboxHeaderInfo(1)
            if money ~= 0 then
                error("TakeInboxMoney left money=" .. tostring(money))
            end

            MailFrameTab_OnClick(nil, 2)
            if not (SendMailFrame and SendMailFrame:IsShown()) then
                error("SendMailFrame did not open")
            end

            PickupContainerItem(0, 1)
            ClickSendMailItemButton(1)
            if not HasSendMailItem(1) then
                error("ClickSendMailItemButton did not attach cursor item")
            end

            local itemName, itemID, itemTexture, stackCount, quality = GetSendMailItem(1)
            if itemName ~= "Hearthstone" then
                error("send attachment name=" .. tostring(itemName))
            end
            if itemID ~= 6948 then
                error("send attachment itemID=" .. tostring(itemID))
            end
            if not itemTexture then
                error("send attachment missing texture")
            end
            if stackCount ~= 1 then
                error("send attachment stackCount=" .. tostring(stackCount))
            end
            if quality ~= 1 then
                error("send attachment quality=" .. tostring(quality))
            end

            SendMailNameEditBox:SetText("Target")
            SendMailSubjectEditBox:SetText("Subject")
            MailEditBox:SetText("Message")
            MoneyInputFrame_SetCopper(SendMailMoney, 4321)
            SendMailRadioButton_OnClick(2)
            SendMailMailButton_OnClick(SendMailMailButton)

            local _, _, _, _, _, codAmount = GetInboxHeaderInfo(2)
            if codAmount ~= 4321 then
                error("sent COD amount=" .. tostring(codAmount))
            end
            if HasSendMailItem(1) then
                error("SendMail did not clear attachment slot")
            end

            C_Mail.SetOpeningAll(true)
            C_Mail.SetOpeningAll(false)
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
    lua_error_output::assert_no_lua_errors("mail panel flow", &stdout, &stderr);
}


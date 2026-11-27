#![cfg(feature = "client-wowforever")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

#[test]
fn original_bank_interaction_consumes_supplied_tab_data() {
    with_blizzard_addon_startup_shape(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        env.exec(
            r#"
            local bankType = Enum.BankType.Character
            A_Admin.SetBankViewableTypes({ bankType })
            A_Admin.SetShouldUsePlayerBagsInBank(true)
            A_Admin.SetPurchasedBankTabData(bankType, {
                {
                    ID = Enum.BagIndex.CharacterBankTab_1,
                    name = "First host tab",
                    icon = 123,
                    depositFlags = 0,
                    tabCleanupConfirmation = "Confirm first cleanup",
                    tabNameEditBoxHeader = "Rename first tab",
                },
                {
                    ID = Enum.BagIndex.CharacterBankTab_2,
                    name = "Second host tab",
                    icon = 456,
                    depositFlags = 1,
                    tabCleanupConfirmation = "Confirm second cleanup",
                    tabNameEditBoxHeader = "Rename second tab",
                },
            })
            assert(not pcall(A_Admin.SetBankViewableTypes, { 999 }))
            assert(not pcall(A_Admin.SetPurchasedBankTabData, 999, {}))
            A_Admin.FireEvent(
                "PLAYER_INTERACTION_MANAGER_FRAME_SHOW",
                Enum.PlayerInteractionType.Banker
            )
            assert(BankFrame:IsShown(), "the original banker interaction opens BankFrame")
            local tabs = BankFrame.BankPanel.purchasedBankTabData
            assert(#tabs == 2)
            assert(tabs[1].name == "First host tab" and tabs[1].bankType == bankType)
            assert(tabs[2].name == "Second host tab" and tabs[2].bankType == bankType)
            assert(tabs[2].icon == 456 and tabs[2].depositFlags == 1)
            "#,
        )
        .expect("selected bank interaction should consume supplied C_Bank state");
    });
}

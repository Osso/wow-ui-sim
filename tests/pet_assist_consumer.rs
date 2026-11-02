//! Original pet action button reads the supplied assist reaction cache.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::{WowLuaEnv, state::PetActionSlot};

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn pet_assist_updates_original_action_button(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.pet.icon = Some("Interface/Icons/INV_Misc_QuestionMark".into());
        sim.pet_actions[0] = PetActionSlot {
            has_action: true,
            is_token: true,
            name: Some("PET_MODE_ASSIST".into()),
            ..PetActionSlot::default()
        };
    }
    env.exec(r#"
        local button = { index = 1 }
        assert(PetActionButtonMixin.GetActionButtonInfo(button).isActive == false)
        local events = 0
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('PET_BAR_UPDATE')
        listener:SetScript('OnEvent', function()
            events = events + 1
            assert(PetActionButtonMixin.GetActionButtonInfo(button).isActive == true)
        end)
        local callback
        for i = 1, 20 do
            local key, value = debug.getupvalue(SlashCommandUtil.CheckAddSecureSlashCommand, i)
            if not key then break end
            if key == 'addonTbl' then callback = value.SecureCmdList[SLASH_COMMAND.PET_ASSIST] end
        end
        assert(callback, 'runtime-selected Blizzard pet assist command missing')
        assert(select('#', callback('pet')) == 0)
        assert(events == 1)
        assert(PetActionButtonMixin.GetActionButtonInfo(button).isActive == true)
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet.control_mask = 4;
    }
    env.exec("assert(PetActionButtonMixin.GetActionButtonInfo({index=1}).isActive == false)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.pet.control_mask = 0;
        sim.pet.reaction = 1;
        sim.pet.icon = None;
    }
    env.exec("assert(select('#', C_PetInfo.PetAssistMode()) == 0)").unwrap();
    assert_eq!(env.state().borrow().pet.reaction, 1);
}
}

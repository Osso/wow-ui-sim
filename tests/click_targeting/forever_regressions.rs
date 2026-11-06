use super::click_targeting_full_ui::{
    drain_test_errors, env_with_full_ui, env_with_full_ui_configured, install_test_error_handler,
};
use crate::common;

#[test]
fn blizzard_player_world_entry_updates_combo_frame_from_target_power() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui_configured(|env| {
                env.exec("SetCVar('comboPointLocation', '1')").expect("enable player-frame combo points");
            });
            install_test_error_handler(&env);
            env.exec(r#"
                TargetUnit('enemy1')
                A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
            "#).expect("configure target combo points");
            env.fire_event("PLAYER_ENTERING_WORLD").expect("update PlayerFrame through event");
            env.exec(r#"
                assert(ComboFrame:IsShown(), 'combo frame hidden after world-entry event')
                assert(COMBO_FRAME_LAST_NUM_POINTS == 3, 'combo count not updated by world-entry event')
                A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)
            "#).expect("read and clear combo power");
            env.fire_event("PLAYER_ENTERING_WORLD").expect("update zero combo through event");
            env.exec(r#"
                assert(not ComboFrame:IsShown(), 'combo frame shown with zero points')
                assert(COMBO_FRAME_LAST_NUM_POINTS == 0, 'zero combo count not updated by event')
            "#).expect("read zero combo state");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "player world-entry event errors: {errors:?}");
        }
    });
}

// The private-partition `AuraButton.Count` case needs the mainline
// `TargetFrame.Auras` container: `Blizzard_UnitFrame/Blizzard_UnitFrame_Mainline.toc:27-28`
// lists `Shared/TargetFrameAuraContainer.lua|.xml`, and the classic UnitFrame TOC
// (`mists/AddOns/Blizzard_UnitFrame/Blizzard_UnitFrame_Classic.toc:24-27`) lists
// neither file while `Classic/TargetFrame.lua:451-500` creates `TargetFrameBuff<N>`
// buttons instead. See `classic_target_frame_fixture!` for the classic consumer path.
target_frame_aura_container_fixture! {
#[test]
fn blizzard_target_player_aura_updates_keep_count_region() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui();
            install_test_error_handler(&env);
            env.exec("TargetUnit('player')").expect("target player");
            env.fire_on_update(0.016).expect("tick target auras");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "target aura update errors: {errors:?}");
            let (has_count, public_field_stays_public): (bool, bool) = env.eval(r#"
                local function find_auras(frame, depth)
                    if depth > 15 then return nil end
                    if frame.Auras then return frame.Auras end
                    for i = 1, frame:GetNumChildren() do
                        local child = select(i, frame:GetChildren())
                        local found = child and find_auras(child, depth + 1)
                        if found then return found end
                    end
                end
                local container = assert(find_auras(TargetFrame, 0))
                local button
                for i = 1, container:GetNumChildren() do
                    local child = select(i, container:GetChildren())
                    if child and child:GetObjectType() == 'AuraButton' then
                        button = child
                        break
                    end
                end
                assert(button, 'target aura button missing')
                local count = button.Count
                button.AddonField = 'public-only'
                return count ~= nil and GetForbiddenObjectTable(button).Count == count,
                    GetForbiddenObjectTable(button).AddonField == nil
            "#).expect("inspect target aura partitions");
            assert!(has_count, "target aura private mixin cannot access Count");
            assert!(public_field_stays_public, "ordinary Lua fields leaked into private partition");
        }
    });
}
}

// Classic `TargetFrameMixin:OnEvent` reads
// `C_PlayerInteractionManager.IsReplacingUnit()` on every PLAYER_TARGET_CHANGED
// (`mists|era|anniversary/AddOns/Blizzard_UnitFrame/Classic/TargetFrame.lua:168`)
// and `UpdateAuras` (:451-500) then builds `TargetFrameBuff<N>` buttons from the
// `TargetBuffFrameTemplate` and sizes each one through ShouldAuraBeLarge(caster)
// (:502), where `caster` is the seventh `AuraUtil.ForEachAura` value (:461).
classic_target_frame_fixture! {
#[test]
fn blizzard_classic_target_player_aura_update_runs_without_errors() {
    common::with_perf_lock(|| {
        test_timeout! {
            let env = env_with_full_ui();
            install_test_error_handler(&env);
            // Supplied unit state, not a target-only fixture: two helpful player
            // auras with distinct icons. `UnitAura('target', ...)` must project the
            // state of the unit that is actually targeted.
            env.exec(r#"
                A_Admin.ClearBuffs()
                A_Admin.AddBuff(19740, 'Blessing of Might', 135906, 300, 1)
                A_Admin.AddBuff(20217, 'Blessing of Kings', 135995, 300, 1)
                TargetUnit('player')
            "#)
            .expect("seed player auras and target the player");
            env.fire_on_update(0.016).expect("tick target auras");
            let errors = drain_test_errors(&env);
            assert!(errors.is_empty(), "classic target aura update errors: {errors:?}");
            // `TargetFrameMixin:Update` (Classic/TargetFrame.lua:146) reaches
            // `UpdateAuras` (:451) after the IsReplacingUnit read, so the
            // player-targeted frame must build one `TargetFrameBuff<N>` per
            // projected helpful aura:
            //   frameName = selfName.."Buff"..buffIndex  (:463)
            //   frameIcon = _G[frameName.."Icon"]        (:476)
            //   frameIcon:SetTexture(icon)               (:477)
            let mismatch: String = env
                .eval(
                    r#"
                    if TargetFrame.unit ~= 'target' or not UnitExists('target') then
                        return 'TargetFrame lost the target unit'
                    end
                    if TargetFrame.buffs == nil then
                        return 'classic TargetFrame.xml:339 `$parentBuffs` container missing'
                    end
                    local expected = {}
                    for index = 1, 4 do
                        local name, icon = UnitAura('player', index, 'HELPFUL')
                        if not name then break end
                        expected[#expected + 1] = { name = name, icon = icon }
                    end
                    if #expected ~= 2 then
                        return 'seeded player aura count ' .. #expected
                    end
                    for index, aura in ipairs(expected) do
                        local name, icon = UnitAura('target', index, 'HELPFUL')
                        if name ~= aura.name or icon ~= aura.icon then
                            return string.format('target aura %d is %s/%s, expected %s/%s',
                                index, tostring(name), tostring(icon), aura.name, tostring(aura.icon))
                        end
                        if UnitBuff('target', index) ~= aura.name then
                            return 'UnitBuff target mismatch at ' .. index
                        end
                        local data = C_UnitAuras.GetAuraDataByIndex('target', index, 'HELPFUL')
                        if not data or data.name ~= aura.name or data.icon ~= aura.icon then
                            return 'C_UnitAuras target mismatch at ' .. index
                        end
                        local buttonName = 'TargetFrameBuff' .. index
                        local button = _G[buttonName]
                        if not button then return 'missing ' .. buttonName end
                        if button:GetObjectType() ~= 'Button' then
                            return buttonName .. ' is not a Button'
                        end
                        if not button:IsShown() then return buttonName .. ' is hidden' end
                        if button:GetID() ~= index then
                            return buttonName .. ' id ' .. tostring(button:GetID())
                        end
                        local texture = _G[buttonName .. 'Icon']:GetTexture()
                        if texture ~= aura.icon then
                            return buttonName .. ' icon ' .. tostring(texture)
                        end
                    end
                    if _G['TargetFrameBuff' .. (#expected + 1)] then
                        return 'unexpected extra target buff button'
                    end
                    return ''
                "#,
                )
                .expect("classic target frame aura probe");
            assert_eq!(mismatch, "", "classic target frame buff projection");
            // A grouped member with stored auras is the second form of "unit with
            // seeded auras": targeting `party1` must project its own buff list
            // (`default_party_auras` party slot 0: Power Word: Fortitude, 135987).
            env.state().borrow_mut().party_group_active = true;
            env.exec("TargetUnit('party1')").expect("target party1");
            env.fire_on_update(0.016).expect("tick party target auras");
            let party_mismatch: String = env
                .eval(
                    r#"
                    local name, icon = UnitAura('target', 1, 'HELPFUL')
                    if name ~= 'Power Word: Fortitude' or icon ~= 135987 then
                        return 'party1 target aura ' .. tostring(name) .. '/' .. tostring(icon)
                    end
                    if UnitAura('target', 2, 'HELPFUL') ~= nil then
                        return 'party1 target aura count above 1'
                    end
                    local button = _G['TargetFrameBuff1']
                    if not button or not button:IsShown() then
                        return 'TargetFrameBuff1 missing for party1'
                    end
                    if _G['TargetFrameBuff1Icon']:GetTexture() ~= 135987 then
                        return 'TargetFrameBuff1 icon not updated for party1'
                    end
                    if _G['TargetFrameBuff2'] and _G['TargetFrameBuff2']:IsShown() then
                        return 'TargetFrameBuff2 shown for a single party1 aura'
                    end
                    return ''
                "#,
                )
                .expect("classic party target aura probe");
            assert_eq!(
                party_mismatch, "",
                "classic target frame party aura projection"
            );
            let errors = drain_test_errors(&env);
            assert!(
                errors.is_empty(),
                "classic party target aura update errors: {errors:?}"
            );
        }
    });
}
}

//! Forever pending pet orders; no world-position or pet movement simulation.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn summon_pet(env: &WowLuaEnv) {
    env.state().borrow_mut().pet.icon = Some("Interface/Icons/Ability_Hunter_Pet_Wolf".into());
}

#[test]
fn forever_pet_move_order_query_returns_one_boolean_and_requires_a_pet_to_begin() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local query = rawget(C_PetInfo, 'IsPetOrderMoveToPending')
        assert(type(query) == 'function')
        assert(query() == false and type(query()) == 'boolean')
        assert(select('#', query()) == 1)
        assert(query('ignored', {}) == false)
        PetMoveTo()
        assert(query() == false, 'no summoned pet cannot start a move order')
        CameraOrSelectOrMoveStart()
        CameraOrSelectOrMoveStop(false)
        assert(query() == false)
        "#,
    )
    .unwrap();
    let other_env = WowLuaEnv::new().unwrap();
    summon_pet(&env);
    env.exec("PetMoveTo(); assert(C_PetInfo.IsPetOrderMoveToPending())")
        .unwrap();
    other_env
        .exec("assert(not C_PetInfo.IsPetOrderMoveToPending())")
        .expect("pet orders must remain local to their environment");
}

#[test]
fn forever_pet_move_order_begin_and_screen_confirmation_publish_synchronous_edges() {
    let env = WowLuaEnv::new().unwrap();
    summon_pet(&env);
    env.exec(
        r#"
        local query = C_PetInfo.IsPetOrderMoveToPending
        local events = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('PET_ORDER_MOVE_TO_PENDING_CHANGED')
        frame:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'PET_ORDER_MOVE_TO_PENDING_CHANGED')
            assert(select('#', ...) == 1)
            local pending = ...
            assert(type(pending) == 'boolean' and pending == query())
            events[#events + 1] = pending
        end)
        assert(select('#', PetMoveTo()) == 0)
        assert(query() == true and #events == 1 and events[1] == true)
        PetMoveTo()
        assert(#events == 1, 'unchanged pending state must not publish another edge')
        CameraOrSelectOrMoveStop(false)
        assert(query(), 'a release without a pending screen press cannot confirm')
        CameraOrSelectOrMoveStart()
        assert(query() and #events == 1)
        CameraOrSelectOrMoveStop(false)
        assert(query() == false and #events == 2 and events[2] == false)
        CameraOrSelectOrMoveStop(false)
        assert(#events == 2)
        "#,
    )
    .unwrap();
}

#[test]
fn forever_pet_move_order_cancel_preserves_dragged_cursor_and_survives_cleanup() {
    let env = WowLuaEnv::new().unwrap();
    summon_pet(&env);
    env.exec(
        r#"
        PickupSpell(133)
        local cursorType, cursorID = GetCursorInfo()
        assert(cursorType == 'spell')
        PetMoveTo()
        assert(C_PetInfo.IsPetOrderMoveToPending())
        assert(SpellIsTargeting() == false, 'pet orders are outside spell targeting')
        assert(GetCursorInfo() == cursorType)
        CameraOrSelectOrMoveStart()
        assert(SpellStopTargeting() == true)
        assert(C_PetInfo.IsPetOrderMoveToPending() == false)
        local afterType, afterID = GetCursorInfo()
        assert(afterType == cursorType and afterID == cursorID)
        assert(SpellStopTargeting() == false)
        PetMoveTo()
        CameraOrSelectOrMoveStop(false)
        assert(C_PetInfo.IsPetOrderMoveToPending(), 'cancel must discard the old press')
        "#,
    )
    .unwrap();
    env.apply_post_load_workarounds();
    env.exec(
        r#"
        assert(type(rawget(C_PetInfo, 'IsPetOrderMoveToPending')) == 'function')
        assert(C_PetInfo.IsPetOrderMoveToPending())
        assert(select('#', C_PetInfo.IsPetOrderMoveToPending()) == 1)
        assert(SpellStopTargeting() == true)
        assert(not C_PetInfo.IsPetOrderMoveToPending())
        "#,
    )
    .expect("cleanup must preserve the query and cancellation lifecycle");
}

#[test]
fn forever_pet_move_order_producers_preserve_existing_addon_protection() {
    let env = WowLuaEnv::new().unwrap();
    summon_pet(&env);
    let directory = tempfile::tempdir().unwrap();
    let addon = directory.path().join("PetMoveOrderProbe");
    std::fs::create_dir(&addon).unwrap();
    let toc = addon.join("PetMoveOrderProbe.toc");
    std::fs::write(&toc, "## Title: PetMoveOrderProbe\nprobe.lua\n").unwrap();
    std::fs::write(
        addon.join("probe.lua"),
        r#"
        function InsecurePetMoveBegin() PetMoveTo() end
        function InsecurePetMoveConfirm()
            CameraOrSelectOrMoveStart()
            CameraOrSelectOrMoveStop(false)
        end
        function InsecurePetMoveCancel() SpellStopTargeting() end
        function InsecurePetMoveQuery() return C_PetInfo.IsPetOrderMoveToPending() end
        "#,
    )
    .unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    env.exec(
        r#"
        InsecurePetMoveBegin()
        assert(C_PetInfo.IsPetOrderMoveToPending() == false)
        "#,
    )
    .expect("insecure begin must preserve the empty pending state");
    // The VM carries addon taint through the remainder of a calling chunk.
    // Start each secure producer in a fresh untainted host invocation.
    env.exec(
        r#"
        PetMoveTo()
        assert(C_PetInfo.IsPetOrderMoveToPending() == true)
        "#,
    )
    .expect("secure begin must establish a pending command");
    env.exec(
        r#"
        assert(InsecurePetMoveQuery() == true, 'query remains readable to addons')
        InsecurePetMoveConfirm()
        InsecurePetMoveCancel()
        assert(C_PetInfo.IsPetOrderMoveToPending() == true)
        "#,
    )
    .expect("new pet-order producers must pass through the existing protected wrappers");
    env.exec("assert(SpellStopTargeting() == true)")
        .expect("a fresh secure caller must still be able to cancel");
}

#[test]
fn forever_pet_move_order_selected_gamepad_consumer_observes_and_confirms_real_pending_state() {
    use std::path::Path;
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let addon = ui.join("Blizzard_GamepadActionBars");
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(!toc.is_game_type_restricted());
    assert!(toc.allows_screen(ScreenKind::Game));
    assert_eq!(
        toc.metadata.get("AllowLoadGameType").map(String::as_str),
        Some("camelot")
    );
    let index = toc
        .files
        .iter()
        .position(|path| path == Path::new("MainActionBarFrame.lua"))
        .expect("selected gamepad TOC must include the original consumer");
    assert!(toc.file_allows_screen(index, ScreenKind::Game));
    let source = std::fs::read_to_string(addon.join(&toc.files[index])).unwrap();
    assert!(source.contains("C_PetInfo.IsPetOrderMoveToPending()"));
    assert!(source.contains("CameraOrSelectOrMoveStop(false)"));

    let env = WowLuaEnv::new().unwrap();
    summon_pet(&env);
    // Load the whole selected file unchanged. Inspect its local action table
    // through existing debug support, without replacing any runtime API.
    env.exec(&source)
        .expect("load selected MainActionBarFrame.lua");
    env.exec(
        r#"
        local function upvalue(fn, wanted)
            for index = 1, 100 do
                local name, value = debug.getupvalue(fn, index)
                if name == wanted then return value end
                if name == nil then break end
            end
            error('missing original upvalue: ' .. wanted)
        end
        local resolve = upvalue(GamepadMainActionBarFrameMixin.UpdateInteractIcons,
            'ResolveNonBindableInteract')
        local actions = upvalue(resolve, 'interactActions')
        local petAction = actions[1]
        assert(petAction.condition({}) == false, 'reject constant true')
        PetMoveTo()
        assert(petAction.condition({}) == true, 'reject constant false/no-op producer')
        petAction.action()
        assert(petAction.condition({}) == false, 'original screen-click action must confirm')
        PetMoveTo()
        assert(SpellStopTargeting())
        assert(petAction.condition({}) == false, 'original condition observes cancellation')
        "#,
    )
    .expect("unchanged selected gamepad condition and confirmation must consume real state");
}

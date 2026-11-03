//! WM61 S2 secure invocation through the selected Blizzard AuraUtil consumer.
#![cfg(feature = "retail-12-1-5")]

use crate::common::aura_container_harness::load_env;

#[test]
fn aura_priority_securecall_preserves_caller_source() {
    let env = load_env();
    env.exec(
        r#"
        local consumer = AuraUtil.IsPriorityDebuff
        local target = AuraUtil.CheckIsPriorityAura
        local info = debug.getinfo(consumer, 'S')
        assert(info.what == 'Lua' and info.source:find('AuraUtil.lua', 1, true))
        local hits, source = 0, nil
        local function observe(event)
            if event == 'call' and debug.getinfo(2, 'f').func == target then
                hits = hits + 1
                source = debug.getstacktaint()
            end
        end
        debug.setobjecttaint(observe, nil)
        local function addon()
            debug.sethook(observe, 'c')
            consumer(343960)
            debug.sethook()
            assert(hits == 1, 'original securecall target was not reached')
            assert(source == 'SecureAuraProbe', 'securecall did not preserve caller source')
            assert(debug.getstacktaint() == 'SecureAuraProbe', 'caller source was not restored')
        end
        debug.setobjecttaint(addon, 'SecureAuraProbe')
        addon()
        "#,
    )
    .unwrap();
}

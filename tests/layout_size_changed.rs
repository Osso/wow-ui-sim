//! OnSizeChanged dispatch from the per-tick layout pass: explicit resizes, anchor-driven
//! resizes, and UntrustedLayoutScriptExecution suppression across anchored frames.

use wow_ui_sim::lua_api::WowLuaEnv;

const HELPERS: &str = r#"
    SizeLog = {}
    function RecordSize(tag)
        return function(_, w, h)
            SizeLog[#SizeLog + 1] = string.format('%s=%dx%d', tag, w, h)
        end
    end
    function TakeSizeLog()
        local log = table.concat(SizeLog, ',')
        SizeLog = {}
        return log
    end
"#;

fn env_with_helpers() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(HELPERS).unwrap();
    env
}

fn tick_and_take(env: &WowLuaEnv) -> String {
    env.fire_on_update(0.016).unwrap();
    env.eval::<String>("return TakeSizeLog()").unwrap()
}

#[test]
fn explicit_and_anchor_driven_resizes_fire_on_size_changed_once_per_change() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local parent = CreateFrame('Frame', 'SizeParent', UIParent)
        parent:SetPoint('CENTER')
        parent:SetSize(100, 50)
        local fill = CreateFrame('Frame', 'SizeFill', parent)
        fill:SetPoint('TOPLEFT', parent, 'TOPLEFT', 10, -5)
        fill:SetPoint('BOTTOMRIGHT', parent, 'BOTTOMRIGHT', -10, 5)
        local sibling = CreateFrame('Frame', 'SizeSibling', UIParent)
        sibling:SetPoint('TOPLEFT', parent, 'BOTTOMLEFT')
        sibling:SetPoint('TOPRIGHT', parent, 'BOTTOMRIGHT')
        sibling:SetHeight(4)
        parent:SetScript('OnSizeChanged', RecordSize('parent'))
        fill:SetScript('OnSizeChanged', RecordSize('fill'))
        sibling:SetScript('OnSizeChanged', RecordSize('sibling'))
        "#,
    )
    .unwrap();
    assert_eq!(
        tick_and_take(&env),
        "parent=100x50,fill=80x40,sibling=100x4",
        "first layout pass reports the initial size of every frame from 0x0"
    );
    assert_eq!(tick_and_take(&env), "", "no layout change, no dispatch");

    env.exec("SizeParent:SetSize(200, 60)").unwrap();
    assert_eq!(
        env.eval::<String>("return TakeSizeLog()").unwrap(),
        "",
        "dispatch is deferred to the layout pass"
    );
    assert_eq!(
        tick_and_take(&env),
        "parent=200x60,fill=180x50,sibling=200x4",
        "child and anchored sibling sizes follow the parent through anchors"
    );

    env.exec("SizeParent:ClearAllPoints(); SizeParent:SetPoint('TOPLEFT', 30, -30)")
        .unwrap();
    assert_eq!(tick_and_take(&env), "", "moving without resizing is not a size change");

    env.exec("SizeParent:SetScale(2)").unwrap();
    assert_eq!(
        tick_and_take(&env),
        "sibling=400x4",
        "scale keeps the scaled frames' own units; only the unscaled anchored sibling grows"
    );
}

#[test]
fn size_queried_mid_script_is_still_reported_by_the_next_layout_pass() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local f = CreateFrame('Frame', 'SizeQueried', UIParent)
        f:SetPoint('CENTER')
        f:SetScript('OnSizeChanged', RecordSize('queried'))
        f:SetSize(30, 20)
        assert(f:GetWidth() == 30)
        "#,
    )
    .unwrap();
    assert_eq!(tick_and_take(&env), "queried=30x20");
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn untrusted_layout_aspect_suppresses_addon_size_handlers_on_frames_anchored_to_it() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local function AddonFn(fn)
            debug.setobjecttaint(fn, 'LayoutSizeProbe')
            return fn
        end
        local guarded = CreateFrame('Frame', 'SizeGuarded', UIParent)
        guarded:SetPoint('CENTER')
        guarded:SetSize(40, 40)
        guarded:AddForbiddenAspects(Enum.ForbiddenAspect.UntrustedLayoutScriptExecution)
        local anchored = CreateFrame('Frame', 'SizeAnchored', UIParent)
        anchored:AddForbiddenAspects(guarded:GetInheritableForbiddenAspects(Enum.ScriptObjectPropagationPath.Layout))
        anchored:SetPoint('TOPLEFT', guarded, 'BOTTOMLEFT')
        anchored:SetPoint('TOPRIGHT', guarded, 'BOTTOMRIGHT')
        anchored:SetHeight(8)
        local plain = CreateFrame('Frame', 'SizePlain', UIParent)
        plain:SetPoint('CENTER')
        plain:SetSize(5, 5)
        for _, frame in ipairs({guarded, anchored, plain}) do
            local name = frame:GetName()
            frame:SetScript('OnSizeChanged', AddonFn(RecordSize(name .. ':addon')))
            frame:HookScript('OnSizeChanged', RecordSize(name .. ':secure'))
        end
        "#,
    )
    .unwrap();
    assert_eq!(
        tick_and_take(&env),
        "SizeGuarded:secure=40x40,SizeAnchored:secure=40x8,SizePlain:addon=5x5,SizePlain:secure=5x5"
    );
    env.exec("SizeGuarded:SetWidth(90)").unwrap();
    assert_eq!(
        tick_and_take(&env),
        "SizeGuarded:secure=90x40,SizeAnchored:secure=90x8",
        "an addon cannot observe the guarded frame's size through its own or anchored handlers"
    );
}

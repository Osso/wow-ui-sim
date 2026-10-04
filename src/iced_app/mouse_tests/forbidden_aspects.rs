use super::*;

#[test]
fn forbidden_aspect_mouse_editbox_focus_and_typing_survive_scripted_rejections() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
        AspectClickedBox = CreateFrame('EditBox', nil, UIParent)
        AspectOtherBox = CreateFrame('EditBox', nil, UIParent)
        AspectClickedBox:SetSize(100, 100)
        AspectClickedBox:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
        AspectClickedBox:SetFrameStrata('TOOLTIP')
        AspectClickedBox:EnableMouse(true)
        AspectClickedBox:SetAutoFocus(false)
        AspectOtherBox:SetAutoFocus(false)
        AspectClickedBox:SetText('ab')
        AspectClickedBox:SetCursorPosition(2)
        AspectClickedBox:ClearFocus()
        AspectOtherBox:ClearFocus()
        AspectFocusGained, AspectFocusLost = 0, 0
        AspectClickedBox:SetScript('OnEditFocusGained', function()
            AspectFocusGained = AspectFocusGained + 1
        end)
        AspectClickedBox:SetScript('OnEditFocusLost', function()
            AspectFocusLost = AspectFocusLost + 1
        end)
        AspectClickedBox:AddForbiddenAspects(Enum.ForbiddenAspect.ScriptedInput)
        AspectOtherBox:AddForbiddenAspects(Enum.ForbiddenAspect.ScriptedInput)
        assert(not AspectClickedBox:HasFocus() and not AspectOtherBox:HasFocus())
    "#,
        )
        .expect("create unfocused restricted editboxes");
    rebuild_hittable_cache(&app);
    let cursor = Point::new(150.0, 150.0);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    app.handle_mouse_down(cursor);
    app.handle_mouse_up(cursor);
    app.env
        .borrow()
        .exec(
            r#"
        assert(AspectClickedBox:HasFocus() and not AspectOtherBox:HasFocus())
        assert(AspectFocusGained == 1 and AspectFocusLost == 0)
        assert(AspectClickedBox:GetCursorPosition() == 2)
        local function addonCall(method, frame, ...)
            local args = {...}
            local function invoke() return frame[method](frame, unpack(args)) end
            debug.setobjecttaint(invoke, 'AspectMouseProbe')
            return pcall(invoke)
        end
        assert(not addonCall('SetFocus', AspectOtherBox))
        assert(not addonCall('ClearFocus', AspectClickedBox))
        assert(not addonCall('SetCursorPosition', AspectClickedBox, 0))
        assert(AspectClickedBox:HasFocus() and not AspectOtherBox:HasFocus())
        assert(AspectClickedBox:GetCursorPosition() == 2)
        assert(AspectClickedBox:GetText() == 'ab')
        assert(AspectFocusGained == 1 and AspectFocusLost == 0)
    "#,
        )
        .expect("GUI click acquires focus and rejected scripts preserve it and the cursor");
    let _ = app.update(Message::KeyPress(
        "X".to_string(),
        Some("x".to_string()),
        std::time::Instant::now(),
    ));
    app.env
        .borrow()
        .exec(
            r#"
        assert(AspectClickedBox:GetText() == 'abx')
        assert(AspectClickedBox:GetCursorPosition() == 3)
        assert(AspectClickedBox:HasFocus() and not AspectOtherBox:HasFocus())
        assert(AspectOtherBox:GetText() == '')
        assert(AspectFocusGained == 1 and AspectFocusLost == 0)
    "#,
        )
        .expect("GUI keyboard text reaches the mouse-focused restricted editbox");
}

#[test]
fn forbidden_aspect_mouse_scripted_click_rejects_lua_but_preserves_physical_click() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            AspectMouseButton = CreateFrame('Button', nil, UIParent)
            AspectMouseButton:SetSize(100, 100)
            AspectMouseButton:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
            AspectMouseButton:SetFrameStrata('TOOLTIP')
            AspectMouseButton:EnableMouse(true)
            AspectMouseClicks = 0
            AspectMouseButton:SetScript('OnClick', function()
                AspectMouseClicks = AspectMouseClicks + 1
            end)
            AspectMouseButton:Click()
            assert(AspectMouseClicks == 1, 'unrestricted Lua click must dispatch')
            "#,
        )
        .expect("create ordinary clickable control");
    rebuild_hittable_cache(&app);
    let cursor = Point::new(150.0, 150.0);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    app.handle_mouse_down(cursor);
    app.handle_mouse_up(cursor);
    app.env
        .borrow()
        .exec(
            r#"
            assert(AspectMouseClicks == 2, 'unrestricted physical click must dispatch')
            AspectMouseButton:AddForbiddenAspects(Enum.ForbiddenAspect.ScriptedInput)
            local function addonClick() AspectMouseButton:Click() end
            debug.setobjecttaint(addonClick, 'AspectMouseProbe')
            AspectMouseScriptAccepted = pcall(addonClick)
            AspectMouseAfterScript = AspectMouseClicks
            "#,
        )
        .expect("attempt scripted input on restricted button");
    app.handle_mouse_down(cursor);
    app.handle_mouse_up(cursor);
    app.env
        .borrow()
        .exec(
            r#"
            assert(AspectMouseClicks == AspectMouseAfterScript + 1,
                'ScriptedInput must not block physical GUI click delivery')
            assert(not AspectMouseScriptAccepted, 'restricted Lua Click must error')
            assert(AspectMouseAfterScript == 2, 'rejected Lua Click must not dispatch OnClick')
            "#,
        )
        .expect("reject only scripted click while physical input remains usable");
}

#[test]
fn forbidden_aspect_mouse_query_focus_rejects_only_annotated_query() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            AspectMouseFrame = CreateFrame('Frame', nil, UIParent)
            AspectMouseFrame:SetSize(100, 100)
            AspectMouseFrame:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
            AspectMouseFrame:SetFrameStrata('TOOLTIP')
            AspectMouseFrame:EnableMouse(true)
            AspectMouseEnters, AspectMouseLeaves = 0, 0
            AspectMouseFrame:SetScript('OnEnter', function()
                AspectMouseEnters = AspectMouseEnters + 1
            end)
            AspectMouseFrame:SetScript('OnLeave', function()
                AspectMouseLeaves = AspectMouseLeaves + 1
            end)
            "#,
        )
        .expect("create ordinary hover control");
    rebuild_hittable_cache(&app);
    let inside = Point::new(150.0, 150.0);
    let outside = Point::new(350.0, 350.0);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(inside)));
    app.env
        .borrow()
        .exec(
            r#"
            assert(AspectMouseEnters == 1 and AspectMouseLeaves == 0)
            assert(AspectMouseFrame:IsMouseMotionFocus(), 'ordinary focus query must work')
            AspectMouseFrame:AddForbiddenAspects(Enum.ForbiddenAspect.QueryFocus)
            local function addonQuery() return AspectMouseFrame:IsMouseMotionFocus() end
            debug.setobjecttaint(addonQuery, 'AspectMouseProbe')
            AspectMouseQueryAccepted = pcall(addonQuery)
            assert(AspectMouseFrame:IsMouseMotionFocus(), 'secure callers keep the focus query')
            assert(GetMouseFocus() == AspectMouseFrame)
            assert(GetMouseFoci()[1] == AspectMouseFrame)
            assert(AspectMouseFrame:IsMouseOver(), 'unannotated geometry query stays available')
            "#,
        )
        .expect("query restriction leaves actual hover and unannotated queries unchanged");
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(outside)));
    app.env
        .borrow()
        .exec(
            r#"
            assert(AspectMouseEnters == 1 and AspectMouseLeaves == 1)
            assert(GetMouseFocus() ~= AspectMouseFrame)
            assert(GetMouseFoci()[1] ~= AspectMouseFrame)
            assert(not AspectMouseFrame:IsMouseOver())
            "#,
        )
        .expect("real pointer departure still dispatches OnLeave and clears hover");
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(inside)));
    app.env
        .borrow()
        .exec(
            r#"
            assert(AspectMouseEnters == 2 and AspectMouseLeaves == 1)
            assert(GetMouseFocus() == AspectMouseFrame)
            assert(GetMouseFoci()[1] == AspectMouseFrame)
            assert(AspectMouseFrame:IsMouseOver())
            assert(not AspectMouseQueryAccepted, 'restricted IsMouseMotionFocus must error')
            "#,
        )
        .expect("restricted queries do not consume physical hover transitions");
}

#[test]
fn forbidden_aspect_always_propagate_input_forces_inherited_child_click_propagation() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            AspectPropagationDowns = {}
            local function makePair(name, x, aspect)
                local parent = CreateFrame('Button', name .. 'Parent', UIParent)
                parent:SetSize(100, 100)
                parent:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', x, -100)
                parent:SetFrameStrata('TOOLTIP')
                parent:SetScript('OnMouseDown', function()
                    AspectPropagationDowns[#AspectPropagationDowns + 1] = name .. 'Parent'
                end)
                if aspect then parent:AddForbiddenAspects(aspect) end
                local child = CreateFrame('Button', name .. 'Child', parent)
                child:SetAllPoints(parent)
                child:SetScript('OnMouseDown', function()
                    AspectPropagationDowns[#AspectPropagationDowns + 1] = name .. 'Child'
                end)
                return child
            end
            local forced = makePair('Forced', 100, Enum.ForbiddenAspect.AlwaysPropagateInput)
            local plain = makePair('Plain', 300, nil)
            assert(forced:CanPropagateMouseClicks() and forced:CanPropagateMouseMotion())
            assert(forced:GetPropagateKeyboardInput(), 'keyboard propagation is forced too')
            assert(not plain:CanPropagateMouseClicks() and not plain:GetPropagateKeyboardInput())
            local function addonClear() forced:SetPropagateMouseClicks(false) end
            debug.setobjecttaint(addonClear, 'AspectPropagationProbe')
            assert(not pcall(addonClear), 'addons cannot clear forced mouse propagation')
            forced:SetPropagateMouseClicks(false)
            assert(forced:CanPropagateMouseClicks(), 'stored preference never hides the forced aspect')
            "#,
        )
        .expect("create forced and plain parent/child pairs");
    rebuild_hittable_cache(&app);
    for x in [150.0, 350.0] {
        let point = Point::new(x, 150.0);
        app.handle_mouse_down(point);
        app.handle_mouse_up(point);
    }
    let downs: String = app
        .env
        .borrow()
        .eval("return table.concat(AspectPropagationDowns, ',')")
        .expect("read dispatched mouse downs");
    assert_eq!(downs, "ForcedChild,ForcedParent,PlainChild");
}

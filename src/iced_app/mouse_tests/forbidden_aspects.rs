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
        assert(not pcall(AspectOtherBox.SetFocus, AspectOtherBox))
        assert(not pcall(AspectClickedBox.ClearFocus, AspectClickedBox))
        assert(not pcall(AspectClickedBox.SetCursorPosition, AspectClickedBox, 0))
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
            AspectMouseScriptAccepted = pcall(function() AspectMouseButton:Click() end)
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
            AspectMouseQueryAccepted = pcall(function()
                return AspectMouseFrame:IsMouseMotionFocus()
            end)
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

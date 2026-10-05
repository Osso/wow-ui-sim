use super::*;

fn build_simulated_mouse_target() -> (App, Point) {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            SimTarget = CreateFrame('Button', nil, UIParent)
            SimTarget:SetSize(100, 100)
            SimTarget:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
            SimTarget:SetFrameStrata('TOOLTIP')
            SimTarget:EnableMouse(true)
            SimTarget:EnableMouseWheel(true)
            SimTarget:RegisterForClicks('LeftButtonUp', 'RightButtonUp')
            SimLog = {}
            SimTarget:SetScript('OnMouseDown', function(_, button) table.insert(SimLog, 'down:' .. button) end)
            SimTarget:SetScript('OnMouseUp', function(_, button) table.insert(SimLog, 'up:' .. button) end)
            SimTarget:SetScript('OnClick', function(_, button) table.insert(SimLog, 'click:' .. button) end)
            SimTarget:SetScript('OnMouseWheel', function(_, delta) table.insert(SimLog, 'wheel:' .. delta) end)
            function SimLogText() return table.concat(SimLog, ',') end
            function AddonSimulate(name, ...)
                local args = {...}
                local function invoke() return _G[name](unpack(args)) end
                debug.setobjecttaint(invoke, 'SimulateMouseProbe')
                return invoke()
            end
            "#,
        )
        .expect("create simulated mouse target");
    rebuild_hittable_cache(&app);
    let cursor = Point::new(150.0, 150.0);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(cursor)));
    (app, cursor)
}

fn sim_log(app: &App) -> String {
    app.env.borrow().eval("return SimLogText()").unwrap()
}

fn queued_inputs(app: &App) -> usize {
    app.env
        .borrow()
        .state()
        .borrow()
        .simulated_mouse_inputs
        .len()
}

#[test]
fn secure_simulated_click_wheel_and_right_button_dispatch_on_next_tick() {
    let (mut app, _) = build_simulated_mouse_target();
    let returned: i64 = app
        .env
        .borrow()
        .eval("return select('#', SimulateMouseClick('LeftButton'))")
        .unwrap();
    assert_eq!(returned, 0);
    assert_eq!(queued_inputs(&app), 2, "click queues down and up");
    assert_eq!(sim_log(&app), "", "input waits for the dispatcher");

    app.dispatch_simulated_mouse_inputs();
    assert_eq!(queued_inputs(&app), 0);
    assert_eq!(
        sim_log(&app),
        "down:LeftButton,up:LeftButton,click:LeftButton"
    );

    app.env
        .borrow()
        .exec("SimLog = {}; SimulateMouseWheel(2); SimulateMouseDown('RightButton'); SimulateMouseUp('RightButton')")
        .unwrap();
    app.dispatch_simulated_mouse_inputs();
    assert_eq!(
        sim_log(&app),
        "wheel:2,down:RightButton,click:RightButton,up:RightButton"
    );
}

#[test]
fn simulated_input_is_refused_for_insecure_callers_and_restricted_foci() {
    let (mut app, _) = build_simulated_mouse_target();
    app.env
        .borrow()
        .exec("AddonSimulate('SimulateMouseClick', 'LeftButton'); AddonSimulate('SimulateMouseWheel', 1)")
        .unwrap();
    assert_eq!(
        queued_inputs(&app),
        0,
        "no gamepad limited input for addons"
    );

    let target_id = app
        .env
        .borrow()
        .state()
        .borrow()
        .hovered_frame
        .expect("hovered target");
    {
        let env = app.env.borrow();
        let mut state = env.state().borrow_mut();
        state.widgets.get_mut(target_id).unwrap().is_protected = true;
        state.player.in_combat = true;
    }
    app.env
        .borrow()
        .exec("SimulateMouseClick('LeftButton')")
        .unwrap();
    assert_eq!(
        queued_inputs(&app),
        0,
        "protected focus is locked down in combat"
    );

    app.env.borrow().state().borrow_mut().player.in_combat = false;
    app.env
        .borrow()
        .exec("SimulateMouseDown('LeftButton')")
        .unwrap();
    assert_eq!(
        queued_inputs(&app),
        1,
        "protected focus is allowed out of combat"
    );
    app.env
        .borrow()
        .state()
        .borrow_mut()
        .simulated_mouse_inputs
        .clear();

    app.env
        .borrow()
        .state()
        .borrow_mut()
        .widgets
        .get_mut(target_id)
        .unwrap()
        .forbidden = true;
    app.env
        .borrow()
        .exec("SimulateMouseUp('LeftButton')")
        .unwrap();
    assert_eq!(queued_inputs(&app), 0, "forbidden focus refuses simulation");
    app.dispatch_simulated_mouse_inputs();
    assert_eq!(sim_log(&app), "");
}

#[test]
fn simulated_mouse_rejects_invalid_arguments() {
    let (app, _) = build_simulated_mouse_target();
    app.env
        .borrow()
        .exec(
            r#"
            for _, name in ipairs({'SimulateMouseClick', 'SimulateMouseDown', 'SimulateMouseUp'}) do
                for _, value in ipairs({'MiddleButton', 'leftbutton', 1, false}) do
                    local ok, err = pcall(_G[name], value)
                    assert(not ok and tostring(err):find('button must be'), name .. ': ' .. tostring(err))
                end
                assert(not pcall(_G[name]))
            end
            for _, value in ipairs({'1', 0/0, math.huge}) do
                local ok, err = pcall(SimulateMouseWheel, value)
                assert(not ok and tostring(err):find('delta'), tostring(err))
            end
            "#,
        )
        .unwrap();
    assert_eq!(queued_inputs(&app), 0);
}

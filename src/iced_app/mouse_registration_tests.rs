use super::test_support::*;
use super::*;
use crate::screen::ScreenKind;

fn load_click_xml_addon(app: &App, xml: &str) {
    let directory = tempfile::tempdir().expect("create click XML addon directory");
    let toc = directory.path().join("XmlClickRegistration.toc");
    std::fs::write(
        &toc,
        "## Title: XmlClickRegistration\nfixture.lua\nfixture.xml\n",
    )
    .expect("write click XML addon TOC");
    std::fs::write(
        directory.path().join("fixture.lua"),
        r#"
        __xml_click_receipts = {}
        function XmlClickReceipt(self, button, down)
            table.insert(__xml_click_receipts,
                self:GetName() .. ":" .. button .. ":" .. tostring(down))
        end
        "#,
    )
    .expect("write click receipt handler");
    std::fs::write(directory.path().join("fixture.xml"), xml).expect("write click XML fixture");
    let env = app.env.borrow();
    env.register_rilua_function("__xml_click_edges", click_edges_during_xml_onload)
        .expect("register test-only OnLoad input probe");
    let loaded = crate::loader::load_addon(&env.loader_env(), &toc)
        .expect("load click XML addon through TOC");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

// Observe the dispatch predicate during OnLoad without re-entering App input
// while the loader holds the environment borrow.
fn click_edges_during_xml_onload(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let id = crate::lua_api::methods::frame_id_from_stack(state, 1)?;
    let edges = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        let frame = sim.widgets.get(id).expect("OnLoad frame exists");
        [
            super::mouse::frame_click_registration_matches(frame, "LeftButton", true),
            super::mouse::frame_click_registration_matches(frame, "LeftButton", false),
            super::mouse::frame_click_registration_matches(frame, "RightButton", true),
            super::mouse::frame_click_registration_matches(frame, "RightButton", false),
        ]
    };
    for accepted in edges {
        state.push(rilua::Val::Bool(accepted));
    }
    Ok(4)
}

fn assert_xml_click_receipts(app: &App, expected: &str) {
    let receipts: String = app
        .env
        .borrow()
        .eval("return table.concat(__xml_click_receipts, ';')")
        .expect("read physical XML button click receipts");
    assert_eq!(receipts, expected);
}

fn dispatch_left_and_right_clicks(app: &mut App, position: Point) {
    app.handle_mouse_down(position);
    app.handle_mouse_up(position);
    app.handle_right_mouse_down(position);
    app.handle_right_mouse_up(position);
}

#[test]
fn xml_register_for_clicks_applies_literal_edges_preserves_default_and_allows_lua_mutation() {
    let mut app = build_test_app(ScreenKind::Game);
    load_click_xml_addon(
        &app,
        r#"
        <Ui>
          <Button name="XmlUpClicks" parent="UIParent" registerForClicks="LeftButtonUp, RightButtonUp">
            <Size x="100" y="100"/>
            <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="100" y="-100"/></Anchors>
            <Scripts><OnClick>XmlClickReceipt(self, button, down)</OnClick></Scripts>
          </Button>
          <Button name="XmlMixedClicks" parent="UIParent" registerForClicks="RightButtonUp, LeftButtonDown">
            <Size x="100" y="100"/>
            <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="300" y="-100"/></Anchors>
            <Scripts><OnClick>XmlClickReceipt(self, button, down)</OnClick></Scripts>
          </Button>
          <Button name="XmlDefaultClicks" parent="UIParent">
            <Size x="100" y="100"/>
            <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="500" y="-100"/></Anchors>
            <Scripts><OnClick>XmlClickReceipt(self, button, down)</OnClick></Scripts>
          </Button>
        </Ui>
        "#,
    );
    rebuild_hittable_cache(&app);
    let up_position = Point::new(150.0, 150.0);
    let mixed_position = Point::new(350.0, 150.0);
    let default_position = Point::new(550.0, 150.0);

    app.handle_mouse_down(up_position);
    assert_xml_click_receipts(&app, "");
    app.handle_mouse_up(up_position);
    assert_xml_click_receipts(&app, "XmlUpClicks:LeftButton:false");
    app.handle_right_mouse_down(up_position);
    assert_xml_click_receipts(&app, "XmlUpClicks:LeftButton:false");
    app.handle_right_mouse_up(up_position);
    assert_xml_click_receipts(
        &app,
        "XmlUpClicks:LeftButton:false;XmlUpClicks:RightButton:false",
    );

    app.env.borrow().exec("__xml_click_receipts = {}").unwrap();
    app.handle_mouse_down(mixed_position);
    assert_xml_click_receipts(&app, "XmlMixedClicks:LeftButton:true");
    app.handle_mouse_up(mixed_position);
    assert_xml_click_receipts(&app, "XmlMixedClicks:LeftButton:true");
    app.handle_right_mouse_down(mixed_position);
    assert_xml_click_receipts(&app, "XmlMixedClicks:LeftButton:true");
    app.handle_right_mouse_up(mixed_position);
    assert_xml_click_receipts(
        &app,
        "XmlMixedClicks:LeftButton:true;XmlMixedClicks:RightButton:false",
    );

    app.env.borrow().exec("__xml_click_receipts = {}").unwrap();
    app.handle_right_mouse_down(default_position);
    app.handle_right_mouse_up(default_position);
    app.handle_mouse_down(default_position);
    assert_xml_click_receipts(&app, "");
    app.handle_mouse_up(default_position);
    assert_xml_click_receipts(&app, "XmlDefaultClicks:LeftButton:false");

    app.env
        .borrow()
        .exec(
            r#"
            __xml_click_receipts = {}
            XmlMixedClicks:RegisterForClicks("LeftButtonUp")
            "#,
        )
        .expect("mutate registration after XML creation");
    rebuild_hittable_cache(&app);
    app.handle_mouse_down(mixed_position);
    assert_xml_click_receipts(&app, "");
    app.handle_mouse_up(mixed_position);
    app.handle_right_mouse_down(mixed_position);
    app.handle_right_mouse_up(mixed_position);
    assert_xml_click_receipts(&app, "XmlMixedClicks:LeftButton:false");
}

const CLICK_TEMPLATE_XML: &str = r#"
    <Ui>
      <Button name="XmlClickBase" virtual="true" registerForClicks="LeftButtonUp, RightButtonUp">
        <Size x="100" y="100"/>
        <Scripts>
          <OnLoad>self.leftDownAtLoad, self.leftUpAtLoad, self.rightDownAtLoad, self.rightUpAtLoad = __xml_click_edges(self)</OnLoad>
          <OnClick>XmlClickReceipt(self, button, down)</OnClick>
        </Scripts>
      </Button>
      <Button name="XmlClickDerived" virtual="true" inherits="XmlClickBase" registerForClicks="RightButtonUp, LeftButtonDown"/>
      <Button name="XmlClickLeaf" virtual="true" inherits="XmlClickDerived"/>
      <Button name="XmlInheritedClicks" parent="UIParent" inherits="XmlClickBase">
        <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="100" y="-100"/></Anchors>
      </Button>
      <Button name="XmlDerivedClicks" parent="UIParent" inherits="XmlClickLeaf">
        <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="300" y="-100"/></Anchors>
      </Button>
      <Button name="XmlOverrideClicks" parent="UIParent" inherits="XmlClickLeaf" registerForClicks="LeftButtonUp, RightButtonUp">
        <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="500" y="-100"/></Anchors>
      </Button>
    </Ui>
"#;

#[test]
fn xml_click_templates_replace_inherited_edges_before_onload_and_physical_dispatch() {
    let mut app = build_test_app(ScreenKind::Game);
    load_click_xml_addon(&app, CLICK_TEMPLATE_XML);
    app.env
        .borrow()
        .exec(
            r#"
            assert(not XmlInheritedClicks.leftDownAtLoad and XmlInheritedClicks.leftUpAtLoad
                and not XmlInheritedClicks.rightDownAtLoad and XmlInheritedClicks.rightUpAtLoad,
                'base XML registration must be applied before instance OnLoad')
            assert(XmlDerivedClicks.leftDownAtLoad and not XmlDerivedClicks.leftUpAtLoad
                and not XmlDerivedClicks.rightDownAtLoad and XmlDerivedClicks.rightUpAtLoad,
                'derived registration must replace base and survive omitted leaf/instance before OnLoad')
            assert(not XmlOverrideClicks.leftDownAtLoad and XmlOverrideClicks.leftUpAtLoad
                and not XmlOverrideClicks.rightDownAtLoad and XmlOverrideClicks.rightUpAtLoad,
                'instance XML registration must replace derived registration before OnLoad')
            "#,
        )
        .expect("observe inherited XML input registration during OnLoad");
    rebuild_hittable_cache(&app);

    dispatch_left_and_right_clicks(&mut app, Point::new(150.0, 150.0));
    dispatch_left_and_right_clicks(&mut app, Point::new(350.0, 150.0));
    dispatch_left_and_right_clicks(&mut app, Point::new(550.0, 150.0));
    assert_xml_click_receipts(
        &app,
        concat!(
            "XmlInheritedClicks:LeftButton:false;XmlInheritedClicks:RightButton:false;",
            "XmlDerivedClicks:LeftButton:true;XmlDerivedClicks:RightButton:false;",
            "XmlOverrideClicks:LeftButton:false;XmlOverrideClicks:RightButton:false"
        ),
    );
}

#[test]
fn lua_create_frame_applies_xml_template_click_edges_before_onload_and_dispatch() {
    let mut app = build_test_app(ScreenKind::Game);
    load_click_xml_addon(&app, CLICK_TEMPLATE_XML);
    app.env
        .borrow()
        .exec(
            r#"
            LuaBaseClicks = CreateFrame('Button', 'LuaBaseClicks', UIParent, 'XmlClickBase')
            LuaBaseClicks:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -300)
            LuaDerivedClicks = CreateFrame('Button', 'LuaDerivedClicks', UIParent, 'XmlClickLeaf')
            LuaDerivedClicks:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 300, -300)
            assert(not LuaBaseClicks.leftDownAtLoad and LuaBaseClicks.leftUpAtLoad
                and not LuaBaseClicks.rightDownAtLoad and LuaBaseClicks.rightUpAtLoad,
                'Lua CreateFrame must apply base XML registration before OnLoad')
            assert(LuaDerivedClicks.leftDownAtLoad and not LuaDerivedClicks.leftUpAtLoad
                and not LuaDerivedClicks.rightDownAtLoad and LuaDerivedClicks.rightUpAtLoad,
                'Lua CreateFrame must apply derived XML override before OnLoad')
            "#,
        )
        .expect("create Lua buttons from loaded XML templates");
    rebuild_hittable_cache(&app);

    dispatch_left_and_right_clicks(&mut app, Point::new(150.0, 350.0));
    dispatch_left_and_right_clicks(&mut app, Point::new(350.0, 350.0));
    assert_xml_click_receipts(
        &app,
        concat!(
            "LuaBaseClicks:LeftButton:false;LuaBaseClicks:RightButton:false;",
            "LuaDerivedClicks:LeftButton:true;LuaDerivedClicks:RightButton:false"
        ),
    );
}

#[test]
fn register_for_clicks_any_up_matches_addon_lowercase_spelling() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            LowercaseAnyUpButton = CreateFrame("Button", "LowercaseAnyUpButton", UIParent)
            LowercaseAnyUpButton:SetSize(100, 100)
            LowercaseAnyUpButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            LowercaseAnyUpButton:RegisterForClicks("anyUp")
            LowercaseAnyUpButton:SetScript("OnClick", function(_, button, down)
                __lowercase_any_up_count = (__lowercase_any_up_count or 0) + 1
                __lowercase_any_up_button = button
                __lowercase_any_up_down = down
            end)

            __lowercase_any_up_count = 0
            __lowercase_any_up_button = nil
            __lowercase_any_up_down = nil
            "#,
        )
        .expect("lowercase anyUp setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);
    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let (count, button, down): (f64, String, bool) = app
        .env
        .borrow()
        .eval("return __lowercase_any_up_count, __lowercase_any_up_button, __lowercase_any_up_down")
        .expect("lowercase anyUp click state should be readable");
    assert_eq!(count, 1.0);
    assert_eq!(button, "LeftButton");
    assert!(!down, "anyUp click should pass down=false");
}

#[test]
fn left_button_up_fires_mouse_up_before_click() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            UpOrderButton = CreateFrame("Button", "UpOrderButton", UIParent)
            UpOrderButton:SetSize(100, 100)
            UpOrderButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            UpOrderButton:RegisterForClicks("LeftButtonUp")
            UpOrderButton:SetScript("OnMouseUp", function()
                __up_order = (__up_order or "") .. "up;"
            end)
            UpOrderButton:SetScript("OnClick", function()
                __up_order = (__up_order or "") .. "click;"
            end)
            __up_order = ""
            "#,
        )
        .expect("up-order setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);

    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let order: String = app
        .env
        .borrow()
        .eval("return __up_order")
        .expect("up-order should be readable");
    assert_eq!(order, "up;click;");
}

#[test]
fn right_click_handlers_see_held_alt_modifier() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            AltRightButton = CreateFrame("Button", "AltRightButton", UIParent)
            AltRightButton:SetSize(100, 100)
            AltRightButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            AltRightButton:RegisterForClicks("RightButtonUp")
            AltRightButton:SetScript("OnClick", function(_, button)
                __alt_right_button = button
                __alt_right_seen = IsAltKeyDown()
            end)

            __alt_right_button = nil
            __alt_right_seen = false
            "#,
        )
        .expect("alt right-click setup should succeed");
    }

    let _ = app.update(crate::iced_app::Message::ModifiersChanged(
        iced::keyboard::Modifiers::ALT,
    ));
    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);
    app.handle_right_mouse_down(click_pos);
    app.handle_right_mouse_up(click_pos);

    let (button, alt_seen): (String, bool) = app
        .env
        .borrow()
        .eval("return __alt_right_button, __alt_right_seen")
        .expect("alt right-click result should be readable");
    assert_eq!(button, "RightButton");
    assert!(alt_seen, "right-click handlers should see held Alt");
}

#[test]
fn register_for_clicks_left_button_down_fires_click_on_mouse_down_only() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            DownClickButton = CreateFrame("Button", "DownClickButton", UIParent)
            DownClickButton:SetSize(100, 100)
            DownClickButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            DownClickButton:RegisterForClicks("LeftButtonDown")
            DownClickButton:SetScript("OnClick", function(_, button, down)
                __down_click_count = (__down_click_count or 0) + 1
                __down_click_button = button
                __down_click_down = down
            end)

            __down_click_count = 0
            __down_click_button = nil
            __down_click_down = nil
            "#,
        )
        .expect("down-click setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);

    app.handle_mouse_down(click_pos);

    let (count_after_down, button, down): (f64, String, bool) = app
        .env
        .borrow()
        .eval("return __down_click_count, __down_click_button, __down_click_down")
        .expect("down-click state should be readable after mouse down");
    assert_eq!(count_after_down, 1.0);
    assert_eq!(button, "LeftButton");
    assert!(down, "LeftButtonDown click should pass down=true");

    app.handle_mouse_up(click_pos);

    let count_after_up: f64 = app
        .env
        .borrow()
        .eval("return __down_click_count")
        .expect("down-click count should be readable after mouse up");
    assert_eq!(
        count_after_up, 1.0,
        "LeftButtonDown registration should not fire OnClick again on mouse up"
    );
}

#[test]
fn register_for_mouse_matches_addon_lowercase_spelling() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            LowercaseMouseRegisteredButton = CreateFrame("Button", "LowercaseMouseRegisteredButton", UIParent)
            LowercaseMouseRegisteredButton:SetSize(100, 100)
            LowercaseMouseRegisteredButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            LowercaseMouseRegisteredButton:RegisterForMouse("leftbuttondown", "leftbuttonup")
            LowercaseMouseRegisteredButton:SetScript("OnMouseDown", function(_, button)
                __lowercase_mouse_down = (__lowercase_mouse_down or "") .. button .. ";"
            end)
            LowercaseMouseRegisteredButton:SetScript("OnMouseUp", function(_, button)
                __lowercase_mouse_up = (__lowercase_mouse_up or "") .. button .. ";"
            end)

            __lowercase_mouse_down = ""
            __lowercase_mouse_up = ""
            "#,
        )
        .expect("lowercase mouse registration setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);

    app.handle_right_mouse_down(click_pos);
    app.handle_right_mouse_up(click_pos);
    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let (down_buttons, up_buttons): (String, String) = app
        .env
        .borrow()
        .eval("return __lowercase_mouse_down, __lowercase_mouse_up")
        .expect("lowercase mouse registration counters should be readable");
    assert_eq!(down_buttons, "LeftButton;");
    assert_eq!(up_buttons, "LeftButton;");
}

#[test]
fn register_for_mouse_restricts_physical_mouse_button_events() {
    let mut app = build_test_app(ScreenKind::Game);

    {
        let env = app.env.borrow();
        env.exec(
            r#"
            MouseRegisteredButton = CreateFrame("Button", "MouseRegisteredButton", UIParent)
            MouseRegisteredButton:SetSize(100, 100)
            MouseRegisteredButton:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            MouseRegisteredButton:RegisterForMouse("LeftButtonDown", "LeftButtonUp")
            MouseRegisteredButton:SetScript("OnMouseDown", function(_, button)
                __mouse_registered_down = (__mouse_registered_down or "") .. button .. ";"
            end)
            MouseRegisteredButton:SetScript("OnMouseUp", function(_, button)
                __mouse_registered_up = (__mouse_registered_up or "") .. button .. ";"
            end)

            __mouse_registered_down = ""
            __mouse_registered_up = ""
            "#,
        )
        .expect("mouse registration setup should succeed");
    }

    rebuild_hittable_cache(&app);
    let click_pos = Point::new(150.0, 150.0);

    app.handle_right_mouse_down(click_pos);
    app.handle_right_mouse_up(click_pos);
    app.handle_mouse_down(click_pos);
    app.handle_mouse_up(click_pos);

    let (down_buttons, up_buttons): (String, String) = app
        .env
        .borrow()
        .eval("return __mouse_registered_down, __mouse_registered_up")
        .expect("mouse registration counters should be readable");
    assert_eq!(down_buttons, "LeftButton;");
    assert_eq!(up_buttons, "LeftButton;");
}

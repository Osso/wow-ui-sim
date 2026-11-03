use super::test_support::*;
use super::*;
use crate::iced_app::mouse::frame_click_registration_matches;
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
            frame_click_registration_matches(frame, "LeftButton", true),
            frame_click_registration_matches(frame, "LeftButton", false),
            frame_click_registration_matches(frame, "RightButton", true),
            frame_click_registration_matches(frame, "RightButton", false),
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
fn mouse_up_returns_to_each_press_target_without_clicking_outside() {
    use crate::iced_app::Message;
    use iced::{Event, Rectangle, Size, mouse, widget::shader};

    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            MouseReleaseLog = {}
            local function makeButton(name, x)
                local frame = CreateFrame("Button", name, UIParent)
                frame:SetSize(100, 100)
                frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", x, -100)
                frame:RegisterForClicks("LeftButtonUp", "RightButtonUp")
                frame:SetScript("OnMouseUp", function(_, button, upInside)
                    table.insert(MouseReleaseLog, name .. ":" .. button .. ":" .. tostring(upInside))
                end)
                frame:SetScript("OnClick", function()
                    table.insert(MouseReleaseLog, name .. ":click")
                end)
            end
            makeButton("MouseReleaseFirst", 100)
            makeButton("MouseReleaseSecond", 300)
            "#,
        )
        .expect("mouse release buttons should load");
    rebuild_hittable_cache(&app);

    let first = Point::new(150.0, 150.0);
    let second = Point::new(350.0, 150.0);
    let release_outside = |app: &mut App, button: mouse::Button| {
        let event = Event::Mouse(mouse::Event::ButtonReleased(button));
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
        let cursor = mouse::Cursor::Available(Point::new(850.0, 150.0));
        let action =
            <&App as shader::Program<Message>>::update(&&*app, &mut (), &event, bounds, cursor)
                .expect("ordinary canvas should route the outside release");
        let (message, _, _) = action.into_inner();
        let _ = app.update(message.expect("release should publish a message"));
    };
    app.handle_mouse_down(first);
    app.handle_right_mouse_down(second);
    app.handle_mouse_up(second);
    assert!(app.env.borrow().state().borrow().mouse_buttons.right);
    release_outside(&mut app, mouse::Button::Right);

    app.handle_mouse_down(first);
    release_outside(&mut app, mouse::Button::Left);

    app.handle_mouse_down(first);
    app.handle_mouse_up(first);
    app.handle_right_mouse_down(second);
    app.handle_right_mouse_up(second);
    app.handle_mouse_up(second);

    let receipts: String = app
        .env
        .borrow()
        .eval("return table.concat(MouseReleaseLog, ';')")
        .expect("mouse release receipts should be readable");
    assert_eq!(
        receipts,
        concat!(
            "MouseReleaseFirst:LeftButton:false;",
            "MouseReleaseSecond:RightButton:false;",
            "MouseReleaseFirst:LeftButton:false;",
            "MouseReleaseFirst:LeftButton:true;MouseReleaseFirst:click;",
            "MouseReleaseSecond:click;MouseReleaseSecond:RightButton:true"
        ),
    );
}

#[test]
fn clipped_mouse_up_reports_outside_without_clicking() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            local parent = CreateFrame("Frame", nil, UIParent)
            parent:SetSize(100, 100)
            parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            parent:SetClipsChildren(true)

            local button = CreateFrame("Button", nil, parent)
            button:SetSize(100, 100)
            button:SetPoint("TOPLEFT", parent, "TOPLEFT", 50, 0)
            button:SetScript("OnMouseUp", function(_, _, upInside)
                ClippedReleaseInside = upInside
            end)
            button:SetScript("OnClick", function() ClippedReleaseClicked = true end)
            "#,
        )
        .unwrap();
    rebuild_hittable_cache(&app);

    app.handle_mouse_down(Point::new(175.0, 150.0));
    app.handle_mouse_up(Point::new(225.0, 150.0));
    app.env
        .borrow()
        .exec("assert(ClippedReleaseInside == false and not ClippedReleaseClicked)")
        .unwrap();
}

#[test]
fn outside_release_without_a_game_press_preserves_hover() {
    let mut app = build_test_app(ScreenKind::Game);
    setup_pass_through_test_frames(&app);
    rebuild_hittable_cache(&app);
    let cursor = Point::new(150.0, 150.0);
    app.handle_mouse_move(cursor);
    let hovered = app.hovered_frame;
    assert!(hovered.is_some());

    app.handle_mouse_up_outside();
    app.handle_right_mouse_up_outside();

    assert_eq!(app.hovered_frame, hovered);
    assert_eq!(app.mouse_position, Some(cursor));
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
fn double_click_on_title_button_opens_content_on_second_release() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r##"
            DoubleClickContent = CreateFrame("Frame", nil, UIParent)
            DoubleClickContent:Hide()
            DoubleClickTrace = ""

            local function header(name, x)
                local button = CreateFrame("Button", name, UIParent)
                button:SetSize(100, 40)
                button:SetPoint("TOPLEFT", UIParent, "TOPLEFT", x, -100)
                button:RegisterForClicks("LeftButtonUp")
                button:SetScript("OnDoubleClick", function(self, buttonName, ...)
                    DoubleClickTarget = self:GetName()
                    DoubleClickButton = buttonName
                    DoubleClickExtraArgs = select("#", ...)
                    DoubleClickTrace = DoubleClickTrace .. "double;"
                    DoubleClickContent:Show()
                end)
            end

            header("DoubleClickNeighbor", 100)
            header("DoubleClickTitle", 200)
            DoubleClickNeighbor:SetScript("OnClick", function()
                DoubleClickTrace = DoubleClickTrace .. "click;"
            end)
            DoubleClickNeighbor:SetScript("PostClick", function()
                DoubleClickTrace = DoubleClickTrace .. "post;"
            end)
            "##,
        )
        .expect("double-click header setup should succeed");
    rebuild_hittable_cache(&app);

    let neighbor = Point::new(199.0, 120.0);
    let title = Point::new(201.0, 120.0);
    let content_shown = |app: &App| {
        app.env
            .borrow()
            .eval::<bool>("return DoubleClickContent:IsShown()")
            .expect("content visibility should be readable")
    };

    app.handle_mouse_down(neighbor);
    app.handle_mouse_up(neighbor);
    assert!(
        !content_shown(&app),
        "a single click must leave content hidden"
    );

    app.handle_mouse_down(title);
    app.handle_mouse_up(title);
    assert!(
        !content_shown(&app),
        "nearby clicks on separate buttons must not pair"
    );

    app.handle_mouse_down(title);
    assert!(
        !content_shown(&app),
        "LeftButtonUp registration must wait for release"
    );
    app.handle_mouse_up(title);
    assert!(
        content_shown(&app),
        "the second title click must open content"
    );
    let (target, button, extra_args): (String, String, f64) = app
        .env
        .borrow()
        .eval("return DoubleClickTarget, DoubleClickButton, DoubleClickExtraArgs")
        .expect("read double-click arguments");
    assert_eq!(target, "DoubleClickTitle");
    assert_eq!(button, "LeftButton");
    assert_eq!(extra_args, 0.0);

    app.handle_mouse_down(neighbor);
    app.handle_mouse_up(neighbor);
    app.handle_mouse_down(neighbor);
    app.handle_mouse_up(neighbor);
    let trace: String = app
        .env
        .borrow()
        .eval("return DoubleClickTrace")
        .expect("read click dispatch sequence");
    assert_eq!(trace, "click;post;double;click;post;double;");
}

#[test]
fn right_double_click_dispatches_xml_postcall_without_a_normal_binding() {
    let mut app = build_test_app(ScreenKind::Game);
    load_click_xml_addon(
        &app,
        r#"
        <Ui>
          <Button name="XmlDoubleClick" parent="UIParent" registerForClicks="RightButtonUp">
            <Size x="100" y="100"/>
            <Anchors><Anchor point="TOPLEFT" relativeTo="UIParent" relativePoint="TOPLEFT" x="100" y="-100"/></Anchors>
            <Scripts>
              <OnClick>XmlClickReceipt(self, button, down)</OnClick>
              <OnDoubleClick intrinsicOrder="postcall">XmlClickReceipt(self, button, "double")</OnDoubleClick>
            </Scripts>
          </Button>
        </Ui>
        "#,
    );
    app.env
        .borrow()
        .exec(
            r#"
            assert(XmlDoubleClick:GetScript("OnDoubleClick") == nil)
            assert(type(XmlDoubleClick:GetScript("OnDoubleClick", 2)) == "function")
            "#,
        )
        .expect("only the XML postcall binding should be installed");
    rebuild_hittable_cache(&app);

    let position = Point::new(150.0, 150.0);
    app.handle_right_mouse_down(position);
    app.handle_right_mouse_up(position);
    assert_xml_click_receipts(
        &app,
        "XmlDoubleClick:RightButton:false",
    );
    app.handle_right_mouse_down(position);
    app.handle_right_mouse_up(position);
    assert_xml_click_receipts(
        &app,
        "XmlDoubleClick:RightButton:false;XmlDoubleClick:RightButton:double",
    );
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

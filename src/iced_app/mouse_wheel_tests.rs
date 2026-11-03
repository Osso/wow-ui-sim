use super::test_support::*;
use super::*;
use crate::screen::ScreenKind;

#[test]
fn wheel_only_viewport_keeps_scrolling_after_a_row_leaves_the_cursor() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            WheelPanel = CreateFrame("Button", "WheelPanel", UIParent)
            WheelPanel:SetSize(120, 120)
            WheelPanel:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            WheelPanel:EnableMouseWheel(true)
            WheelPanel:SetScript("OnMouseWheel", function(self, delta)
                self.wheelDelta = (self.wheelDelta or 0) + delta
            end)

            WheelViewport = CreateFrame("ScrollFrame", "WheelViewport", WheelPanel)
            WheelViewport:SetSize(100, 100)
            WheelViewport:SetPoint("TOPLEFT", WheelPanel)
            WheelViewport:EnableMouseWheel(true)
            WheelViewport:SetScript("OnMouseWheel", function(self, delta)
                self:SetVerticalScroll(self:GetVerticalScroll() - 40 * delta)
            end)

            local content = CreateFrame("Frame", nil, WheelViewport)
            content:SetSize(100, 280)
            content:SetPoint("TOPLEFT", WheelViewport)
            WheelViewport:SetScrollChild(content)

            WheelRow = CreateFrame("Button", "WheelRow", content)
            WheelRow:SetSize(50, 35)
            WheelRow:SetPoint("TOPLEFT", content, "TOPLEFT", 15, -15)
            "#,
        )
        .unwrap();
    rebuild_hittable_cache(&app);
    let (row, panel) = {
        let env = app.env.borrow();
        let state = env.state().borrow();
        (
            state.widgets.get_id_by_name("WheelRow").unwrap(),
            state.widgets.get_id_by_name("WheelPanel").unwrap(),
        )
    };
    let cursor = Point::new(130.0, 130.0);
    app.handle_mouse_move(cursor);
    assert_eq!(app.hit_test(cursor), Some(row));
    app.handle_scroll(0.0, -1.0);
    assert_eq!(
        app.hit_test(cursor),
        Some(panel),
        "wheel-only viewport should remain transparent to hover",
    );
    app.handle_scroll(0.0, -1.0);
    app.env
        .borrow()
        .exec(
            r#"
            assert(WheelViewport:GetVerticalScroll() == 80)
            assert(WheelPanel.wheelDelta == nil)
            WheelViewport:EnableMouseWheel(false)
            "#,
        )
        .unwrap();
    app.handle_scroll(0.0, -1.0);
    app.env
        .borrow()
        .exec(
            r#"
            assert(WheelViewport:GetVerticalScroll() == 80)
            assert(WheelPanel.wheelDelta == -1)
            WheelViewport:EnableMouseWheel(true)
            "#,
        )
        .unwrap();
    app.handle_scroll(0.0, -1.0);
    app.env
        .borrow()
        .exec("assert(WheelViewport:GetVerticalScroll() == 120)")
        .unwrap();
}

#[test]
fn wheel_over_click_only_child_prefers_its_eligible_ancestor() {
    let mut app = build_test_app(ScreenKind::Game);
    app.env
        .borrow()
        .exec(
            r#"
            WheelRecipients = {}
            local function createWheelFrame(name, level)
                local frame = CreateFrame("Frame", name, UIParent)
                frame:SetSize(100, 100)
                frame:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
                frame:SetFrameLevel(level)
                frame:EnableMouseWheel(true)
                frame:SetScript("OnMouseWheel", function()
                    table.insert(WheelRecipients, name)
                end)
                return frame
            end

            local parent = createWheelFrame("WheelClickParent", 10)
            createWheelFrame("WheelBehind", 20)
            local child = CreateFrame("Frame", "WheelClickChild", parent)
            child:SetAllPoints(parent)
            child:SetFrameStrata("DIALOG")
            child:SetMouseClickEnabled(true)
            "#,
        )
        .unwrap();
    rebuild_hittable_cache(&app);

    let cursor = Point::new(150.0, 150.0);
    let child = app
        .env
        .borrow()
        .state()
        .borrow()
        .widgets
        .get_id_by_name("WheelClickChild")
        .unwrap();
    assert_eq!(
        app.cached_hittable
            .borrow()
            .as_ref()
            .unwrap()
            .topmost_matching_at(cursor, |_| true),
        Some(child)
    );
    app.handle_mouse_move(cursor);
    app.handle_scroll(0.0, -1.0);
    app.env
        .borrow()
        .exec("WheelClickParent:EnableMouseWheel(false)")
        .unwrap();
    app.handle_scroll(0.0, -1.0);
    let recipients: String = app
        .env
        .borrow()
        .eval("return table.concat(WheelRecipients, ',')")
        .unwrap();
    assert_eq!(recipients, "WheelClickParent,WheelBehind");
}

#[test]
fn xml_wheel_script_enables_input_for_templates_and_respects_opt_out() {
    let mut app = build_test_app(ScreenKind::Game);
    let addon = tempfile::tempdir().unwrap();
    let toc = addon.path().join("WheelXml.toc");
    std::fs::write(&toc, "## Title: WheelXml\nWheelXml.xml\n").unwrap();
    std::fs::write(
        addon.path().join("WheelXml.xml"),
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
          <ScrollFrame name="WheelXmlBase" virtual="true">
            <Scripts>
              <OnMouseWheel>self:SetVerticalScroll(self:GetVerticalScroll() - 40 * delta)</OnMouseWheel>
            </Scripts>
          </ScrollFrame>
          <ScrollFrame name="WheelXmlDirect" parent="UIParent">
            <Scripts>
              <OnMouseWheel>self:SetVerticalScroll(self:GetVerticalScroll() - 40 * delta)</OnMouseWheel>
            </Scripts>
          </ScrollFrame>
          <ScrollFrame name="WheelXmlOptOut" parent="UIParent">
            <Scripts>
              <OnMouseWheel autoEnableInput="false">self:SetVerticalScroll(self:GetVerticalScroll() - 40 * delta)</OnMouseWheel>
            </Scripts>
          </ScrollFrame>
        </Ui>
        "#,
    )
    .unwrap();
    {
        let env = app.env.borrow();
        crate::loader::load_addon(&env.loader_env(), &toc).unwrap();
        env.exec(
            r#"
            WheelXmlInstance = CreateFrame("ScrollFrame", "WheelXmlInstance", UIParent, "WheelXmlBase")
            WheelXmlInstance:SetSize(100, 100)
            WheelXmlInstance:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
            assert(WheelXmlDirect:IsMouseWheelEnabled())
            assert(WheelXmlOptOut:HasScript("OnMouseWheel"))
            assert(not WheelXmlOptOut:IsMouseWheelEnabled())
            assert(WheelXmlInstance:IsMouseWheelEnabled())
            assert(not WheelXmlInstance:IsMouseMotionEnabled())
            "#,
        )
        .unwrap();
    }
    rebuild_hittable_cache(&app);
    app.handle_mouse_move(Point::new(150.0, 150.0));
    app.handle_scroll(0.0, -1.0);
    app.env
        .borrow()
        .exec("assert(WheelXmlInstance:GetVerticalScroll() == 40)")
        .unwrap();
}

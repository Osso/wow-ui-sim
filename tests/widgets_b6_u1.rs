use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;
fn wrath() -> bool {
    ACTIVE == ClientProfile::Wrath
}
#[path = "common/wrath_label_fixtures.rs"]
mod originals;
use originals::{lfr_env, tooltip_env};

fn read_stored_texture_size(env: &WowLuaEnv, name: &str) -> (f32, f32) {
    let sim = env.state().borrow();
    let id = sim.widgets.get_id_by_name(name).unwrap();
    let frame = sim.widgets.get(id).unwrap();
    (frame.width, frame.height)
}

#[test]
fn b6_u1_original_lfr_row_attachment() {
    if !wrath() {
        return;
    }
    let env = lfr_env();
    assert_eq!(
        read_stored_texture_size(&env, "B6TipTexture1"),
        (12.0, 12.0)
    );
    assert_eq!(
        read_stored_texture_size(&env, "B6TipTexture10"),
        (12.0, 12.0)
    );
    env.exec(r#"
        assert(B6Tip:NumLines()==5,'texture calls must not add rows')
        assert(B6Tip:GetLeftLine(2):GetText()=='Leader')
        assert(B6Tip:GetLeftLine(3):GetText()=='2 members')
        assert(math.abs(B6TipTexture1:GetWidth()-12)<0.01)
        assert(math.abs(B6TipTexture1:GetHeight()-12)<0.01)
        assert(B6TipTexture1:GetNumPoints()==1)
        local point,relative,relativePoint=B6TipTexture1:GetPoint(1)
        assert(point=='TOPRIGHT' and relative==B6TipTextLeft2 and relativePoint=='TOPLEFT')
        local tlx,tly,blx,bly,trx,try,brx,bry=B6TipTexture1:GetTexCoord()
        assert(tlx==0 and tly==0 and blx==0 and bly==1 and trx==0.25 and try==0 and brx==0.25 and bry==1,'UV crop retained')
    "#).unwrap();
}
#[test]
fn b6_u1_original_lfr_gap_is_local_6_4_at_four_three() {
    if !wrath() {
        return;
    }
    let env = lfr_env();
    {
        let mut sim = env.state().borrow_mut();
        sim.screen_width = 1600.0;
        sim.screen_height = 1200.0;
        sim.ensure_layout_rects();
    }
    let gap: f64 = env
        .eval("return B6TipTextLeft2:GetLeft()-B6TipTexture1:GetRight()")
        .unwrap();
    assert!((gap - 6.4).abs() < 0.01, "local 4:3 DDC gap: {gap}");
}

#[test]
fn b6_u1_attached_icon_follows_caption_after_cached_layout() {
    if !wrath() {
        return;
    }
    let env = lfr_env();
    env.state().borrow_mut().ensure_layout_rects();
    let before: (f64, f64) = env
        .eval("return B6TipTextLeft2:GetLeft(),B6TipTexture1:GetRight()")
        .unwrap();
    env.exec("B6TipTextLeft2:SetPoint('TOPLEFT',B6TipTextLeft1,'BOTTOMLEFT',100,-2)")
        .unwrap();
    env.state().borrow_mut().ensure_layout_rects();
    let after: (f64, f64) = env
        .eval("return B6TipTextLeft2:GetLeft(),B6TipTexture1:GetRight()")
        .unwrap();
    assert!(
        (after.0 - before.0).abs() > 0.1,
        "fixture must actually move caption"
    );
    assert!(
        ((after.0 - before.0) - (after.1 - before.1)).abs() < 0.01,
        "icon must follow its anchored sibling after cached layout"
    );
}

#[test]
fn b6_u1_xml_stored_size_parent_fill_and_single_attachment_anchor() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    assert_eq!(
        read_stored_texture_size(&env, "B6TipTexture1"),
        (12.0, 12.0)
    );
    env.exec(
        r#"
        assert(B6TipTexture1:GetNumPoints()==2)
        local p1,r1,rp1=B6TipTexture1:GetPoint(1)
        local p2,r2,rp2=B6TipTexture1:GetPoint(2)
        assert(p1=='TOPLEFT' and r1==B6Tip and rp1=='TOPLEFT')
        assert(p2=='BOTTOMRIGHT' and r2==B6Tip and rp2=='BOTTOMRIGHT')
        B6Tip:SetSize(200,100)
        assert(math.abs(B6TipTexture1:GetWidth()-B6Tip:GetWidth())<0.01)
        assert(math.abs(B6TipTexture1:GetHeight()-B6Tip:GetHeight())<0.01)
        B6Tip:AddLine('header'); B6Tip:AddLine('body')
        B6Tip:AddTexture('')
        assert(B6Tip:NumLines()==2 and B6TipTexture1:GetTexture()==nil)
        assert(B6TipTexture1:GetNumPoints()==1)
        local p,r,rp,x,y=B6TipTexture1:GetPoint(1)
        assert(p=='TOPRIGHT' and r==B6TipTextLeft2 and rp=='TOPLEFT' and x<0 and y==0)
        assert(math.abs(B6TipTexture1:GetWidth()-12)<0.01)
        assert(math.abs(B6TipTexture1:GetHeight()-12)<0.01)
    "#,
    )
    .unwrap();
    assert_eq!(
        read_stored_texture_size(&env, "B6TipTexture1"),
        (12.0, 12.0)
    );
}
#[test]
fn b6_u1_no_slot_and_too_few_rows_do_not_insert() {
    if !wrath() {
        return;
    }
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local tip=CreateFrame('GameTooltip','B6Bare',UIParent)
        tip:AddTexture(''); assert(tip:NumLines()==0)
        tip:AddLine('header'); tip:AddTexture(''); assert(tip:NumLines()==1)
        tip:AddLine('body'); tip:AddTexture(''); assert(tip:NumLines()==2)
        assert(B6BareTexture1==nil)
    "#,
    )
    .unwrap();
}

#[test]
fn b6_u1_numeric_legacy_input_attaches_not_new_row() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    env.exec(
        r#"
        B6Tip:AddLine('header'); B6Tip:AddLine('body')
        B6Tip:AddTexture(136243)
        assert(B6Tip:NumLines()==2,'numeric file identifier must not add a row')
    "#,
    )
    .unwrap();
}
#[cfg(feature = "gui")]
fn b6_measure_tip(env: &WowLuaEnv, fonts: &mut wow_ui_sim::render::WowFontSystem) -> (f64, f64) {
    env.exec("B6Tip:Show()").unwrap();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut env.state().borrow_mut(), fonts);
    env.eval("return B6Tip:GetWidth(),B6Tip:GetHeight()")
        .unwrap()
}
#[cfg(feature = "gui")]
#[test]
fn b6_u1_slot_size_blank_spacing_and_ten_slot_cap() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    let mut fonts = wow_ui_sim::render::WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP: fonts required"
    );
    // Every body line is longer than the header; space additions therefore
    // change the max body width rather than being masked by header width.
    env.exec("B6Tip:AddLine('h'); B6Tip:AddLine(string.rep('W',30)); B6TipTexture1:SetSize(18,7)")
        .unwrap();
    let stored_size = read_stored_texture_size(&env, "B6TipTexture1");
    assert_eq!(stored_size, (18.0, 7.0));
    let before = b6_measure_tip(&env, &mut fonts);
    env.exec("B6Tip:AddTexture('')").unwrap();
    let after = b6_measure_tip(&env, &mut fonts);
    assert!(
        after.0 > before.0,
        "fresh empty attachment must reserve width"
    );
    assert!(
        (after.1 - before.1).abs() < 0.01,
        "slot height must not replace left height"
    );
    let effective_size: (f64, f64) = env
        .eval("return B6TipTexture1:GetWidth(),B6TipTexture1:GetHeight()")
        .unwrap();
    assert!((effective_size.0 - stored_size.0 as f64).abs() < 0.01);
    assert!((effective_size.1 - stored_size.1 as f64).abs() < 0.01);
    assert_eq!(read_stored_texture_size(&env, "B6TipTexture1"), stored_size);
    env.exec("assert(B6TipTexture1:GetTexture()==nil)").unwrap();
    for index in 2..=10 {
        env.exec(&format!(
            "B6Tip:AddLine(string.rep('W',{})); B6Tip:AddTexture('')",
            30 + index * 2
        ))
        .unwrap();
    }
    env.exec("B6Tip:AddLine(string.rep('W',60))").unwrap();
    let cap_before = b6_measure_tip(&env, &mut fonts);
    env.exec("B6Tip:AddTexture('')").unwrap();
    let cap_after = b6_measure_tip(&env, &mut fonts);
    assert_eq!(cap_before, cap_after, "11th attachment is a no-op at cap");
    env.exec("assert(B6Tip:NumLines()==12 and B6TipTexture11==nil)")
        .unwrap();
}

#[test]
fn b6_u1_previous_most_recent_texture_aligns_captions_not_icon_left_edges() {
    if !wrath() {
        return;
    }
    let env = tooltip_env();
    env.exec(
        r#"
        B6Tip:AddLine('header');B6Tip:AddLine('caption two')
        B6TipTexture1:SetSize(20,12);B6TipTexture2:SetSize(30,12)
        B6Tip:AddTexture('');B6Tip:Show()
    "#,
    )
    .unwrap();
    env.state().borrow_mut().ensure_layout_rects();
    env.exec(
        r#"
        B6Row2X=B6TipTextLeft2:GetLeft()
        B6Icon1Left=B6TipTexture1:GetLeft()
        B6Gap=B6Row2X-B6TipTexture1:GetRight()
        assert(B6Gap>0,'icon right must be left of text')
        B6Tip:AddLine('caption three');B6Tip:AddTexture('');B6Tip:Show()
    "#,
    )
    .unwrap();
    env.state().borrow_mut().ensure_layout_rects();
    env.exec(r#"
        local function near(a,b) assert(math.abs(a-b)<0.01) end
        near(B6TipTextLeft2:GetLeft(),B6Row2X)
        near(B6TipTextLeft3:GetLeft(),B6Row2X)
        near(B6TipTextLeft3:GetLeft()-B6TipTexture2:GetRight(),B6Gap)
        assert(math.abs(B6TipTexture2:GetLeft()-B6Icon1Left)>0.01,'unequal icons have different left edges')
        -- Preanchored NEXT template slot baseline uses CURRENT width, not old width.
        near(B6TipTextLeft4:GetLeft(),B6TipTextLeft3:GetLeft()-B6TipTexture2:GetWidth()-B6Gap)
        B6Tip:AddLine('no attachment four')
        B6Tip:AddLine('caption five');B6TipTexture3:SetWidth(16);B6Tip:AddTexture('');B6Tip:Show()
    "#).unwrap();
    env.state().borrow_mut().ensure_layout_rects();
    env.exec(
        r#"
        local offset=B6TipTextLeft5:GetLeft()-B6TipTextLeft4:GetLeft()
        assert(math.abs(offset-B6TipTexture3:GetWidth()-B6Gap)<0.01,
            'nonadjacent most-recent texture uses width+gap offset')
    "#,
    )
    .unwrap();
}

//! R88l.2 VERIFIED API branches; explicit host seams are not native writers.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::globals::real::merchant_item_link::{
    MerchantLinkOffer as Offer, MerchantNamedLink,
};

#[test]
fn merchant_item_link_arguments_and_arity() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        for _, f in ipairs({function() GetMerchantItemLink() end,
            function() GetMerchantItemLink(nil) end,
            function() GetMerchantItemLink(true) end,
            function() GetMerchantItemLink('not a number') end}) do
            local ok, err = pcall(f)
            assert(not ok and tostring(err):find('Usage: GetMerchantItemLink(index)',1,true))
        end
        for _, n in ipairs({2147483648, -2147483649}) do
            local ok, err = pcall(GetMerchantItemLink,n)
            assert(not ok and tostring(err):find('integer overflow attempting to store '..string.format('%f',n),1,true))
        end
        for _, n in ipairs({-2147483648,2147483647,0,1,-1}) do
            assert(select('#',GetMerchantItemLink(n))==0)
        end
    "#).unwrap();
    env.state().borrow_mut().merchant_items = vec![6948];
    env.exec(
        r#"
        assert(select('#',GetMerchantItemLink(1,'ignored'))==1)
        assert(GetMerchantItemLink('1')==GetMerchantItemLink(1.9))
        assert(select('#',GetMerchantItemLink(-0.9))==0)
        local s=GetMerchantItemLink(1)
        assert(s:find('|Hitem:6948:',1,true))
        assert(not s:find(':0:',1,true) and not s:find(':80:',1,true))
        assert(s:find('|h[Hearthstone]|h|r',1,true))
    "#,
    )
    .unwrap();
}

#[test]
fn merchant_item_link_misses_and_builder_nil_are_distinct() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_items = vec![0, u32::MAX, 6948];
    env.exec("assert(select('#',GetMerchantItemLink(1))==0); assert(select('#',GetMerchantItemLink(2))==0)").unwrap();
    for offer in [
        Offer::Missing,
        Offer::Unsupported,
        Offer::Currency { record: None },
        Offer::Spell { record: None },
        Offer::ResolvedSpell { record: None },
    ] {
        env.state()
            .borrow_mut()
            .merchant_links
            .offers
            .insert(2, offer);
        env.exec("assert(select('#',GetMerchantItemLink(3))==0)")
            .unwrap();
    }
    env.state()
        .borrow_mut()
        .merchant_links
        .offers
        .insert(2, Offer::Item { fields: None });
    env.exec("assert(select('#',GetMerchantItemLink(3))==1); assert(GetMerchantItemLink(3)==nil)")
        .unwrap();
    env.state().borrow_mut().merchant_links.unavailable = true;
    env.exec("assert(select('#',GetMerchantItemLink(3))==0)")
        .unwrap();
    env.state().borrow_mut().merchant_links.unavailable = false;
    env.state()
        .borrow_mut()
        .merchant_links
        .rejected_slots
        .insert(2);
    env.exec("assert(select('#',GetMerchantItemLink(3))==0)")
        .unwrap();
}

#[test]
fn merchant_item_link_explicit_builder_fields_and_special_paths() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().merchant_items = vec![6948];
    env.state().borrow_mut().merchant_links.offers.insert(
        0,
        Offer::Item {
            fields: Some(vec![0, 10, 100, -2, 0]),
        },
    );
    env.exec("assert(GetMerchantItemLink(1):find('|Hitem:6948::10:100:-2:|h',1,true))")
        .unwrap();
    let record = MerchantNamedLink {
        id: 42,
        second: 7,
        color: "TOKEN".into(),
        label: Some("Supplied record".into()),
    };
    env.state().borrow_mut().merchant_links.offers.insert(
        0,
        Offer::Currency {
            record: Some(record.clone()),
        },
    );
    env.exec("assert(GetMerchantItemLink(1)=='|cnTOKEN:|Hcurrency:42:7|h[Supplied record]|h|r')")
        .unwrap();
    for offer in [
        Offer::Spell {
            record: Some(record.clone()),
        },
        Offer::ResolvedSpell {
            record: Some(record.clone()),
        },
    ] {
        env.state()
            .borrow_mut()
            .merchant_links
            .offers
            .insert(0, offer);
        env.exec("assert(GetMerchantItemLink(1)=='|cTOKEN|Hspell:42:7|h[Supplied record]|h|r')")
            .unwrap();
    }
    let missing_text = MerchantNamedLink {
        label: None,
        ..record
    };
    env.state().borrow_mut().merchant_links.offers.insert(
        0,
        Offer::Currency {
            record: Some(missing_text),
        },
    );
    env.exec("assert(select('#',GetMerchantItemLink(1))==1); assert(GetMerchantItemLink(1)==nil)")
        .unwrap();
}

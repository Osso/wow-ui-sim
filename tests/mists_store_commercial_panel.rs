#![cfg(feature = "client-mists")]

#[path = "common/lua_error_output.rs"]
mod lua_error_output;

use std::process::Command;

#[path = "common/bounded_process.rs"]
mod bounded_process;

#[path = "common/wow_sim_binary.rs"]
mod wow_sim_binary;

#[test]
fn mists_store_catalog_checkout_and_token_surfaces_are_interactive() {
    let output = bounded_process::output(
        90,
        Command::new(wow_sim_binary::path()).args([
            "--no-addons",
            "--no-saved-vars",
            "--exec-lua",
            STORE_COMMERCIAL_LUA,
            "lua-errors",
        ]),
    )
    .expect("failed to run wow-sim");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "store commercial flow failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    lua_error_output::assert_no_lua_errors("store commercial flow", &stdout, &stderr);
}

const STORE_COMMERCIAL_LUA: &str = r#"
    local function fail(message)
        error(message, 0)
    end

    local function assertShown(frame, label)
        if not frame or not frame:IsShown() then
            fail(label .. " did not show")
        end
    end

    LoadAddOn("Blizzard_CatalogShop")
    LoadAddOn("Blizzard_WowTokenUI")
    LoadAddOn("Blizzard_SimpleCheckout")

    if type(StoreFrame) ~= "table" then
        fail("StoreFrame missing")
    end
    if type(CatalogShopFrame) ~= "table" then
        fail("CatalogShopFrame missing")
    end
    if type(WowTokenRedemptionFrame) ~= "table" then
        fail("WowTokenRedemptionFrame missing")
    end
    if type(SimpleCheckout) ~= "table" then
        fail("SimpleCheckout missing")
    end
    for _, soundKitKey in ipairs({
        "CATALOG_SHOP_OPEN_LOADING_SCREEN",
        "CATALOG_SHOP_LOADING_SCREEN_LOOP",
        "CATALOG_SHOP_OPEN_SHOP_AFTER_LOAD",
        "CATALOG_SHOP_GOLD_SHIMMER_START",
        "CATALOG_SHOP_GOLD_SHIMMER_LOOP",
        "CATALOG_SHOP_GOLD_SHIMMER_END",
    }) do
        if type(SOUNDKIT[soundKitKey]) ~= "number" then
            fail("CatalogShop SOUNDKIT missing " .. soundKitKey)
        end
        PlaySound(SOUNDKIT[soundKitKey])
    end

    StoreMicroButton:GetScript("OnClick")(StoreMicroButton, "LeftButton", false)

    assertShown(CatalogShopFrame, "CatalogShopFrame")

    local categoryIDs = C_CatalogShop.GetAvailableCategoryIDs()
    if not categoryIDs or #categoryIDs == 0 then
        fail("CatalogShop categories missing")
    end

    assertShown(CatalogShopFrame.ProductContainerFrame, "CatalogShop ProductContainerFrame")

    local productProvider = CatalogShopFrame.ProductContainerFrame.ProductsScrollBoxContainer.ScrollBox:GetDataProvider()
    if not productProvider or productProvider:GetSize() == 0 then
        fail("CatalogShop product provider missing")
    end

    local firstProduct = productProvider:FindElementDataByPredicate(function(elementData)
        return elementData.elementType == CatalogShopConstants.ScrollViewElementType.Product
    end)
    if not firstProduct then
        fail("CatalogShop first product missing")
    end

    local visibleProductFrames = 0
    CatalogShopFrame.ProductContainerFrame.ProductsScrollBoxContainer.ScrollBox:ForEachFrame(function(frame, elementData)
        if elementData.elementType == CatalogShopConstants.ScrollViewElementType.Product then
            if not frame:IsShown() or frame:GetWidth() <= 0 or frame:GetHeight() <= 0 then
                fail("CatalogShop product frame not visible or sized")
            end
            visibleProductFrames = visibleProductFrames + 1
        end
    end)
    if visibleProductFrames == 0 then
        fail("CatalogShop rendered product frames missing")
    end

    if not CatalogShopFrame.ProductContainerFrame:TrySelectProduct(firstProduct) then
        fail("CatalogShop product selection failed")
    end
    if CatalogShopFrame:GetSelectedProductInfo() ~= firstProduct then
        fail("CatalogShop selected product did not persist")
    end

    CatalogShopFrame:PurchaseProduct()

    SimpleCheckout:CalculateDesiredSize()
    SimpleCheckout:RecalculateSize()
    SimpleCheckout:Show()
    assertShown(SimpleCheckout, "SimpleCheckout")
    if SimpleCheckout:GetWidth() <= 0 or SimpleCheckout:GetHeight() <= 0 then
        fail("SimpleCheckout did not calculate a usable size")
    end
    SimpleCheckout:Hide()

    local price = C_WowTokenPublic.GetCurrentMarketPrice()
    local tokenCount = C_WowTokenSecure.GetTokenCount()
    if type(price) ~= "number" or price < 0 then
        fail("WoW Token market price probe failed")
    end
    if type(tokenCount) ~= "number" or tokenCount < 0 then
        fail("WoW Token secure count probe failed")
    end
    "#;

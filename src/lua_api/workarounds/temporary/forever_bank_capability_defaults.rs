//! Temporary Forever bank UI capability and tab-catalog gaps.
//! Retire with supplied bank-type capabilities and purchased-tab metadata.

const BANK_CAPABILITIES: &str = r#"
-- Missing system: bank-type UI capability policy, not ordinary bag inventory.
if rawget(C_Bank, "ShouldUsePlayerBagsInBank") == nil then
    function C_Bank.ShouldUsePlayerBagsInBank()
        return false
    end
end

-- Missing system: purchased bank-tab metadata (IDs, names, icons and flags).
-- Legacy bank bag counts do not provide a BankTabData catalog.
if rawget(C_Bank, "FetchPurchasedBankTabData") == nil then
    function C_Bank.FetchPurchasedBankTabData(_bankType)
        return {}
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(BANK_CAPABILITIES)?;
    Ok(())
}

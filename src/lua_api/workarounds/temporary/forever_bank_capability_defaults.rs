//! Temporary Forever bank tab-catalog gap.
//! Retire when purchased-tab metadata is supplied.

const BANK_CAPABILITIES: &str = r#"
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

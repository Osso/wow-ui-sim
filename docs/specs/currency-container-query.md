# Currency-container query

Retail and PTR `C_CurrencyInfo.IsCurrencyContainer(currencyID, quantity)` returns one boolean. The generated Retail declaration is `CurrencyInfoDocumentation.lua:442-455`. Both arguments require signed 32-bit numeric conversion, with range checked before truncation. A currency matches if any associated row has an inclusive minimum and an inclusive maximum, or a zero maximum. Unknown currencies return false. These decisions are VERIFIED and the boolean publisher is EMULATED in WM53-A.

Rows come from the 70391 `CurrencyContainer.csv` export, SHA-256 `438d4627054979bfc6fcb81fac3d1673400a1752b6ee0ded6ae4ef91ead01dd0`. The CurrencyTypeID, MinAmount and MaxAmount column meanings are INFERRED; their equality to 70688 data is unknown. Regenerate with `python3 tools/gen_currency_containers.py <CurrencyContainer.csv> crates/sim-data/data/currency_containers.rs`. Two independent outputs had SHA-256 `bfbf15b656c14aac9bb4f0eda14632f3d7d98f232640f3266c1e50a63afc6236`.

Native secret-policy installation and output secrecy remain unknown; this model does not claim secret parity.

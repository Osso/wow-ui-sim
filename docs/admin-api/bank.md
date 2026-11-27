# Bank state

Forever bank APIs use simulator host inputs. No bank types or purchased tab records exist by default.

### `A_Admin.SetBankViewableTypes(bankTypes)`

Supplies the bank types that the original bank UI can open. `bankTypes` must be a dense array of integer `Enum.BankType` values: `Character` (0), `Guild` (1), or `Account` (2). An empty array means no bank type is viewable.

### `A_Admin.SetShouldUsePlayerBagsInBank(enabled)`

Sets the boolean returned by `C_Bank.ShouldUsePlayerBagsInBank()`. The default is `false` because the host supplied no capability value.

### `A_Admin.SetPurchasedBankTabData(bankType, records)`

`bankType` must be one of the documented `Enum.BankType` values: `Character` (0), `Guild` (1), or `Account` (2).

Supplies records returned by `C_Bank.FetchPurchasedBankTabData(bankType)`. `records` must be a dense array. Each row requires:

- `ID`: unsigned integer bag index.
- `name`: string.
- `icon`: unsigned integer file ID.
- `depositFlags`: unsigned integer bag-slot flags.
- `tabCleanupConfirmation`: string.
- `tabNameEditBoxHeader`: string.

The returned rows include the supplied `bankType`. An omitted bank type returns an empty array. These records follow the pinned `Blizzard_APIDocumentationGenerated/BankDocumentation.lua` `BankTabData` shape. They are scenario inputs, not live account data.

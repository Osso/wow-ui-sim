# Installed-product fixture

`casc-installed-products.db` is synthetic protobuf, encoded independently of the production parser. It contains no account or installation-path data.

| Product code | Current version | Active build key | Active install key |
|---|---|---|---|
| `wow` | `12.1.0.69933` | `dcfc90fffd79ba00406ae46f5f657592` | 32 `a` characters |
| `wow_classic_beta` | `1.60.1.69977` | `3bd89ce2721f7c75e7525dc83741076f` | empty |

Both records set installed=true, playable=false and updateComplete=false. Completed and incomplete build keys are deliberately different: 32 `9` and 32 `8` characters respectively.

Wire structure: Database field 1 → ProductInstall fields 2 (product code), 4 (cached state) → CachedProductState field 1 → BaseProductState fields 1/2/3 (flags), 7 (version), 12 (completed key), 14 (active key), 16 (install key), 18 (incomplete key). Tags follow TACTLib's `TACTLib/Agent/Protobuf/ProtoDatabase.cs`.

The tests pair this database with a stale retail-only `.build.info`. Identity must come from the requested database record without modifying either file.

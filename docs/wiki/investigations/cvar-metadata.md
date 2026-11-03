# CVar metadata

`C_CVar.GetCVarInfo` combines live simulator values with profile-pinned native
metadata. Addons use its account and character flags to decide where settings
belong; returning arbitrary flags can change which settings they later restore.

## Content

### Query behavior

Retail, PTR and Forever register the query in the `C_CVar` namespace. A known
CVar returns seven values: current value, default value, server-account,
server-character, locked-from-user, secure and read-only. The current/default
values come from `CVarStorage`; the five flags come from the selected static map.
Lookup is case-insensitive, and secret names and invalid argument types fail.

Captured all-false flags remain distinct from missing metadata. Unknown names
and default-only names absent from the native catalog return no values.
Runtime-registered custom CVars have local storage and zero native flags. If a
built-in name is registered at runtime, its captured flags still take precedence.
Profiles without a matching snapshot use an empty map and do not register this
query. The metadata does not change the separate CVar write or combat policies.

### Native snapshots and data boundary

The maps contain flags from Wowless revision
`69a3654e08305de6fd5ce3533faf7e5597135109`. Its capture addon enumerates console
CVars and compares all seven native `C_CVar.GetCVarInfo` returns. These are
maintained native-query expectations; individual row-capture receipts are not
bundled, and absent names do not establish false flags.

| Profile | Snapshot version | Entries | Catalog last-change build |
|---|---|---:|---|
| Retail | 12.1.5.70575 | 1,663 | 70448 |
| PTR | 12.1.7.70493 | 1,780 | 70493 |
| Forever | 1.60.1.70532 | 1,786 | 70450 |

Snapshot versions and build configurations are immutable inputs, independent of
later UI-source pins. `sim-data-profile` selects generated PHF maps at compile
time. The maps contain no current/default values and require no runtime JSON
loading. Root `data/cvars-info/` retains source/output hashes, capture-contract
provenance and the upstream MIT license.

### Generation and regression coverage

The generator uses only the Python standard library:

```sh
python tools/gen_cvar_info.py --check
python -m unittest discover -s tools/tests -p test_gen_cvar_info.py
```

Omit `--check` to regenerate the three maps, provenance and license. The optional
`--cache-dir PATH` reuses downloaded sources and downloads any missing entries;
it does not enforce offline operation. Check mode compares every generated
output without rewriting repository files and accepts equivalent license line
endings while preserving an existing notice.

Generator tests cover captured-zero masks, invalid input, snapshot pinning,
provenance and check-mode behavior. Data-crate tests cover profile selection,
catalog sizes and flag masks. The `cvar_info::` integration tests cover the exact
tuple, live values, runtime registration, argument rejection and native flags;
Forever also checks temporary values without changing defaults. These checks do
not establish current native parity or compatibility with every addon.

## Sources

- [CVar query](../../../src/c_api/c_cvar_info.rs) and
  [storage](../../../src/cvars.rs) — live values and metadata precedence.
- [Generator](../../../tools/gen_cvar_info.py) and
  [provenance](../../../data/cvars-info/provenance.json) — immutable source pins,
  capture contract, profile counts and hashes.
- [Upstream license](../../../data/cvars-info/LICENSE.wowless) — data licensing.
- [API tests](../../../tests/cvar_info.rs),
  [catalog tests](../../../crates/sim-data-profile/src/cvar_info_tests.rs) and
  [generator tests](../../../tools/tests/test_gen_cvar_info.py) — bounded regression
  contracts.
- [Generated data boundary](../design/generated-data-crates.md) — compile-time
  profile selection and source-data placement.

## See Also

- [[client-profiles]] — runtime profile selection.
- [[lua-api]] — API registration and simulator state.

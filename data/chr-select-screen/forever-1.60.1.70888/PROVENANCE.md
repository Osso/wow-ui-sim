# Character select data — Forever build 1.60.1.70888

Source product: `wowf`; installed build: `1.60.1.70888`; build key: `0597e74b01a3f779d849ffe9b3270196`.
Simulator pin: `1.60.1.70707`. 70707 values are UNKNOWN; these rows are from installed build 70888.
WoWDBDefs revision: `d6f50d4f351193e84226aa62a36faa0f58528afa`.
**VERIFIED:** The DB2 files were extracted from the local CASC install and exported with the recorded layouts and schema hashes; no CDN data was used.

| Table | FileDataID | Layout | DB2 SHA-256 | DBD SHA-256 | Rows | Unreadable |
|---|---:|---|---|---|---:|---:|
| `ChrClasses` | 1361031 | `AFC9B0C2` | `4d9c28a14a1b75a1463e7202b63c325d49cf05b11302ac2d17266dac5ede7c38` | `3a3d7fe21beef5e07033dde5adb0965de6fd5adb065816ac56ee96fb7b10e7df` | 9 | 0 |
| `ChrRaces` | 1305311 | `4F44C796` | `0e00f5ad28a7a2e9e705a7cb6e2cfaa2367879b3aa9877d162e333d70a0fab58` | `53fd308737d92e7ea2cbffd77add22e82c5a85e0c725545249357e52064b61f6` | 58 | 0 |

The CSV files retain explicit zero values. Missing row IDs are not synthesized.
Regenerate with `tools/gen_chr_select_screen_data.py` and the same build/build key, local DB2 files, definitions revision, and game-engine `export_db2_csv.py` exporter.

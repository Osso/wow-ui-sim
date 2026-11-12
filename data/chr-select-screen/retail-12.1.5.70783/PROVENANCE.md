# Character select data — Retail build 12.1.5.70783

Source product: `wow`; installed build: `12.1.5.70783`; build key: `bc493ecb2898766c0f8947a99b4b3d65`.
Simulator pin: `12.1.5.70575`. Installed 70783 data is not a 70575 export.
WoWDBDefs revision: `d6f50d4f351193e84226aa62a36faa0f58528afa`.
**VERIFIED:** The DB2 files were extracted from the local CASC install and exported with the recorded layouts and schema hashes; no CDN data was used.

| Table | FileDataID | Layout | DB2 SHA-256 | DBD SHA-256 | Rows | Unreadable |
|---|---:|---|---|---|---:|---:|
| `ChrClasses` | 1361031 | `AFC9B0C2` | `ba7342eedbd7f06d9c7743cdc8b369cfbbed95958beba9093daf37a8a083daf6` | `3a3d7fe21beef5e07033dde5adb0965de6fd5adb065816ac56ee96fb7b10e7df` | 15 | 0 |
| `ChrRaces` | 1305311 | `4F44C796` | `e4b6f674e452e228ca8091e5bf19fcc489d3e0f316f8ddb856444528aca8384a` | `53fd308737d92e7ea2cbffd77add22e82c5a85e0c725545249357e52064b61f6` | 59 | 0 |

The CSV files retain explicit zero values. Missing row IDs are not synthesized.
Regenerate with `tools/gen_chr_select_screen_data.py` and the same build/build key, local DB2 files, definitions revision, and game-engine `export_db2_csv.py` exporter.

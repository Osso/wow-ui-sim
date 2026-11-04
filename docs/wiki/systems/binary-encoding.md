# Binary addon encoding

`C_EncodingUtil` implements Base64, compression and CBOR transformations over raw
Lua string bytes.

## Content

### API and byte boundaries

| APIs | Supported formats |
|---|---|
| `EncodeBase64`, `DecodeBase64` | Standard and URL-safe RFC 4648 alphabets |
| `CompressString`, `DecompressString` | Raw Deflate, Zlib and Gzip |
| `SerializeCBOR`, `DeserializeCBOR` | The definite-length CBOR subset below |

Successful transformations return one result except empty CBOR input, which is inferred to return zero. Binary strings preserve NUL
and invalid UTF-8 bytes. The VM applies `AllowedWhenUntainted` secret
authentication to source arguments and serialization values under all options.
For the three decoders, ordinary numbers are converted using Lua's 14-significant-digit format. Nil and tables fail the source check; CBOR reports their type, while Base64 and decompression report their usage strings. These checks are supported by static native evidence, not client execution.

Base64 encoding emits padding. Decoding accepts padded or unpadded input and
rejects invalid alphabet characters and padding bits. Compression uses raw
Deflate by default and offers levels for speed and size. Zlib and Gzip decoding
verifies the stream checksums. Decompression streams through bounded buffers
until completion and rejects incomplete streams. The documented decompressed
output limit is 104,857,600 bytes, and the encoding nesting limit is 100.

### CBOR values and limits

Decoding supports unsigned/negative integer widths through 64 bits, half/single/
double floats including infinities and NaN, booleans, null/undefined, byte/text
strings, arrays and maps. Numbers become Lua doubles, with their precision
limits. The decoder returns both CBOR string types as raw bytes. Arrays use
one-based numeric keys, and null and undefined become nil, including array
holes. Decoding preserves map key types and creates a fresh tree each time.

Tags, indefinite lengths, unsupported simple values, nil/NaN map keys,
nonempty truncated input, trailing CBOR data and excessive nesting are rejected.
These rules describe simulator behavior.

Native static analysis implies zero returns for empty CBOR, one empty string for empty Base64, and an internal decompression error for empty compressed input. These three end-to-end outcomes are **inferred**, because the native return or rethrow connections are not fully established. The selected CVar reader falls back to an empty table for empty CBOR. A truncated CBOR header raises `attempted to deserialize an incomplete or ill-formed data item`; native static analysis verifies that error path.

Serialization uses inferred wire choices. An exact dense `1..N` table, including
an empty table, becomes an array. Sparse or mixed tables become maps. Nil becomes
null, strings become byte strings, and exactly represented integers within the
contiguous double integer range use compact integer widths. Other numbers use
double wire values, preserving negative zero and large-double boundaries.
Repeated table references serialize as independent trees. Cycles and excess
depth always fail. `ignoreSerializationErrors=true` replaces unsupported values
with CBOR undefined, while unsupported keys and secret-access failures still
fail. Native map ordering, sparse-array choice and byte-for-byte serialization
are not established.

### Consumer and verification boundary

The selected QuestieDB reader uses these APIs to decode Base64, Zlib and CBOR
metadata and database rows. The decoder preserves the numeric keys and wide
race-mask values the reader consumes.

[Encoding regressions](../../../tests/encoding_util_binary.rs) include independent
wire vectors, binary round trips, malformed input, checksums, numeric boundaries,
fresh trees, options and limits. Set `QUESTIEDB_UI_PATH` to exercise the
installed reader. The wire vectors and round trips cover simulator format
behavior.

### Legacy serialized CVar storage

Older simulator builds stored the Delves companion's locked ability map as
JSON because both CBOR serialization and Base64 encoding were compatibility
substitutes. The storage loader now converts the recognized legacy shape for
`lastLockedTieredEntranceCompanionAbilities` to Base64 CBOR in memory. Numeric
companion and node IDs retain their key types and locked flags. The migration
applies only when reading that CVar's legacy shape from disk.

[Migration checks](../../../tests/forever_delves_cvar_migration.rs) exercise
the original selected CVar helper against the native empty default and
migrated values, validate the wire value, and verify the input file is intact.

## Sources

- [Implementation](../../../src/c_api/encoding_util.rs) and
  [CBOR model](../../../src/c_api/encoding_util/cbor.rs) — shared byte handling,
  compression and explicit serialization/decoding bounds.
- [Pinned Forever API documentation](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_APIDocumentationGenerated/EncodingUtilDocumentation.lua)
  and [constants](https://github.com/Gethe/wow-ui-source/blob/aff0d8cd53bf4d9ef1c1dd2786b94461a276fb16/Interface/AddOns/Blizzard_APIDocumentationGenerated/EncodingUtilConstantsDocumentation.lua)
  — signatures, enums, secret arguments and documented resource limits.
- [RFC 4648](https://www.rfc-editor.org/rfc/rfc4648),
  [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951),
  [RFC 1950](https://www.rfc-editor.org/rfc/rfc1950),
  [RFC 1952](https://www.rfc-editor.org/rfc/rfc1952) and
  [RFC 8949](https://www.rfc-editor.org/rfc/rfc8949) — binary format definitions.
- [QuestieDB client probes](https://github.com/Questie/QuestieDB/blob/master/docs/client-metadata-probes.md)
  — addon-author observations of fresh CBOR trees, sparse maps, floats and
  compression checksums.
- [Captured Forever CVar defaults](https://github.com/wowless/wowless/blob/69a3654e08305de6fd5ce3533faf7e5597135109/data/products/wow_classic_beta/cvars.yaml)
  and [storage migration](../../../src/cvars.rs) — native empty default and
  the narrow legacy simulator format boundary.

## See Also

- [[lua-api]] — publication and adjacent compatibility boundaries.

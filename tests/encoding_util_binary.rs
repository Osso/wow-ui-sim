use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn encoding_util_hex_matches_wire_bytes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local encode, decode = C_EncodingUtil.EncodeHex, C_EncodingUtil.DecodeHex
        local bytes = string.char(0, 1, 15, 16, 127, 128, 165, 255)
        assert(encode(bytes) == '00010f107f80a5ff')
        assert(decode('00010F107f80A5FF') == bytes)
        assert(encode('') == '' and select('#', encode('')) == 1)
        assert(decode('') == '' and select('#', decode('')) == 1)
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_hex_rejects_malformed_input() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local encode, decode = C_EncodingUtil.EncodeHex, C_EncodingUtil.DecodeHex
        assert(not pcall(decode, 'abc'))
        assert(select('#', decode('00g0')) == 0)
        assert(select('#', decode('0g')) == 0)
        assert(select('#', decode(' 0')) == 0)
        assert(not pcall(encode))
        assert(not pcall(decode))
        assert(not pcall(encode, {}))
        assert(not pcall(decode, {}))
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_base64_preserves_bytes_and_supports_both_alphabets() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local decode = C_EncodingUtil.DecodeBase64
        local bytes = string.char(251, 255, 254)
        assert(decode('+//+') == bytes)
        assert(decode('-__-', 1) == bytes)
        assert(decode('AAE=') == string.char(0, 1))
        assert(decode('AAE') == string.char(0, 1))
        assert(decode('') == '' and select('#', decode('')) == 1)
        assert(not pcall(decode, '-__-', 0))
        assert(not pcall(decode, '+//+', 1))
        assert(not pcall(decode, 'A'))
        assert(not pcall(decode, 'AAB='), 'nonzero padding bits are invalid')
        assert(not pcall(decode, '%%%%'))
        assert(not pcall(decode, 'AA==', 2))
        assert(not pcall(decode))
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_encodes_base64_wire_bytes_and_compresses_all_methods_and_levels() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local bytes = string.char(251, 255, 254)
        assert(C_EncodingUtil.EncodeBase64(bytes) == '+//+')
        assert(C_EncodingUtil.EncodeBase64(bytes, 1) == '-__-')
        assert(C_EncodingUtil.EncodeBase64(string.char(0,1)) == 'AAE=')
        assert(C_EncodingUtil.EncodeBase64('') == '')
        assert(not pcall(C_EncodingUtil.EncodeBase64, bytes, 2))
        assert(not pcall(C_EncodingUtil.EncodeBase64))
        local binary = string.rep(string.char(0,255,128,0), 5000)
        for method = 0, 2 do
            for level = 0, 2 do
                local compressed = C_EncodingUtil.CompressString(binary, method, level)
                assert(type(compressed) == 'string' and #compressed < #binary)
                assert(C_EncodingUtil.DecompressString(compressed, method) == binary)
                if method == 1 then assert(compressed:byte(1) == 120) end
                if method == 2 then assert(compressed:sub(1,2) == string.char(31,139)) end
            end
        end
        assert(C_EncodingUtil.DecompressString(C_EncodingUtil.CompressString(binary)) == binary)
        assert(C_EncodingUtil.DecompressString(C_EncodingUtil.CompressString('')) == '')
        assert(not pcall(C_EncodingUtil.CompressString, binary, 3))
        assert(not pcall(C_EncodingUtil.CompressString, binary, 0, 3))
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_decompresses_independent_binary_vectors_and_checks_corruption() {
    let env = WowLuaEnv::new().unwrap();
    // Independent Node zlib fixtures, not produced by simulator encoding APIs.
    env.exec(
        r#"
        local vectors = {
            'W8zYxMjkWCCh5ZyUmefC8L+BAQA=',
            'eJxbzNjEyORYIKHlnJSZ58Lwv4EBAC9MBVw=',
            'H4sIAAAAAAAAClvM2MTI5FggoeWclJnnwvC/gQEAdYKsAxIAAAA=',
        }
        local expected = string.char(163,1,130,1,2,65,112,24,42,67,98,105,110,68,0,255,128,0)
        for index, encoded in ipairs(vectors) do
            local compressed = C_EncodingUtil.DecodeBase64(encoded)
            local output = C_EncodingUtil.DecompressString(compressed, index - 1)
            assert(output == expected, 'binary output differs for method ' .. (index - 1))
            assert(select('#', C_EncodingUtil.DecompressString(compressed, index - 1)) == 1)
            if index == 1 then assert(C_EncodingUtil.DecompressString(compressed) == expected) end
            assert(not pcall(C_EncodingUtil.DecompressString, compressed:sub(1, -5), index - 1))
            if index > 1 then
                local corrupted = compressed:sub(1, -2) .. string.char((compressed:byte(-1) + 1) % 256)
                assert(not pcall(C_EncodingUtil.DecompressString, corrupted, index - 1))
            end
        end
        -- Independent Node zlib fixture expands across several 8192-byte
        -- output chunks; small fixtures alone miss streaming flush errors.
        local large = C_EncodingUtil.DecodeBase64(
            'eJztw4EJAAAIwKCd3ucF3aFgO6Wqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqv4DWuZ0Vw==')
        assert(C_EncodingUtil.DecompressString(large, 1) == string.rep(string.char(0,255,128,0), 10000))
        assert(not pcall(C_EncodingUtil.DecompressString, large:sub(1, -5), 1))
        assert(not pcall(C_EncodingUtil.DecompressString, '', 1))
        assert(not pcall(C_EncodingUtil.DecompressString, 'invalid', 3))
        assert(not pcall(C_EncodingUtil.DecompressString))
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_cbor_handles_integer_widths_floats_maps_holes_and_binary_strings() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local decode = C_EncodingUtil.DeserializeCBOR
        assert(decode(string.char(0)) == 0)
        assert(decode(string.char(24, 255)) == 255)
        assert(decode(string.char(25, 1, 0)) == 256)
        assert(decode(string.char(26, 0, 1, 0, 0)) == 65536)
        assert(decode(string.char(27, 0, 0, 0, 2, 0, 0, 0, 178)) == 8589934770)
        assert(decode(string.char(32)) == -1)
        assert(decode(string.char(56, 24)) == -25)
        assert(decode(string.char(249, 62, 0)) == 1.5)
        assert(decode(string.char(250, 71, 195, 80, 0)) == 100000)
        assert(decode(string.char(251, 63, 241, 153, 153, 153, 153, 153, 154)) == 1.1)
        assert(decode(string.char(249, 124, 0)) == math.huge)
        assert(decode(string.char(249, 252, 0)) == -math.huge)
        local nan = decode(string.char(249, 126, 0))
        assert(nan ~= nan)
        assert(decode(string.char(244)) == false and decode(string.char(245)) == true)
        for _, nilByte in ipairs({246, 247}) do
            assert(decode(string.char(nilByte)) == nil)
            assert(select('#', decode(string.char(nilByte))) == 1)
        end
        assert(decode(string.char(67, 0, 255, 128)) == string.char(0, 255, 128))
        assert(decode(string.char(99, 0, 255, 128)) == string.char(0, 255, 128))
        local array = decode(string.char(131, 1, 246, 2))
        assert(array[1] == 1 and array[2] == nil and array[3] == 2)
        local map = decode(string.char(163, 32, 1, 244, 2, 97, 49, 3))
        assert(map[-1] == 1 and map[false] == 2 and map['1'] == 3 and map[1] == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_cbor_rejects_malformed_unsupported_and_overdeep_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local decode = C_EncodingUtil.DeserializeCBOR
        assert(decode('') == nil and select('#', decode('')) == 1)
        for _, bytes in ipairs({string.char(24), string.char(67,1), string.char(129),
            string.char(161,1), string.char(192,0), string.char(159,255),
            string.char(28), string.char(255), string.char(0,0), string.char(161,246,0),
            string.char(161,249,126,0,0)}) do
            assert(not pcall(decode, bytes), 'must reject unsupported/truncated CBOR')
        end
        assert(not pcall(decode))
        assert(not pcall(decode, {}))
        local depth100 = string.rep(string.char(129), 100) .. string.char(0)
        assert(type(decode(depth100)) == 'table')
        assert(not pcall(decode, string.char(129) .. depth100))
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_cbor_returns_fresh_trees_and_cleanup_keeps_json_separate() {
    let env = WowLuaEnv::new().unwrap();
    let probe = r#"
        local bytes = string.char(163,1,130,1,2,65,112,24,42,67,98,105,110,68,0,255,128,0)
        local first = C_EncodingUtil.DeserializeCBOR(bytes)
        local second = C_EncodingUtil.DeserializeCBOR(bytes)
        assert(first ~= second and first[1] ~= second[1])
        assert(first.p == 42 and first.bin == string.char(0,255,128,0))
        first[1][1] = 99
        assert(second[1][1] == 1)
        assert(C_EncodingUtil.DeserializeJSON('{"ok":true}').ok == true)
        assert(not pcall(C_EncodingUtil.DeserializeCBOR, '{"ok":true}'))
    "#;
    env.exec(probe).unwrap();
    env.apply_post_load_workarounds();
    env.exec(probe).unwrap();
}

#[test]
fn encoding_util_cbor_serializes_standard_wire_values_and_binary_roundtrips() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local encode, decode = C_EncodingUtil.SerializeCBOR, C_EncodingUtil.DeserializeCBOR
        assert(encode(nil) == string.char(246))
        assert(encode(false) == string.char(244) and encode(true) == string.char(245))
        assert(encode(24) == string.char(24,24))
        assert(encode(-25) == string.char(56,24))
        assert(encode(8589934770) == string.char(27,0,0,0,2,0,0,0,178))
        assert(encode(string.char(255,0)) == string.char(66,255,0))
        assert(encode({}) == string.char(128))
        assert(encode({1,2}) == string.char(130,1,2))
        assert(encode({[0]='z'}) == string.char(161,0,65,122))
        local shared = {1.1, math.huge, -math.huge, string.char(0,255,128)}
        local source = {[1]='a', [3]='b', ['1']='string key', [false]='false key', [-1]='negative',
            left=shared, right=shared, fractional=1.5, zero=-1 / math.huge}
        local encoded = C_EncodingUtil.EncodeBase64(C_EncodingUtil.CompressString(encode(source), 1, 2))
        local copy = decode(C_EncodingUtil.DecompressString(C_EncodingUtil.DecodeBase64(encoded), 1))
        assert(copy[1] == 'a' and copy[2] == nil and copy[3] == 'b')
        assert(copy['1'] == 'string key' and copy[false] == 'false key' and copy[-1] == 'negative')
        assert(copy.left ~= copy.right and copy.left[1] == 1.1 and copy.right[4] == string.char(0,255,128))
        assert(copy.left[2] == math.huge and copy.left[3] == -math.huge and copy.fractional == 1.5)
        assert(1 / copy.zero == -math.huge)
        local nan = 0/0
        assert(decode(encode(nan)) ~= decode(encode(nan)))
        local keyMap = decode(encode({[{1,2}]='table key'}))
        local key, value = next(keyMap)
        assert(type(key) == 'table' and key[1] == 1 and key[2] == 2 and value == 'table key')
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_cbor_preserves_double_integer_boundaries_and_negative_external_integers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local encode, decode = C_EncodingUtil.SerializeCBOR, C_EncodingUtil.DeserializeCBOR
        -- Independent major-type-1 argument 9007199254740993 means
        -- -(argument + 1), which is the exactly representable -9007199254740994.
        assert(decode(string.char(59,0,32,0,0,0,0,0,1)) == -9007199254740994,
            'negative CBOR integer must round only after adding one to its magnitude')
        local two53, two64 = 2^53, 2^64
        for _, value in ipairs({two53-1, two53, two53+2, -(two53-1), -two53, -(two53+2),
            two64-2048, two64, two64+4096, -(two64-2048), -two64, -(two64+4096)}) do
            assert(decode(encode(value)) == value, 'CBOR roundtrip changed a boundary double')
            local keyMap = decode(encode({[value]='boundary'}))
            assert(keyMap[value] == 'boundary', 'CBOR roundtrip changed a numeric map key')
        end
        -- Maximum CBOR integer arguments round to the nearest Lua double.
        assert(decode(string.char(27,255,255,255,255,255,255,255,255)) == two64)
        assert(decode(string.char(59,255,255,255,255,255,255,255,255)) == -two64)
        local negativeZero = -1 / math.huge
        assert(1 / decode(encode(negativeZero)) == -math.huge)
        "#,
    )
    .unwrap();
}

#[test]
fn encoding_util_cbor_serializer_options_reject_cycles_and_preserve_depth_limit() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local encode, decode = C_EncodingUtil.SerializeCBOR, C_EncodingUtil.DeserializeCBOR
        assert(not pcall(encode, function() end))
        assert(encode(function() end, {ignoreSerializationErrors=true}) == string.char(247))
        local copy = decode(encode({1, function() end, 3}, {ignoreSerializationErrors=true}))
        assert(copy[1] == 1 and copy[2] == nil and copy[3] == 3)
        assert(not pcall(encode, {}, false))
        assert(not pcall(encode, {}, {ignoreSerializationErrors='yes'}))
        assert(not pcall(encode, {[function() end]=1}, {ignoreSerializationErrors=true}))
        local cycle = {}; cycle.self = cycle
        assert(not pcall(encode, cycle))
        assert(not pcall(encode, cycle, {ignoreSerializationErrors=true}))
        local root = {}; local current = root
        for index = 2, 100 do current[1] = {}; current = current[1] end
        current[1] = 0
        assert(type(encode(root)) == 'string')
        current[1] = {0}
        assert(not pcall(encode, root))
        assert(not pcall(encode, root, {ignoreSerializationErrors=true}))
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(feature = "client-wowforever")]
fn encoding_util_selected_questiedb_reader_decodes_actual_metadata_and_fresh_rows() {
    use std::path::PathBuf;
    use wow_ui_sim::lua_api::AddonInfo;
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;

    let Some(addon) = std::env::var_os("QUESTIEDB_UI_PATH").map(PathBuf::from) else {
        eprintln!("skipping installed QuestieDB: set QUESTIEDB_UI_PATH to its addon directory");
        return;
    };
    let toc_path = wow_ui_sim::loader::find_toc_file(&addon).unwrap();
    assert_eq!(
        toc_path.file_name().and_then(|name| name.to_str()),
        Some("QuestieDB_Camelot.toc")
    );
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game) && !toc.is_game_type_restricted());
    assert_eq!(
        toc.metadata.get("Interface").map(String::as_str),
        Some("16001")
    );
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addons.push(AddonInfo {
        folder_name: "QuestieDB".into(),
        addon_dir: Some(addon.clone()),
        metadata: toc.metadata.clone(),
        enabled: true,
        ..Default::default()
    });
    env.exec("EncodingQuestieConsumer = {}").unwrap();
    for path in [
        "src/config.lua",
        "src/meta/codec.lua",
        "src/meta/questMeta.lua",
        "src/read/baked.lua",
    ] {
        let index = toc
            .files
            .iter()
            .position(|file| file.to_string_lossy().replace('\\', "/") == path)
            .expect("selected TOC must include the original reader and its schema");
        assert!(toc.file_allows_screen(index, ScreenKind::Game));
        let source = std::fs::read_to_string(addon.join(path)).unwrap();
        let mut separator = String::from("=");
        while source.contains(&format!("]{separator}]")) {
            separator.push('=');
        }
        env.exec(&format!(
            "assert(loadstring([{separator}[{source}]{separator}], '@QuestieDB/{path}'))('QuestieDB', EncodingQuestieConsumer)"
        )).expect("load whole selected QuestieDB source unchanged");
    }
    env.exec(
        r#"
        local lib = EncodingQuestieConsumer
        local backend = lib.read.baked.CreateBackend(lib.Meta.Quest)
        local ids, idMap = backend.getAllIds()
        assert(#ids == 5232 and ids[1] == 2 and ids[#ids] == 99411)
        assert(idMap[2] == true and idMap[99411] == true and idMap[-1] == nil)
        local row = backend.scalarRow(2)
        assert(row[1] == "Sharptalon's Claw" and row[4] == 20 and row[5] == 30)
        assert(row[6] == 8589934770 and row.p == 33575046)
        local tableValue = backend.readField(2, 2)
        assert(tableValue[3][1] == 16305, 'integer-keyed CBOR map must stay numeric')
        local producer = assert(backend.tableProducer(2, 2, row))
        local first, second = producer(), producer()
        assert(first ~= second and first[3] ~= second[3])
        first[3][1] = -1
        assert(second[3][1] == 16305)
        "#,
    )
    .expect("original baked reader must decode real chunked metadata and produce fresh values");
}

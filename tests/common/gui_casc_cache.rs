use std::path::Path;

use cascette_client_storage::{index::IndexManager, storage::local_header::LocalHeader};
use cascette_crypto::md5::{ContentKey, EncodingKey, FileDataId};
use cascette_formats::{
    encoding::{CKeyEntryData, EKeyEntryData, EncodingBuilder},
    root::{ContentFlags, LocaleFlags, RootBuilder, RootVersion},
};

pub const BUILD_KEY: &str = "dcfc90fffd79ba00406ae46f5f657592";
pub const FIRST_FDID: u32 = 910001;
pub const SECOND_FDID: u32 = 910002;

const FIRST_ASSET: &[u8] = b"GUI CASC cache fixture asset one";
const SECOND_ASSET: &[u8] = b"GUI CASC cache fixture asset two";

#[derive(Clone, Copy)]
pub struct Options {
    pub index_encoding_file: bool,
    pub corrupt_encoding_file: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            index_encoding_file: true,
            corrupt_encoding_file: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ExpectedResolution {
    pub content_key: [u8; 16],
    pub encoding_key: [u8; 16],
}

pub fn expected_resolutions() -> [ExpectedResolution; 2] {
    [
        ExpectedResolution {
            content_key: *ContentKey::from_data(FIRST_ASSET).as_bytes(),
            encoding_key: *EncodingKey::from_data(&encode_blte(FIRST_ASSET)).as_bytes(),
        },
        ExpectedResolution {
            content_key: *ContentKey::from_data(SECOND_ASSET).as_bytes(),
            encoding_key: *EncodingKey::from_data(&encode_blte(SECOND_ASSET)).as_bytes(),
        },
    ]
}

pub fn write_install(root: &Path, options: Options) {
    let data_dir = root.join("Data/data");
    std::fs::create_dir_all(&data_dir).expect("create fixture data directory");
    std::fs::write(
        root.join(".product.db"),
        include_bytes!("../fixtures/casc-installed-products.db"),
    )
    .expect("write fixture product database");
    std::fs::write(
        root.join(".build.info"),
        "Branch!STRING:0|Active!DEC:1|Build Key!HEX:16|Version!STRING:0|Product!STRING:0\n",
    )
    .expect("write fixture build-info header");

    let expected = expected_resolutions();
    let first_content_key = ContentKey::from_bytes(expected[0].content_key);
    let second_content_key = ContentKey::from_bytes(expected[1].content_key);
    let first_encoding_key = EncodingKey::from_bytes(expected[0].encoding_key);
    let second_encoding_key = EncodingKey::from_bytes(expected[1].encoding_key);

    let root_data = build_root(first_content_key, second_content_key);
    let root_content_key = ContentKey::from_data(&root_data);
    let root_blte = encode_blte(&root_data);
    let root_encoding_key = EncodingKey::from_data(&root_blte);

    let encoding_data = build_encoding(
        root_content_key,
        root_encoding_key,
        &[
            (first_content_key, first_encoding_key, FIRST_ASSET.len()),
            (second_content_key, second_encoding_key, SECOND_ASSET.len()),
        ],
    );
    let encoding_content_key = ContentKey::from_data(&encoding_data);
    let encoding_blte = if options.corrupt_encoding_file {
        b"not a BLTE file".to_vec()
    } else {
        encode_blte(&encoding_data)
    };
    let encoding_file_key = EncodingKey::from_data(&encoding_blte);

    let config_path = root.join("Data/config/dc/fc").join(BUILD_KEY);
    std::fs::create_dir_all(config_path.parent().expect("config parent"))
        .expect("create fixture config directory");
    std::fs::write(
        config_path,
        format!(
            "root = {}\nencoding = {} {}\n",
            root_content_key, encoding_content_key, encoding_file_key
        ),
    )
    .expect("write fixture build config");

    let mut archive = Vec::new();
    let (root_offset, root_size) =
        append_archive_entry(&mut archive, root_encoding_key, &root_blte);
    let (encoding_offset, encoding_size) =
        append_archive_entry(&mut archive, encoding_file_key, &encoding_blte);
    std::fs::write(data_dir.join("data.000"), archive).expect("write fixture archive");

    let mut indices = IndexManager::new(&data_dir);
    indices
        .add_entry(&root_encoding_key, 0, root_offset, root_size)
        .expect("index root file");
    if options.index_encoding_file {
        indices
            .add_entry(&encoding_file_key, 0, encoding_offset, encoding_size)
            .expect("index encoding file");
    }
    indices.save_all().expect("write fixture index");
}

fn build_root(first_content_key: ContentKey, second_content_key: ContentKey) -> Vec<u8> {
    let mut builder = RootBuilder::new(RootVersion::V4);
    let locale = LocaleFlags::new(LocaleFlags::ENUS);
    let content = ContentFlags::new(ContentFlags::INSTALL | ContentFlags::NO_NAME_HASH);
    builder.add_file(
        FileDataId::new(FIRST_FDID),
        first_content_key,
        None,
        locale,
        content,
    );
    builder.add_file(
        FileDataId::new(SECOND_FDID),
        second_content_key,
        None,
        locale,
        content,
    );
    builder.build().expect("build fixture root file")
}

fn build_encoding(
    root_content_key: ContentKey,
    root_encoding_key: EncodingKey,
    assets: &[(ContentKey, EncodingKey, usize); 2],
) -> Vec<u8> {
    let mut builder = EncodingBuilder::new();
    builder.add_ckey_entry(CKeyEntryData {
        content_key: root_content_key,
        file_size: 0,
        encoding_keys: vec![root_encoding_key],
    });
    builder.add_ekey_entry(EKeyEntryData {
        encoding_key: root_encoding_key,
        espec: "z".to_owned(),
        file_size: 0,
    });
    for (content_key, encoding_key, size) in assets {
        builder.add_ckey_entry(CKeyEntryData {
            content_key: *content_key,
            file_size: u64::try_from(*size).expect("fixture file size fits u64"),
            encoding_keys: vec![*encoding_key],
        });
        builder.add_ekey_entry(EKeyEntryData {
            encoding_key: *encoding_key,
            espec: "z".to_owned(),
            file_size: u64::try_from(*size).expect("fixture file size fits u64"),
        });
    }
    builder
        .build()
        .expect("build fixture encoding tables")
        .build()
        .expect("serialize fixture encoding file")
}

fn append_archive_entry(
    archive: &mut Vec<u8>,
    encoding_key: EncodingKey,
    blte: &[u8],
) -> (u32, u32) {
    let offset = u32::try_from(archive.len()).expect("fixture archive offset fits u32");
    let encoded_size = u32::try_from(blte.len()).expect("fixture BLTE size fits u32");
    archive.extend_from_slice(&LocalHeader::new(*encoding_key.as_bytes(), encoded_size).to_bytes());
    archive.extend_from_slice(blte);
    let size = u32::try_from(archive.len()).expect("fixture archive size fits u32") - offset;
    (offset, size)
}

fn encode_blte(content: &[u8]) -> Vec<u8> {
    let mut blte = Vec::with_capacity(9 + content.len());
    blte.extend_from_slice(b"BLTE");
    blte.extend_from_slice(&0_u32.to_be_bytes());
    blte.push(b'N');
    blte.extend_from_slice(content);
    blte
}

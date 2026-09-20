use super::*;

fn atlas_fixture(csv: &str) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), csv).unwrap();
    file
}

#[test]
fn atlas_csv_reads_reordered_named_columns() {
    let file = atlas_fixture(
        "UiCanvasID,AtlasHeight,ID,UiTextureAtlasSetID,FileDataID,AtlasWidth\n2,1024,3945,7,8026708,512\n",
    );
    let entries = load_atlas(file.path()).unwrap();
    let atlas = &entries[&3945];
    assert_eq!(
        (atlas.file_data_id, atlas.width, atlas.height),
        (8026708, 512, 1024)
    );
    assert_eq!((atlas.set_id, atlas.canvas_id), (7, 2));
}

#[test]
fn atlas_csv_rejects_each_missing_required_header() {
    let columns = [
        "ID",
        "FileDataID",
        "UiTextureAtlasSetID",
        "AtlasWidth",
        "AtlasHeight",
        "UiCanvasID",
    ];
    for missing in columns {
        let header = columns
            .iter()
            .copied()
            .filter(|name| *name != missing)
            .collect::<Vec<_>>()
            .join(",");
        let file = atlas_fixture(&format!("{header}\n"));
        let error = load_atlas(file.path())
            .err()
            .expect("missing header must fail")
            .to_string();
        assert!(error.contains(missing), "{error}");
        assert!(error.contains("missing required column"), "{error}");
    }
}

#[test]
fn atlas_csv_rejects_truncated_rows_without_panicking() {
    for row in [
        "3944,8026705,1",
        "3944,8026705,1,512",
        "3944,8026705,1,512,512",
    ] {
        let file = atlas_fixture(&format!(
            "ID,FileDataID,UiTextureAtlasSetID,AtlasWidth,AtlasHeight,UiCanvasID\n{row}\n"
        ));
        let error = load_atlas(file.path())
            .err()
            .expect("short row must fail")
            .to_string();
        assert!(error.contains("row 2"), "{error}");
        assert!(error.contains("expected 6 columns"), "{error}");
    }
}

#[test]
fn explicit_csv_generation_uses_canvas_geometry_and_local_slices() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    for (name, content) in [
        (
            "UiTextureAtlas.csv",
            "ID,FileDataID,UiTextureAtlasSetID,AtlasWidth,AtlasHeight,UiCanvasID\n3944,8026705,1,512,512,1\n3945,8026708,1,1024,1024,2\n",
        ),
        (
            "UiTextureAtlasElement.csv",
            "Name,ID\nUI-HUD-Minimap-Frame-Cycle,35224\n",
        ),
        (
            "UiTextureAtlasMember.csv",
            "CommittedName,ID,UiTextureAtlasID,Width,Height,CommittedLeft,CommittedRight,CommittedTop,CommittedBottom,UiTextureAtlasElementID,OverrideWidth,OverrideHeight,CommittedFlags,UiCanvasID\ncycle-c60-2x,39257,3945,84,84,509,593,509,593,35224,0,0,0,0\ncycle-c60,39256,3944,42,42,256,298,256,298,35224,0,0,0,0\n",
        ),
        (
            "UiTextureAtlasElementSliceData.csv",
            "ID,UiTextureAtlasElementID,Left,Top,Right,Bottom,SliceMode\n1,35224,3,4,5,6,1\n",
        ),
        (
            "listfile.csv",
            "8026705;interface/minimap/cycle.blp\n8026708;interface/minimap/cycle2.blp\n",
        ),
    ] {
        std::fs::write(dir.join(name), content).unwrap();
    }
    let output = dir.join("atlas.rs");
    let elements_output = dir.join("elements.rs");
    run(Options {
        csv_dir: Some(dir.to_owned()),
        listfile: None,
        output: output.clone(),
        elements_output: elements_output.clone(),
    })
    .unwrap();
    let generated = std::fs::read_to_string(output).unwrap();
    let entry = generated
        .lines()
        .find(|line| line.contains("\"ui-hud-minimap-frame-cycle\" => AtlasInfo"))
        .unwrap();
    assert!(entry.contains("width: 42, height: 42"));
    assert!(entry.contains("left_tex_coord: 0.500000, right_tex_coord: 0.582031"));
    assert!(
        generated.contains(
            "left: 3u32, top: 4u32, right: 5u32, bottom: 6u32, mode: AtlasSliceMode::Tile"
        )
    );
    assert!(
        std::fs::read_to_string(elements_output)
            .unwrap()
            .contains("35224u32 => \"ui-hud-minimap-frame-cycle\"")
    );
}

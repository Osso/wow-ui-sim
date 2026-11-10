//! Issue #52: Lua texture state through the production headless screenshot path.
use super::quad_builders::build_texture_quads;
use crate::lua_api::WowLuaEnv;
use crate::render::{QuadBatch, headless::render_to_image};
use crate::texture::TextureManager;
use iced::{Point, Rectangle, Size};
use image::{Rgba, RgbaImage};

struct Fixture {
    env: WowLuaEnv,
    textures: TextureManager,
    _directory: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let addon = directory.path().join("Issue52");
        std::fs::create_dir(&addon).unwrap();
        RgbaImage::from_pixel(12, 12, Rgba([255; 4]))
            .save(addon.join("white.png"))
            .unwrap();
        RgbaImage::from_pixel(12, 12, Rgba([200, 100, 50, 255]))
            .save(addon.join("colour.png"))
            .unwrap();
        let mut sliced = RgbaImage::from_pixel(12, 12, Rgba([0, 255, 0, 255]));
        for y in 0..3 {
            for x in 0..2 {
                sliced.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        sliced.save(addon.join("sliced.png")).unwrap();
        let env = WowLuaEnv::new().unwrap();
        env.state().borrow_mut().addon_base_paths = vec![directory.path().to_path_buf()];
        Self {
            env,
            textures: TextureManager::new().with_addons_path(directory.path()),
            _directory: directory,
        }
    }

    fn append(&self, batch: &mut QuadBatch, script: &str, bounds: Rectangle) {
        self.env
            .exec("Issue52Texture = UIParent:CreateTexture('Issue52Texture')")
            .unwrap();
        self.env.exec(script).unwrap();
        let frame = {
            let state = self.env.state().borrow();
            let id = state.widgets.get_id_by_name("Issue52Texture").unwrap();
            state.widgets.get(id).unwrap().clone()
        };
        build_texture_quads(batch, bounds, &frame, None, 1.0);
    }

    fn image(&mut self, batch: &QuadBatch, width: u32, height: u32) -> RgbaImage {
        let image = render_to_image(batch, &mut self.textures, width, height, None);
        let path = self._directory.path().join("screenshot.png");
        image.save(&path).unwrap();
        image::open(path).unwrap().into_rgba8()
    }
}

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(x, y), Size::new(width, height))
}

fn assert_pixel(image: &RgbaImage, x: u32, y: u32, expected: [u8; 4], tolerance: u8) {
    let actual = image.get_pixel(x, y).0;
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(a, e)| a.abs_diff(e) <= tolerance),
        "pixel ({x},{y}): actual={actual:?}, expected={expected:?} +/- {tolerance}"
    );
}

#[test]
fn issue52_headless_colour_bytes_match_lua_inputs_and_image_tints() {
    let mut fixture = Fixture::new();
    let mut batch = QuadBatch::new();
    let cases = [
        (
            "Issue52Texture:SetColorTexture(0.5,0.5,0.5,1)",
            [128, 128, 128, 255],
        ),
        (
            "Issue52Texture:SetColorTexture(0.235,0.14,0.08,1)",
            [60, 36, 20, 255],
        ),
        (
            "Issue52Texture:SetColorTexture(1,1,1,1); Issue52Texture:SetVertexColor(0.235,0.14,0.08,1)",
            [60, 36, 20, 255],
        ),
        (
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/white.png'); Issue52Texture:SetVertexColor(0.5,0.5,0.5,1)",
            [128, 128, 128, 255],
        ),
        (
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/white.png'); Issue52Texture:SetVertexColor(0.235,0.14,0.08,1)",
            [60, 36, 20, 255],
        ),
        (
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/colour.png')",
            [200, 100, 50, 255],
        ),
        (
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/colour.png'); Issue52Texture:SetVertexColor(0.5,0.5,0.5,1)",
            [100, 50, 25, 255],
        ),
    ];
    for (index, (script, _)) in cases.iter().enumerate() {
        fixture.append(
            &mut batch,
            script,
            bounds(index as f32 * 16.0, 0.0, 16.0, 16.0),
        );
    }
    let image = fixture.image(&batch, 112, 16);
    for (index, (_, expected)) in cases.iter().enumerate() {
        assert_pixel(&image, index as u32 * 16 + 8, 8, *expected, 0);
    }
}

#[test]
fn issue52_headless_image_gradient_multiplies_texture_and_vertex_colour() {
    let mut fixture = Fixture::new();
    let mut batch = QuadBatch::new();
    for (index, orientation) in ["HORIZONTAL", "VERTICAL"].into_iter().enumerate() {
        fixture.append(&mut batch, &format!(
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/colour.png'); Issue52Texture:SetVertexColor(0.5,1,1,1); Issue52Texture:SetGradient('{orientation}', CreateColor(0,0,0,1), CreateColor(1,1,1,1))"),
            bounds(index as f32 * 32.0, 0.0, 32.0, 32.0));
    }
    let image = fixture.image(&batch, 64, 32);
    for (x, y, fraction) in [
        (7, 16, 7.5 / 32.0),
        (23, 16, 23.5 / 32.0),
        (48, 7, 1.0 - 7.5 / 32.0),
        (48, 23, 1.0 - 23.5 / 32.0),
    ] {
        let expected = [100.0 * fraction, 100.0 * fraction, 50.0 * fraction];
        assert_pixel(
            &image,
            x,
            y,
            [
                expected[0].round() as u8,
                expected[1].round() as u8,
                expected[2].round() as u8,
                255,
            ],
            1,
        );
    }
}

#[test]
fn issue52_texture_slice_corners_keep_pixel_size_as_frame_grows() {
    let mut fixture = Fixture::new();
    let mut batch = QuadBatch::new();
    for (x, size) in [(0.0, 24.0), (32.0, 48.0)] {
        fixture.append(&mut batch,
            "Issue52Texture:SetTexture('Interface/AddOns/Issue52/sliced.png'); Issue52Texture:SetTextureSliceMargins(2,3,4,1)",
            bounds(x,0.0,size,size));
    }
    let image = fixture.image(&batch, 80, 48);
    for x in [0, 32] {
        assert_pixel(&image, x, 1, [255, 0, 0, 255], 0);
        assert_pixel(&image, x + 1, 2, [255, 0, 0, 255], 0);
        assert_pixel(&image, x + 2, 1, [0, 255, 0, 255], 0);
        assert_pixel(&image, x + 1, 3, [0, 255, 0, 255], 0);
    }
    // Geometry is part of the renderer contract: asymmetric documented L,T,R,B margins.
    for origin in [0.0, 32.0] {
        let corner = batch
            .vertices
            .chunks_exact(4)
            .find(|q| q[0].position == [origin, 0.0])
            .unwrap();
        assert_eq!(corner[2].position, [origin + 2.0, 3.0]);
    }
}

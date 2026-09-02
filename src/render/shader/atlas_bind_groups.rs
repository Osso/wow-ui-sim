use super::atlas::NUM_TIERS;

pub(super) fn create_texture_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("WoW UI Texture Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

fn create_glyph_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("WoW UI Glyph Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

pub(super) fn create_glyph_atlas(
    device: &wgpu::Device,
    size: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Glyph Atlas"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn create_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("WoW UI Texture Bind Group Layout"),
        entries: &[
            texture_bind_group_layout_entry(0),
            texture_bind_group_layout_entry(1),
            texture_bind_group_layout_entry(2),
            texture_bind_group_layout_entry(3),
            texture_bind_group_layout_entry(4),
            sampler_bind_group_layout_entry(5),
            texture_bind_group_layout_entry(6),
            texture_bind_group_layout_entry(7),
            texture_bind_group_layout_entry(8),
            sampler_bind_group_layout_entry(9),
        ],
    })
}

fn texture_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn atlas_bind_group_views<'a>(
    tier_views: [&'a wgpu::TextureView; NUM_TIERS],
    glyph_view: &'a wgpu::TextureView,
    bc1_view: &'a wgpu::TextureView,
    bc3_view: &'a wgpu::TextureView,
) -> [&'a wgpu::TextureView; 8] {
    [
        tier_views[0],
        tier_views[1],
        tier_views[2],
        tier_views[3],
        tier_views[4],
        glyph_view,
        bc1_view,
        bc3_view,
    ]
}

fn texture_bind_group_entry<'a>(
    binding: u32,
    view: &'a wgpu::TextureView,
) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

fn sampler_bind_group_entry<'a>(sampler: &'a wgpu::Sampler) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding: 5,
        resource: wgpu::BindingResource::Sampler(sampler),
    }
}

fn glyph_sampler_bind_group_entry<'a>(sampler: &'a wgpu::Sampler) -> wgpu::BindGroupEntry<'a> {
    wgpu::BindGroupEntry {
        binding: 9,
        resource: wgpu::BindingResource::Sampler(sampler),
    }
}

fn create_atlas_bind_group_entries<'a>(
    views: [&'a wgpu::TextureView; 8],
    texture_sampler: &'a wgpu::Sampler,
    glyph_sampler: &'a wgpu::Sampler,
) -> [wgpu::BindGroupEntry<'a>; 10] {
    [
        texture_bind_group_entry(0, views[0]),
        texture_bind_group_entry(1, views[1]),
        texture_bind_group_entry(2, views[2]),
        texture_bind_group_entry(3, views[3]),
        texture_bind_group_entry(4, views[4]),
        sampler_bind_group_entry(texture_sampler),
        texture_bind_group_entry(6, views[5]),
        texture_bind_group_entry(7, views[6]),
        texture_bind_group_entry(8, views[7]),
        glyph_sampler_bind_group_entry(glyph_sampler),
    ]
}

pub(super) fn create_atlas_bind_groups(
    device: &wgpu::Device,
    tier_views: [&wgpu::TextureView; NUM_TIERS],
    glyph_view: &wgpu::TextureView,
    bc1_view: &wgpu::TextureView,
    bc3_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let layout = create_atlas_bind_group_layout(device);
    let views = atlas_bind_group_views(tier_views, glyph_view, bc1_view, bc3_view);
    let glyph_sampler = create_glyph_sampler(device);
    let entries = create_atlas_bind_group_entries(views, sampler, &glyph_sampler);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("WoW UI Texture Bind Group"),
        layout: &layout,
        entries: &entries,
    });
    (layout, bind_group)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("test adapter should exist");
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("test device should be created")
    }

    fn create_test_view(device: &wgpu::Device) -> wgpu::TextureView {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("test texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    #[test]
    fn create_atlas_bind_group_entries_keeps_sampler_at_binding_five() {
        let (device, _queue) = create_test_device();
        let tier_views: [wgpu::TextureView; NUM_TIERS] =
            std::array::from_fn(|_| create_test_view(&device));
        let glyph_view = create_test_view(&device);
        let bc1_view = create_test_view(&device);
        let bc3_view = create_test_view(&device);
        let texture_sampler = create_texture_sampler(&device);
        let glyph_sampler = create_glyph_sampler(&device);
        let views = atlas_bind_group_views(
            [
                &tier_views[0],
                &tier_views[1],
                &tier_views[2],
                &tier_views[3],
                &tier_views[4],
            ],
            &glyph_view,
            &bc1_view,
            &bc3_view,
        );

        let entries = create_atlas_bind_group_entries(views, &texture_sampler, &glyph_sampler);

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.binding)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert!(matches!(
            entries[5].resource,
            wgpu::BindingResource::Sampler(_)
        ));
        assert!(matches!(
            entries[9].resource,
            wgpu::BindingResource::Sampler(_)
        ));
        assert!(matches!(
            entries[0].resource,
            wgpu::BindingResource::TextureView(_)
        ));
        assert!(matches!(
            entries[8].resource,
            wgpu::BindingResource::TextureView(_)
        ));
    }
}

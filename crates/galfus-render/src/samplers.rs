pub struct SamplerSet {
    pub point_clamp: wgpu::Sampler,
    pub linear_clamp: wgpu::Sampler,
    pub point_repeat: wgpu::Sampler,
    pub linear_repeat: wgpu::Sampler,
    pub comparison: wgpu::Sampler,
}

pub fn create_standard_samplers(device: &wgpu::Device) -> SamplerSet {
    SamplerSet {
        point_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Point Clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }),
        linear_clamp: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear Clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }),
        point_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Point Repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }),
        linear_repeat: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear Repeat"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }),
        comparison: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Comparison"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::GreaterEqual),
            ..Default::default()
        }),
    }
}

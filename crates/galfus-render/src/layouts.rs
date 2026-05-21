pub struct Layouts {
    pub shared: wgpu::BindGroupLayout,
    pub object: wgpu::BindGroupLayout,
    pub object_standard: wgpu::BindGroupLayout,
    pub object_pbr: wgpu::BindGroupLayout,
    pub frame_semantics: wgpu::BindGroupLayout,
    pub target: wgpu::BindGroupLayout,
    pub light_cull: wgpu::BindGroupLayout,
    pub ssao: wgpu::BindGroupLayout,
    pub ssao_blur: wgpu::BindGroupLayout,
    pub ssao_msaa: wgpu::BindGroupLayout,
    pub ssao_blur_msaa: wgpu::BindGroupLayout,
    pub bloom: wgpu::BindGroupLayout,
    pub skybox: wgpu::BindGroupLayout,
}

pub struct PipelineLayouts {
    pub gizmo: wgpu::PipelineLayout,
    pub forward_standard: wgpu::PipelineLayout,
    pub forward_pbr: wgpu::PipelineLayout,
    pub shadow: wgpu::PipelineLayout,
    pub outline: wgpu::PipelineLayout,
    pub light_cull: wgpu::PipelineLayout,
    pub ssao: wgpu::PipelineLayout,
    pub ssao_blur: wgpu::PipelineLayout,
    pub ssao_msaa: wgpu::PipelineLayout,
    pub ssao_blur_msaa: wgpu::PipelineLayout,
    pub bloom: wgpu::PipelineLayout,
    pub skybox: wgpu::PipelineLayout,
}

pub struct EffectBuffers {
    pub post: wgpu::Buffer,
    pub compose: wgpu::Buffer,
    pub ssao: wgpu::Buffer,
    pub ssao_blur: wgpu::Buffer,
    pub bloom: wgpu::Buffer,
    pub skybox: wgpu::Buffer,
}

pub fn create_pipeline_layouts(device: &wgpu::Device, layouts: &Layouts) -> PipelineLayouts {
    PipelineLayouts {
        gizmo: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gizmo Pipeline Layout"),
            bind_group_layouts: &[&layouts.shared],
            immediate_size: 0,
        }),
        forward_standard: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Forward Standard Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.shared,
                &layouts.object_standard,
                &layouts.frame_semantics,
            ],
            immediate_size: 0,
        }),
        forward_pbr: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Forward PBR Pipeline Layout"),
            bind_group_layouts: &[
                &layouts.shared,
                &layouts.object_pbr,
                &layouts.frame_semantics,
            ],
            immediate_size: 0,
        }),
        shadow: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[&layouts.shared, &layouts.object],
            immediate_size: 0,
        }),
        outline: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Outline Pipeline Layout"),
            bind_group_layouts: &[&layouts.shared, &layouts.object],
            immediate_size: 0,
        }),
        light_cull: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LightCull Pipeline Layout"),
            bind_group_layouts: &[&layouts.light_cull],
            immediate_size: 0,
        }),
        ssao: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[&layouts.ssao],
            immediate_size: 0,
        }),
        ssao_blur: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Blur Pipeline Layout"),
            bind_group_layouts: &[&layouts.ssao_blur],
            immediate_size: 0,
        }),
        ssao_msaa: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO MSAA Pipeline Layout"),
            bind_group_layouts: &[&layouts.ssao_msaa],
            immediate_size: 0,
        }),
        ssao_blur_msaa: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Blur MSAA Pipeline Layout"),
            bind_group_layouts: &[&layouts.ssao_blur_msaa],
            immediate_size: 0,
        }),
        bloom: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Pipeline Layout"),
            bind_group_layouts: &[&layouts.bloom],
            immediate_size: 0,
        }),
        skybox: device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[&layouts.skybox],
            immediate_size: 0,
        }),
    }
}

pub fn create_effect_buffers(device: &wgpu::Device, skybox_uniform_size: u64) -> EffectBuffers {
    EffectBuffers {
        post: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PostProcess Uniform Buffer"),
            size: 96,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        compose: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compose Cover Uniform Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        ssao: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Uniform Buffer"),
            size: 160,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        ssao_blur: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SSAO Blur Uniform Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bloom: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bloom Storage Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        skybox: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Skybox Uniform Buffer"),
            size: skybox_uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
    }
}

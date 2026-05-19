pub struct ShaderModules {
    pub post: wgpu::ShaderModule,
    pub compose: wgpu::ShaderModule,
    pub outline: wgpu::ShaderModule,
    pub ssao: wgpu::ShaderModule,
    pub ssao_blur: wgpu::ShaderModule,
    pub ssao_msaa: wgpu::ShaderModule,
    pub ssao_blur_msaa: wgpu::ShaderModule,
    pub bloom: wgpu::ShaderModule,
    pub skybox: wgpu::ShaderModule,
    pub light_cull: wgpu::ShaderModule,
    pub shadow: wgpu::ShaderModule,
    pub gizmo: wgpu::ShaderModule,
}

pub fn create_shader_modules(device: &wgpu::Device) -> ShaderModules {
    ShaderModules {
        post: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/post/post.wgsl"
        )),
        compose: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/compose/compose.wgsl"
        )),
        outline: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/outline/outline.wgsl"
        )),
        ssao: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/ssao/ssao.wgsl"
        )),
        ssao_blur: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/ssao/ssao_blur.wgsl"
        )),
        ssao_msaa: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/ssao/ssao_msaa.wgsl"
        )),
        ssao_blur_msaa: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/ssao/ssao_blur_msaa.wgsl"
        )),
        bloom: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/bloom/bloom.wgsl"
        )),
        skybox: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/skybox/skybox.wgsl"
        )),
        light_cull: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/light_cull/light_cull.wgsl"
        )),
        shadow: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/passes/shadow/shadow.wgsl"
        )),
        gizmo: device.create_shader_module(wgpu::include_wgsl!(
            "../../vulfram-runtime/src/core/render/gizmos/gizmo.wgsl"
        )),
    }
}

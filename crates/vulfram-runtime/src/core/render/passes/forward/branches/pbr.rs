use crate::core::render::cache::{PipelineKey, RenderCache};
use crate::core::render::state::ResourceLibrary;
use crate::core::resources::{
    PolygonMode, PrimitiveTopology, RenderSide, SurfaceType, VertexStream,
};

pub fn get_pipeline<'a>(
    cache: &'a mut RenderCache,
    frame_index: u64,
    device: &wgpu::Device,
    library: &ResourceLibrary,
    surface: SurfaceType,
    topology: PrimitiveTopology,
    polygon_mode: PolygonMode,
    render_side: RenderSide,
    sample_count: u32,
    shader_id: u64,
    shader_module: &wgpu::ShaderModule,
) -> &'a wgpu::RenderPipeline {
    let (blend, depth_write, depth_compare) = match surface {
        SurfaceType::Transparent => (
            Some(wgpu::BlendState::ALPHA_BLENDING),
            false,
            wgpu::CompareFunction::GreaterEqual,
        ),
        _ => (
            None,
            true,
            wgpu::CompareFunction::Greater,
        ),
    };

    let cull_mode = match render_side {
        RenderSide::Front => Some(wgpu::Face::Front),
        RenderSide::Back => Some(wgpu::Face::Back),
        RenderSide::DoubleSide => None,
    };

    let wgpu_topology = match topology {
        PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
        PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        PrimitiveTopology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
    };

    let wgpu_polygon_mode = match polygon_mode {
        PolygonMode::Fill => wgpu::PolygonMode::Fill,
        PolygonMode::Line => wgpu::PolygonMode::Line,
        PolygonMode::Point => wgpu::PolygonMode::Point,
    };

    let key = PipelineKey {
        shader_id,
        color_format: wgpu::TextureFormat::Rgba16Float,
        color_target_count: 2,
        depth_format: Some(wgpu::TextureFormat::Depth32Float), // Reverse Z
        sample_count,
        topology: wgpu_topology,
        polygon_mode: wgpu_polygon_mode,
        cull_mode,
        front_face: wgpu::FrontFace::Ccw,
        depth_write_enabled: depth_write,
        depth_compare,
        blend,
    };

    cache.get_or_create(key, frame_index, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Forward PBR Pipeline"),
            layout: Some(&library.forward_pbr_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: Some("vs_main"),
                buffers: &[
                    // 0: Position
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Position.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    },
                    // 1: Normal
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Normal.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 1,
                        }],
                    },
                    // 2: Tangent
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Tangent.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 2,
                        }],
                    },
                    // 3: Color0
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Color0.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 3,
                        }],
                    },
                    // 4: UV0
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::UV0.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 4,
                        }],
                    },
                    // 5: UV1
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::UV1.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 5,
                        }],
                    },
                    // 6: Joints
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Joints.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint16x4,
                            offset: 0,
                            shader_location: 6,
                        }],
                    },
                    // 7: Weights
                    wgpu::VertexBufferLayout {
                        array_stride: VertexStream::Weights.stride_bytes(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 7,
                        }],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: key.color_format,
                        blend: key.blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: key.blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: key.topology,
                strip_index_format: None,
                front_face: key.front_face,
                cull_mode: key.cull_mode,
                unclipped_depth: false,
                polygon_mode: key.polygon_mode,
                conservative: false,
            },
            depth_stencil: key.depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: key.depth_write_enabled,
                depth_compare: key.depth_compare,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: key.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    })
}

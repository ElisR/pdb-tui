//! Rasterizer for converting compute shader characters

use crate::gpu::state_windowless::ValidGridSize;
use wgpu::TextureView;

// TODO Put the functionality for loading up the shader and grid size into here

#[derive(Debug)]
pub struct GPURasterizer {
    pub grid_size: ValidGridSize,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_pipeline_layout: wgpu::PipelineLayout,
    pub compute_bind_group: wgpu::BindGroup,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
}

impl GPURasterizer {
    const INPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Uint;

    pub fn new(
        grid_size: ValidGridSize,
        device: &wgpu::Device,
        input_view: &TextureView,
        output_view: &TextureView,
    ) -> Self {
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: Self::INPUT_FORMAT,
                            access: wgpu::StorageTextureAccess::ReadOnly,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            format: Self::OUTPUT_FORMAT,
                            access: wgpu::StorageTextureAccess::WriteOnly,
                        },
                        count: None,
                    },
                ],
                label: Some("Compute Bind Group Layout"),
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline Descriptor"),
            layout: Some(&compute_pipeline_layout),
            module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Compute Shader Source"),
                source: wgpu::ShaderSource::Wgsl(
                    format!(
                        "const grid_width: u32 = {}u;\nconst grid_height: u32 = {}u;\n{}",
                        grid_size.width(),
                        grid_size.height(),
                        include_str!("basic_ascii.wgsl")
                    )
                    .into(),
                    // include_str!("trivial_compute.wgsl").into(),
                ),
            }),
            entry_point: "rasterize",
        });

        Self {
            grid_size,
            compute_pipeline,
            compute_pipeline_layout,
            compute_bind_group,
            compute_bind_group_layout,
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        input_view: &TextureView,
        output_view: &TextureView,
    ) {
        self.compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bing Group"),
            layout: &self.compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(output_view),
                },
            ],
        });
    }

    fn input_format(&self) -> wgpu::TextureFormat {
        Self::INPUT_FORMAT
    }
    fn output_format(&self) -> wgpu::TextureFormat {
        Self::OUTPUT_FORMAT
    }
}

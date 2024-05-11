//! Fancy rasterizer for converting compute shader characters

use wgpu::util::DeviceExt;
use wgpu::TextureView;
use winit::dpi::PhysicalSize;

use crate::ascii::glyph_render::{get_font, AsciiMatrices, NUM_ASCII_MATRICES};
use crate::gpu::state_windowless::ValidGridSize;

#[derive(Debug)]
pub struct FancyGPURasterizer<const W: usize, const H: usize> {
    // TODO Find a solution for validating the relationship between generic parameters again
    pub grid_size: ValidGridSize,
    // TODO Consider storing output size
    pub compute_pipeline_layout: wgpu::PipelineLayout,
    pub compute_ssim_pipeline: wgpu::ComputePipeline,
    pub compute_ascii_pipeline: wgpu::ComputePipeline,

    // Input and output textures
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    // Pre-rendered ASCII glyphs
    pub ascii_matrices: AsciiMatrices<W, H>,
    pub ascii_bind_group: wgpu::BindGroup,
    pub ascii_matrix_buffer: wgpu::Buffer,
    pub ascii_stats_buffer: wgpu::Buffer,

    // Internal SSIM values
    pub ssim_bind_group: wgpu::BindGroup,
    pub ssim_bind_group_layout: wgpu::BindGroupLayout,
    pub ssim_texture: wgpu::Texture,
    pub ssim_view: wgpu::TextureView,
}

impl<const W: usize, const H: usize> FancyGPURasterizer<W, H> {
    const INPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const SSIM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Uint;

    pub fn new(
        grid_size: ValidGridSize,
        output_size: PhysicalSize<u32>,
        device: &wgpu::Device,
        input_view: &TextureView,
        output_view: &TextureView,
    ) -> Self {
        // Textures
        let texture_bind_group_layout =
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
                label: Some("Texture Bind Group Layout"),
            });
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
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

        // Intermediate SSIM storage
        let ssim_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("SSIM Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None, // We do not need a count because we are not using an array of textures, just a 3D texture
                }],
            });
        let ssim_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: output_size.width,
                height: output_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: Self::SSIM_FORMAT,
            view_formats: &[], // NOTE This may be incorrect and needs to be checked
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            label: Some("SSIM Texture"),
        });
        let ssim_view = ssim_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let ssim_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSIM Bind Group"),
            layout: &ssim_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ssim_view),
            }],
        });

        // ASCII information derived from font
        let font = get_font();
        let ascii_matrices = AsciiMatrices::<W, H>::new(&font);
        // FIXME Problem because float32 is not big enough
        let ascii_matrix_raw = ascii_matrices.padded_matrix_list();
        let ascii_stats = ascii_matrices.matrix_stats();
        let ascii_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ASCII Matrix Buffer"),
            contents: bytemuck::cast_slice(&[ascii_matrix_raw]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let ascii_stats_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ASCII Means Buffer"),
            contents: bytemuck::cast_slice(&[ascii_stats]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let ascii_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None, // TODO Check if this should be None Some(97)
                    },
                ],
                label: Some("ASCII Bind Group Layout"),
            });
        let ascii_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ASCII Bind Group"),
            layout: &ascii_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ascii_matrix_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ascii_stats_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &ssim_bind_group_layout,
                    &ascii_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let compute_ssim_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute SSIM Pipeline Descriptor"),
                layout: Some(&compute_pipeline_layout),
                module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Compute SSIM Shader Source"),
                    source: wgpu::ShaderSource::Wgsl(
                        format!(
                            "const grid_width: u32 = {}u;\nconst grid_height: u32 = {}u;\n{}",
                            grid_size.width(),
                            grid_size.height(),
                            include_str!("compute_ssim.wgsl")
                        )
                        .into(),
                    ),
                }),
                entry_point: "compute_ssim",
            });
        let compute_ascii_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute ASCII Pipeline Descriptor"),
                layout: Some(&compute_pipeline_layout),
                module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Compute ASCII Shader Source"),
                    source: wgpu::ShaderSource::Wgsl(
                        format!(
                            "const grid_width: u32 = {}u;\nconst grid_height: u32 = {}u;\n{}",
                            grid_size.width(),
                            grid_size.height(),
                            include_str!("ssim_ascii.wgsl")
                        )
                        .into(),
                    ),
                }),
                entry_point: "ascii_from_ssim",
            });

        Self {
            grid_size,
            compute_ssim_pipeline,
            compute_ascii_pipeline,
            compute_pipeline_layout,
            texture_bind_group,
            texture_bind_group_layout,
            ssim_bind_group,
            ssim_bind_group_layout,
            ssim_texture,
            ssim_view,
            ascii_matrices,
            ascii_matrix_buffer,
            ascii_stats_buffer,
            ascii_bind_group,
        }
    }

    pub fn resize(
        &mut self,
        output_size: PhysicalSize<u32>,
        device: &wgpu::Device,
        input_view: &TextureView,
        output_view: &TextureView,
    ) {
        self.texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &self.texture_bind_group_layout,
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

        self.ssim_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("SSIM Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None, // We do not need a count because we are not using an array of textures, just a 3D texture
                }],
            });
        self.ssim_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: output_size.width,
                height: output_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::OUTPUT_FORMAT,
            view_formats: &[], // NOTE This may be incorrect and needs to be checked
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            label: Some("SSIM Texture"),
        });
        self.ssim_view = self
            .ssim_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.ssim_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("SSIM Bind Group"),
            layout: &self.ssim_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.ssim_view),
            }],
        });
    }

    pub fn run_compute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        output_size: PhysicalSize<u32>,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.ssim_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.ascii_bind_group, &[]);

        compute_pass.set_pipeline(&self.compute_ssim_pipeline);
        compute_pass.dispatch_workgroups(
            output_size.width,
            output_size.height,
            NUM_ASCII_MATRICES as u32,
        );

        compute_pass.set_pipeline(&self.compute_ascii_pipeline);
        compute_pass.dispatch_workgroups(output_size.width, output_size.height, 1);
    }

    fn input_format(&self) -> wgpu::TextureFormat {
        Self::INPUT_FORMAT
    }
    fn output_format(&self) -> wgpu::TextureFormat {
        Self::OUTPUT_FORMAT
    }
}

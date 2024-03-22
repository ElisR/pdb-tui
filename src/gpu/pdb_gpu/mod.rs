//! Adapted Tutorial 10

use std::iter;

use crossterm::event::Event;
use image::{ImageBuffer, Rgba};
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event::*, window::Window};

pub mod camera;
pub mod input;
pub mod instance;
pub mod model;
pub mod resources;
pub mod run_tui;
pub mod run_windowed;
pub mod texture;

use camera::{Camera, CameraController, CameraUniform};
use instance::{Instance, InstanceRaw, LightUniform};
use model::{DrawLight, DrawModel, Vertex};

use crate::gpu::pdb_gpu::input::UnifiedEvent;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const NUM_INSTANCES_PER_ROW: u32 = 1;

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
    })
}

pub trait InnerState {
    fn size(&self) -> winit::dpi::PhysicalSize<u32>;
    fn format(&self) -> wgpu::TextureFormat;
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device);
}

#[derive(Debug)]
struct WindowedState {
    window: Window,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}

impl WindowedState {
    pub fn new(
        window: Window,
        surface: wgpu::Surface,
        size: winit::dpi::PhysicalSize<u32>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> Self {
        let surface_caps = surface.get_capabilities(adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(device, &config);
        Self {
            window,
            surface,
            config,
            size,
        }
    }
}

impl InnerState for WindowedState {
    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
    fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(device, &self.config);
    }
}

#[derive(Debug)]
struct State<IS: InnerState> {
    inner_state: IS,
    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    obj_model: model::Model,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    instances: Vec<Instance>,
    #[allow(dead_code)]
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
}

impl<IS: InnerState> State<IS> {
    pub async fn new_from_inner_state(
        inner_state: IS,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        let camera = Camera {
            eye: nalgebra::Point3::new(50.0, 5.0, -10.0),
            target: nalgebra::Point3::origin(),
            up: nalgebra::Vector3::y(),
            aspect: inner_state.size().width as f32 / inner_state.size().height as f32,
            fovy: std::f32::consts::FRAC_PI_4,
            znear: 0.1,
            zfar: 1000.0,
        };

        let camera_controller = CameraController::new(2.0);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = nalgebra::Vector3::new(x, 0.0, z);

                    let rotation = if position == nalgebra::Vector3::zeros() {
                        nalgebra::Rotation3::from_axis_angle(&nalgebra::Vector3::z_axis(), 0.0)
                    } else {
                        nalgebra::Rotation3::from_axis_angle(
                            &nalgebra::Unit::new_normalize(position),
                            std::f32::consts::FRAC_PI_4,
                        )
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let obj_model = resources::load_model("rbd.obj", &device, &queue)
            .await
            .unwrap();

        let light_uniform = LightUniform {
            position: [20.0, 20.0, 20.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            inner_state.size().width,
            inner_state.size().height,
            "depth_texture",
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                inner_state.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc(), InstanceRaw::desc()],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                inner_state.format(),
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::desc()],
                shader,
            )
        };

        Self {
            device,
            queue,
            inner_state,
            render_pipeline,
            obj_model,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            instances,
            instance_buffer,
            depth_texture,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
        }
    }
    /// Resize the canvas
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.inner_state.resize(new_size, &self.device);
            self.camera.aspect =
                self.inner_state.size().width as f32 / self.inner_state.size().height as f32;
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                self.inner_state.size().width,
                self.inner_state.size().height,
                "depth_texture",
            );
        }
    }
    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Update the light
        if false {
            let old_position: nalgebra::Point3<_> = self.light_uniform.position.into();
            self.light_uniform.position = (nalgebra::Rotation3::from_axis_angle(
                &nalgebra::Vector3::y_axis(),
                std::f32::consts::PI / 180.0,
            ) * old_position)
                .into();
            self.queue.write_buffer(
                &self.light_buffer,
                0,
                bytemuck::cast_slice(&[self.light_uniform]),
            );
        }
    }
    // TODO Consider moving this function outside of `State`, like the function for creating a render pipeline
    /// Create the devices needed for cases with or without a window
    async fn create_adapter_device_queue(
        surface_option: Option<&wgpu::Surface>,
        instance: &wgpu::Instance,
    ) -> (wgpu::Adapter, wgpu::Device, wgpu::Queue) {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: surface_option,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        (adapter, device, queue)
    }
    fn input(&mut self, event: UnifiedEvent) -> bool {
        self.camera_controller.process_events(event)
    }
}

impl State<WindowedState> {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // TODO In later verison of `wgpu` this is annotated with lifetime and no longer needs to be unsafe
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let (adapter, device, queue) =
            Self::create_adapter_device_queue(Some(&surface), &instance).await;
        let inner_state = WindowedState::new(window, surface, size, &adapter, &device);
        Self::new_from_inner_state(inner_state, device, queue).await
    }
    pub fn window(&self) -> &Window {
        &self.inner_state.window
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.inner_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Debug)]
struct WindowlessState {
    size: winit::dpi::PhysicalSize<u32>,
    output_buffer: wgpu::Buffer,
    output_image: Vec<u8>,
    texture: wgpu::Texture,
}

impl WindowlessState {
    const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    // Take a number of bytes and return the next closest multiple of 256
    pub fn pad_bytes_to_256(bytes: u32) -> u32 {
        (bytes + 255) & !255
    }

    // Pad width to 64 since each pixel requires 4 bytes
    pub fn pad_width_to_64(width: u32) -> u32 {
        (width + 63) & !63
    }

    pub fn new(size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) -> Self {
        // TODO Need to add functionality for changing this
        let output_buffer_size = (Self::U32_SIZE * Self::pad_width_to_64(size.width) * size.height)
            as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Windowless Output Buffer"),
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            view_formats: &[], // NOTE This may be incorrect and needs to be checked
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Windowless Output Texture"),
        };
        let texture = device.create_texture(&texture_desc);

        // Multiply by 4 because RGBA
        let output_image_size = size.width as usize * size.height as usize * 4;
        let output_image = Vec::<u8>::with_capacity(output_image_size);
        Self {
            size,
            output_buffer,
            output_image,
            texture,
        }
    }
}

impl InnerState for WindowlessState {
    fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        self.size = new_size;

        self.output_buffer.destroy();
        self.texture.destroy();

        // TODO Find a solution without repeating so much code
        let output_buffer_size = (Self::U32_SIZE
            * Self::pad_width_to_64(self.size.width)
            * self.size.height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Windowless Output Buffer"),
            mapped_at_creation: false,
        };
        self.output_buffer = device.create_buffer(&output_buffer_desc);

        // TODO Also recreate the texture
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            view_formats: &[], // NOTE This may be incorrect and needs to be checked
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Windowless Output Texture"),
        };
        self.texture = device.create_texture(&texture_desc);

        // TODO Work out logic for new offset
    }
}

impl State<WindowlessState> {
    async fn new(size: PhysicalSize<u32>) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let (_adapter, device, queue) = Self::create_adapter_device_queue(None, &instance).await;
        let inner_state = WindowlessState::new(size, &device);
        Self::new_from_inner_state(inner_state, device, queue).await
    }
    // TODO Need to change this error
    // TODO Need to refactor more out of this function
    async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture_view = self
            .inner_state
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.obj_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.inner_state.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.inner_state.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    // Check that this isn't mean to be 4 `u8`s rather than 1 `u32`
                    bytes_per_row: Some({
                        let bytes = WindowlessState::U32_SIZE * self.inner_state.size().width;
                        WindowlessState::pad_bytes_to_256(bytes)
                    }),
                    rows_per_image: Some(self.inner_state.size().height),
                },
            },
            // TODO Stop redefining the same size
            wgpu::Extent3d {
                width: self.inner_state.size().width,
                height: self.inner_state.size().height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(iter::once(encoder.finish()));

        let buffer_slice = self.inner_state.output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv_async().await.unwrap().unwrap();

        {
            let data = buffer_slice.get_mapped_range();
            self.inner_state.output_image.clear();
            self.inner_state.output_image.extend_from_slice(&data[..]);
        }

        self.inner_state.output_buffer.unmap();

        self.inner_state.output_image = self
            .inner_state
            .output_image
            .chunks(
                WindowlessState::U32_SIZE as usize
                    * WindowlessState::pad_width_to_64(self.inner_state.size().width) as usize,
            )
            .flat_map(|row| {
                row.iter().take(
                    WindowlessState::U32_SIZE as usize * self.inner_state.size().width as usize,
                )
            })
            .cloned()
            .collect();

        // let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
        //     self.inner_state.size().width,
        //     self.inner_state.size().height,
        //     &self.inner_state.output_image[..],
        // )
        // .unwrap();
        // buffer.save("from_inner_state.png").unwrap();
        Ok(())
    }
}

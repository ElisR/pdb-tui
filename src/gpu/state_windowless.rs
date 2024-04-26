use image::{ImageBuffer, Rgba};
use std::iter;
use winit::dpi::PhysicalSize;

use crate::gpu::{
    model::{DrawLight, DrawModel},
    trivial_rasterizer::BasicGPURasterizer,
    InnerState, State,
};

const FONT_ASPECT_RATIO: f32 = 2.0;

#[derive(Debug, Clone, Copy)]
pub struct ValidGridSize {
    width: u32,
    height: u32,
}

impl ValidGridSize {
    /// Make sure that the grid size is valid
    /// Sizes need to be powers of two for the compute shader
    pub fn new(width: u32, height: u32) -> Self {
        // Check if width and height are powers of two
        let width_pow_2 = (width & (width - 1)) == 0;
        let height_pow_2 = (height & (height - 1)) == 0;

        if (width_pow_2 && height_pow_2 && (height == 2 * width)) || (height == 1 && width == 1) {
            Self { width, height }
        } else {
            Self {
                width: 1,
                height: 1,
            }
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug)]
pub struct WindowlessState {
    pub output_size: winit::dpi::PhysicalSize<u32>,
    pub output_buffer: wgpu::Buffer,
    pub output_image: Vec<u8>,
    pub texture: wgpu::Texture,
    pub intermediate_texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub intermediate_view: wgpu::TextureView,
    pub rasterizer: BasicGPURasterizer,
}

impl WindowlessState {
    const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;
    // TODO Remove these and refer to the GPU rasterizer version
    const INTERMEDIATE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Uint;

    /// Take a number of bytes and return the next closest multiple of 256
    pub fn pad_bytes_to_256(bytes: u32) -> u32 {
        (bytes + 255) & !255
    }

    /// Pad width to 64 since each pixel requires 4 bytes
    pub fn pad_width_to_64(width: u32) -> u32 {
        (width + 63) & !63
    }

    pub fn new(
        output_size: PhysicalSize<u32>,
        grid_size: ValidGridSize,
        device: &wgpu::Device,
    ) -> Self {
        // TODO Need to add functionality for changing this
        let output_buffer_size = (Self::U32_SIZE
            * Self::pad_width_to_64(output_size.width)
            * output_size.height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Windowless Output Buffer"),
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let intermediate_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: output_size.width * grid_size.width(),
                height: output_size.height * grid_size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::INTERMEDIATE_FORMAT,
            view_formats: &[],
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Intermediate Texture"),
        };
        let intermediate_texture = device.create_texture(&intermediate_texture_desc);
        let intermediate_view =
            intermediate_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let texture_desc = wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
            label: Some("Windowless Output Texture"),
        };
        let texture = device.create_texture(&texture_desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Multiply by 4 because RGBA
        let output_image_size = output_size.width as usize * output_size.height as usize * 4;
        let output_image = Vec::<u8>::with_capacity(output_image_size);

        let rasterizer = BasicGPURasterizer::new(grid_size, device, &intermediate_view, &view);

        Self {
            output_size,
            output_buffer,
            output_image,
            texture,
            intermediate_texture,
            view,
            intermediate_view,
            rasterizer,
        }
    }
}

impl InnerState for WindowlessState {
    fn output_size(&self) -> PhysicalSize<u32> {
        self.output_size
    }
    fn render_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.output_size.width * self.rasterizer.grid_size.width(),
            height: self.output_size.height * self.rasterizer.grid_size.height(),
        }
    }
    fn format(&self) -> wgpu::TextureFormat {
        Self::INTERMEDIATE_FORMAT
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        self.output_size = new_size;

        self.output_buffer.destroy();
        self.texture.destroy();
        self.intermediate_texture.destroy();

        // TODO Find a solution without repeating so much code
        let output_buffer_size = (Self::U32_SIZE
            * Self::pad_width_to_64(self.output_size.width)
            * self.output_size.height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Windowless Output Buffer"),
            mapped_at_creation: false,
        };
        self.output_buffer = device.create_buffer(&output_buffer_desc);

        let intermediate_texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.output_size.width * self.rasterizer.grid_size.width(),
                height: self.output_size.height * self.rasterizer.grid_size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::INTERMEDIATE_FORMAT,
            view_formats: &[],
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Intermediate Texture"),
        };
        self.intermediate_texture = device.create_texture(&intermediate_texture_desc);
        self.intermediate_view = self
            .intermediate_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.output_size.width,
                height: self.output_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::INTERMEDIATE_FORMAT,
            view_formats: &[], // NOTE This may be incorrect and needs to be checked
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::STORAGE_BINDING,
            label: Some("Windowless Output Texture"),
        };
        self.texture = device.create_texture(&texture_desc);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // TODO Move bind group creation to separate function

        self.rasterizer
            .resize(device, &self.intermediate_view, &self.view);
        // TODO Work out logic for new offset
    }
}

impl State<WindowlessState> {
    pub async fn new(output_size: PhysicalSize<u32>, grid_size: PhysicalSize<u32>) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        // TODO Consider moving this valid grid size creation into inner state
        let grid_size = ValidGridSize::new(grid_size.width, grid_size.height);
        let (_adapter, device, queue) = Self::create_adapter_device_queue(None, &instance).await;
        let inner_state = WindowlessState::new(output_size, grid_size, &device);
        let mut state = Self::new_from_inner_state(inner_state, device, queue).await;

        state.fix_aspect_ratio();
        state
    }

    /// Account for the fact that font height is roughly twice the width
    fn fix_aspect_ratio(&mut self) {
        let grid_ratio = self.inner_state.rasterizer.grid_size.height() as f32
            / self.inner_state.rasterizer.grid_size.width() as f32;
        self.camera.aspect /= FONT_ASPECT_RATIO / grid_ratio;
        self.update();
    }

    // TODO Need to change this error
    // TODO Need to refactor more out of this function
    pub async fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.inner_state.intermediate_view,
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
        {
            self.inner_state.rasterizer.run_compute(
                &mut encoder,
                self.inner_state.output_size().width,
                self.inner_state.output_size().width,
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
                    // Check that this isn't meant to be 4 `u8`s rather than 1 `u32`
                    bytes_per_row: Some({
                        let bytes =
                            WindowlessState::U32_SIZE * self.inner_state.output_size().width;
                        WindowlessState::pad_bytes_to_256(bytes)
                    }),
                    rows_per_image: Some(self.inner_state.output_size().height),
                },
            },
            // TODO Stop redefining the same size
            wgpu::Extent3d {
                width: self.inner_state.output_size().width,
                height: self.inner_state.output_size().height,
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
                    * WindowlessState::pad_width_to_64(self.inner_state.output_size().width)
                        as usize,
            )
            .flat_map(|row| {
                row.iter().take(
                    WindowlessState::U32_SIZE as usize
                        * self.inner_state.output_size().width as usize,
                )
            })
            .cloned()
            .collect();

        Ok(())
    }

    // TODO This is currently failing if run - fix it
    #[allow(dead_code)]
    pub fn save_screenshot(&self) {
        // TODO Fix the strangely sized buffer
        let now = chrono::Utc::now();
        let now_string = now.format("%H:%M:%S").to_string();
        let path = format!("from_inner_state_{}.png", now_string);
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
            WindowlessState::pad_width_to_64(self.inner_state.render_size().width),
            self.inner_state.render_size().height,
            &self.inner_state.output_image[..],
        )
        .unwrap();
        buffer.save(path).unwrap();
    }
}

// TODO Add tests back in for power-of-two tests

use image::{ImageBuffer, Rgba};
use std::iter;
use winit::dpi::PhysicalSize;

use crate::gpu::pdb_gpu::model::{DrawLight, DrawModel};
use crate::gpu::pdb_gpu::{InnerState, State};

#[derive(Debug)]
pub struct WindowlessState {
    pub size: winit::dpi::PhysicalSize<u32>,
    pub output_buffer: wgpu::Buffer,
    pub output_image: Vec<u8>,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Multiply by 4 because RGBA
        let output_image_size = size.width as usize * size.height as usize * 4;
        let output_image = Vec::<u8>::with_capacity(output_image_size);

        Self {
            size,
            output_buffer,
            output_image,
            texture,
            view,
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
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // TODO Work out logic for new offset
    }
}

impl State<WindowlessState> {
    pub async fn new(size: PhysicalSize<u32>) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let (_adapter, device, queue) = Self::create_adapter_device_queue(None, &instance).await;
        let inner_state = WindowlessState::new(size, &device);
        let mut state = Self::new_from_inner_state(inner_state, device, queue).await;

        // Accounting for the fact that font height is roughly twice the width
        state.camera.aspect /= 2.0;
        state.update();
        state
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
                    // Check that this isn't meant to be 4 `u8`s rather than 1 `u32`
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
            WindowlessState::pad_width_to_64(self.inner_state.size().width),
            self.inner_state.size().height,
            &self.inner_state.output_image[..],
        )
        .unwrap();
        buffer.save(path).unwrap();
    }
}
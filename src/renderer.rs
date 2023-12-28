use tracing::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{InstanceDescriptor, SurfaceConfiguration};
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::WindowEvent;
use winit::window::Window;

use crate::PIXEL_SIZE;
use chip8::gpu::{Pixel, DISPLAY_HEIGHT, DISPLAY_WIDTH};

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    // render_pipeline: wgpu::RenderPipeline,
    window: Window,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        // TODO: figure out why window.inner_size is 0 in wasm
        let size = LogicalSize::new(
            DISPLAY_WIDTH as u32 * PIXEL_SIZE as u32,
            DISPLAY_HEIGHT as u32 * PIXEL_SIZE as u32,
        );

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 +
        // Browser WebGPU
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..InstanceDescriptor::default()
        });
        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Unable to create surface")
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let default_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("Unable to get default config");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: default_config.format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            ..default_config
        };
        surface.configure(&device, &config);
        Self {
            surface,
            device,
            queue,
            config,
            size: size.to_physical(window.scale_factor()),
            window,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        todo!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn render(&mut self, pixels: &[Pixel]) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }
}

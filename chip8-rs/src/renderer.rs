use wgpu::{include_wgsl, InstanceDescriptor, RenderPassDescriptor, StoreOp};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

use chip8::gpu::Pixel;

pub const DEFAULT_PIXEL_SIZE: usize = 10;

const PIXEL_ON_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

const PIXEL_OFF_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

struct PixelGridSize {
    width: usize,
    height: usize,
}

impl PixelGridSize {
    fn size(&self) -> usize {
        self.width * self.height
    }
}

#[derive(Debug)]
#[cfg_attr(debug_assertions, derive(PartialEq))]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(point: Point<T>) -> Self {
        [point.x, point.y]
    }
}

impl<T: Copy> From<[T; 2]> for Point<T> {
    fn from(point: [T; 2]) -> Self {
        Point {
            x: point[0],
            y: point[1],
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Color {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Vertex::ATTRIBS,
        }
    }
}

pub struct PixelRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pixel_grid_size: PixelGridSize,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl PixelRenderer {
    pub async fn new(window: &Window, height: usize, width: usize) -> Self {
        // TODO: figure out why window.inner_size is 0 in wasm
        let size = PhysicalSize::new(
            width as u32 * DEFAULT_PIXEL_SIZE as u32,
            height as u32 * DEFAULT_PIXEL_SIZE as u32,
        );

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + WebGPU
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..InstanceDescriptor::default()
        });
        let surface = unsafe {
            instance
                .create_surface(window)
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

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .first()
            .expect("No surface formats available")
            .to_owned();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![format],
        };
        surface.configure(&device, &config);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice::<Vertex, u8>(&[]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pixel_grid_size: PixelGridSize {
                width,
                height,
            },
            render_pipeline,
            vertex_buffer,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
        // TODO: handle input
    }

    pub fn update(&mut self) {
        // todo!()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, pixels: &[Pixel]) -> Result<(), wgpu::SurfaceError> {
        debug_assert_eq!(pixels.len(), self.pixel_grid_size.size());

        let pixel_vertices = self.create_pixel_vertices(pixels, &self.pixel_grid_size);

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(PIXEL_OFF_COLOR.to_wgpu()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..(pixel_vertices.len() as u32), 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    fn create_pixel_vertices(&self, pixels: &[Pixel], pixel_grid_size: &PixelGridSize) -> Vec<Vertex> {
        debug_assert_eq!(pixels.len(), pixel_grid_size.size());
        // TODO: only render pixels in a field with the same aspect ratio as the grid, if the window is a different aspect ratio

        let pixel_height = self.size.height as f32 / pixel_grid_size.height as f32;
        let pixel_width = self.size.width as f32 / pixel_grid_size.width as f32;

        let mut vertices = Vec::with_capacity(pixels.len() * 6);
        for j in 0..pixel_grid_size.height {
            for i in 0..pixel_grid_size.width {
                let pixel = pixels[j * pixel_grid_size.width + i];
                let x = i as f32 * pixel_width;
                let y = j as f32 * pixel_height;
                match pixel {
                    Pixel::On => {
                        let pixel_vertices = build_pixel_vertices(
                            Point {
                                x,
                                y,
                            },
                            pixel_width,
                            pixel_height,
                            PIXEL_ON_COLOR,
                        );
                        vertices.extend_from_slice(&pixel_vertices);
                    }
                    Pixel::Off => (),
                };
            }
        }
        let vertices = vertices.iter().map(|v| Vertex {
            position: polar_to_ndc(&self.size, v.position.into()).into(),
            color: v.color,
        }).collect::<Vec<_>>();

        debug_assert_eq!(vertices.len(), pixel_grid_size.size() * 4);

        vertices
    }

    fn create_pixel_indices(&self, pixels: &[Pixel], pixel_grid_size: &PixelGridSize) -> Vec<u32> {
        debug_assert_eq!(pixels.len(), pixel_grid_size.size());

        let mut indices = Vec::with_capacity(pixels.len() * 6);
        debug_assert_eq!(indices.len(), pixel_grid_size.size() * 6);

        indices
    }
}

fn build_pixel_vertices(point: Point<f32>, x_size: f32, y_size: f32, color: Color) -> [Vertex; 4] {
    let x = point.x;
    let y = point.y;

    let top_left = Vertex {
        position: [x, y],
        color: color.into(),
    };
    let top_right = Vertex {
        position: [x + x_size, y],
        color: color.into(),
    };
    let bottom_left = Vertex {
        position: [x, y + y_size],
        color: color.into(),
    };
    let bottom_right = Vertex {
        position: [x + x_size, y + y_size],
        color: color.into(),
    };

    [
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    ]
}

#[inline]
fn polar_to_ndc(size: &PhysicalSize<u32>, polar: Point<f32>) -> Point<f32> {
    let x = polar.x;
    let y = polar.y;

    let x = x / size.width as f32;
    let y = y / size.height as f32;
    let x = x * 2.0 - 1.0;
    let y = y * 2.0 - 1.0;

    Point {
        x,
        y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(10, 10, Point { x: 0.0, y: 0.0 }, Point { x: -1.0, y: -1.0 })]
    #[test_case(10, 10, Point { x: 10.0, y: 10.0 }, Point { x: 1.0, y: 1.0 })]
    #[test_case(10, 10, Point { x: 5.0, y: 5.0 }, Point { x: 0.0, y: 0.0 })]
    #[test_case(10, 20, Point { x: 5.0, y: 10.0 }, Point { x: 0.0, y: 0.0 })]
    #[test_case(20, 10, Point { x: 10.0, y: 5.0 }, Point { x: 0.0, y: 0.0 })]
    fn test_point_to_ndc(width: u32, height: u32, point: Point<f32>, expected: Point<f32>) {
        let size = PhysicalSize::new(width, height);
        let actual = polar_to_ndc(&size, point);
        assert_eq!(actual, expected);
    }
}

use pollster::FutureExt;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use wgpu::{
    Adapter, BindGroupLayout, Buffer, CommandEncoder, Device, Instance, InstanceDescriptor, Queue,
    RenderPass, RenderPipeline, Sampler, Surface, SurfaceConfiguration, SurfaceTexture,
    TextureView, Trace,
};
use winit::{dpi::PhysicalSize, window::Window};
use crate::{logger::Logger, make_error_result, Error, Position, Size, Color};

const BUFFER_SIZE: u64 = 10240 * 10240; // 2MB

/// A Vertex is the most elementary piece of information that is communicated to the GPU. Each set of 3 vertices forms a triangle.
///
/// The vertices must be arranged in counter-clockwise order: top, bottom left, bottom right
///
/// The position of a vertex is defined like so: ``position[0]`` is X, ``position[1]`` is Y and ``position[2]`` is Z
///
/// The position is relative. 0.0 is the middle of the screen. 0.9 is 90% of the way to the positive border (top of the screen for X). -0.9 is 90% of the way to the negative border (bottom of the screen for X)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    /// Describes how to read the vertex buffer
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // Defines how wide a vertex is
            // When the shader goes to read the next vertex, it will skip over the array_stride number of bytes
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Tells the pipeline whether each element of the array in this buffer represents per vertex data or per-instance data
            step_mode: wgpu::VertexStepMode::Vertex,
            // Describes the individual parts of a vertex
            // In our case it's a 1:1 mapping with the struct's fields
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    // Defines the offset in bytes until the attribute starts
                    // For the first attribute the offset is usually 0
                    offset: 0,
                    // This tells the shader what location to store this attibute at
                    // For example @location(0): vec3<f32> in the vertex shader would correspond to the position field of the vertex struct
                    shader_location: 0,
                    // Tells the shader the shape of the attribute
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    // @location(1)
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
/// Responsible for initialising WebGPU and communicating with the graphics card
pub struct Graphics {
    // Boilerplate
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Arc<Window>,
    logger: Rc<RefCell<Logger>>,

    // triangle rendering
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    command_vertex_buffer: Vec<Vertex>,
    command_index_buffer: Vec<u32>,
}

impl Graphics {
    pub fn draw_triangle(&mut self, positions: [Position; 3], color: Color) {
        let vertices: [Vertex; 3] = [
            Vertex {
                position: [positions[0].x, positions[0].y, 0.0],
                color: color.linear,
            },
            Vertex {
                position: [positions[1].x, positions[1].y, 0.0],
                color: color.linear,
            },
            Vertex {
                position: [positions[2].x, positions[2].y, 0.0],
                color: color.linear,
            },
        ];

        for v in vertices {
            self.command_vertex_buffer.push(v);
        }

        let vbl = self.command_vertex_buffer.len() as u32;

        let indices: [u32; 3] = [vbl - 3, vbl - 2, vbl - 1];

        for i in indices {
            self.command_index_buffer.push(i);
        }
    }

    pub(crate) fn new(
        window: Window,
        logger: Rc<RefCell<Logger>>,
        present_mode: wgpu::PresentMode,
    ) -> Self {
        // Boilerplate
        let window_arc: Arc<Window> = Arc::new(window);
        let size: winit::dpi::PhysicalSize<u32> = window_arc.inner_size();
        let instance = create_wgpu_instance();
        let surface = create_surface(&instance, &window_arc);
        let adapter = create_adapter(&instance, &surface);
        let (device, queue) = request_device(&adapter);
        let config = configure_surface(&surface, &adapter, size, &device, present_mode);

        // triangle rendering
        let render_pipeline = create_render_pipeline(&device, &config);
        let (vertex_buffer, index_buffer) = create_buffers(&device);

        Self {
            // Boileplate
            surface,
            device,
            queue,
            config,
            size,
            window: window_arc,
            logger,

            // triangle rendering
            render_pipeline,
            vertex_buffer,
            index_buffer,
            command_vertex_buffer: Vec::new(),
            command_index_buffer: Vec::new(),
        }
    }

    pub(crate) fn render(&mut self) {
        let output = self.surface.get_current_texture();
        match output {
            Ok(current_texture) => {
                let view = Self::create_view(&current_texture);
                let mut encoder = Self::create_encoder(&self.device);
                let mut render_pass = Self::create_render_pass(&mut encoder, view);

                render_pass.set_pipeline(&self.render_pipeline);
                // Writes vertex and index data to buffers
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.command_vertex_buffer),
                );
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    bytemuck::cast_slice(&self.command_index_buffer),
                );

                // Give the buffers to the render pass
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                // Encodes a render command, which when executed by the GPU will rasterize something and execute shaders
                render_pass.draw_indexed(0..self.command_index_buffer.len() as u32, 0, 0..1);

                drop(render_pass);

                self.command_vertex_buffer.clear();
                self.command_index_buffer.clear();

                self.queue.submit(std::iter::once(encoder.finish()));
                current_texture.present();
                self.window.request_redraw();
            }
            Err(error) => {
                self.logger.borrow().log_error(format!(
                    "Rendering error, could not get surface texture: {}",
                    error
                ));
            }
        }
    }

    /// Returns the window id of the window that is being rendered to
    pub(crate) fn get_window_id(&self) -> winit::window::WindowId {
        self.window.as_ref().id()
    }

    /// Every time the window is resized, we need to configure a new rendering surface
    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Redraws the window
    pub(crate) fn request_redraw(&self) {
        self.window.as_ref().request_redraw();
    }

    fn create_view(current_texture: &SurfaceTexture) -> TextureView {
        current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }

    fn create_render_pass(encoder: &mut CommandEncoder, view: TextureView) -> RenderPass {
        // A render pass may contain any number of drawing commands, and before/between each command, the render state may be updated
        // Each drawing command will be executed using the render state that has been set when the draw function is called
        // Generally we wnt to bundle as much rendering as possible into a single pass
        // If we have some rendering that depends on the output of a previous pass (shadows, reflections, VFX) then multipass rendering is unavoidable
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }
}

/// The adapter is a handle to the graphics card
///
/// It can be used to get information about the graphics card, such as it's name and what backend the adapter uses
///
/// It is used to create the Device and Queue
///
/// Panics if the adapter cannot be created
fn create_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface<'static>) -> wgpu::Adapter {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            // power_preference has two variants: LowPower and HighPerformance. LowPower favors battery life, e.g. it will pick an integrated GPU.
            power_preference: wgpu::PowerPreference::default(),
            // tells wgpu to find an adapter that can present to the supplied surface
            compatible_surface: Some(surface),
            // force_fallback_adapter forces wgpu to pick an adapter that will work on all hardware
            // this usually means that the rendering backend will use a "software" system instead of a hardware such as GPU
            force_fallback_adapter: false,
        })
        .block_on();
    match adapter {
        Ok(a) => return a,
        Err(err) => panic!("Could not initialize GPU adapter: {}", err),
    }
}

/// The surface is the part of the window that we draw to
///
/// We need it to draw directly to the screen
///
/// Window needs to implement raw-window-handle's HasRawWindowHandle trait, but fortunately winit's Window fits the bill
///
/// We need the surface to request an adapter, as the adapter must be compatible with the given surface
///
/// Panics if a rendering surface cannot be created
fn create_surface(instance: &Instance, window: &Arc<Window>) -> wgpu::Surface<'static> {
    let surface = instance.create_surface(window.clone());
    match surface {
        Ok(s) => return s,
        Err(err) => {
            panic!("Could not create rendering surface: {}", err);
        }
    }
}

/// The device is the logical connection to the adapter
///
/// It abstracts the underlying implementation and encapsulates a single connection
///
/// Panics if the device cannot be created
fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let device_desc = &wgpu::DeviceDescriptor {
        // required_features lets us specify what extra features we want
        // some features may not be supported on some devices
        // full list of features can be viewed here: https://docs.rs/wgpu/latest/wgpu/struct.Features.html
        required_features: wgpu::Features::empty(),
        // required_limits describes the limit of certain types of resources that we can create
        // full list of limits can be viewed here: https://docs.rs/wgpu/latest/wgpu/struct.Limits.html
        required_limits: wgpu::Limits::default(),
        label: None,
        // memory hints provides the adapter with a preferred memory allocation strategy, if supported
        // full list of memory hints can be viewed here: https://wgpu.rs/doc/wgpu/enum.MemoryHints.html
        memory_hints: Default::default(),
        trace: Trace::Off,
    };
    let device = adapter.request_device(device_desc).block_on();
    match device {
        Ok(d) => return d,
        Err(err) => panic!("Could not initialize connection to GPU: {}", err),
    }
}

/// This config defines how the surface creates it's underlying SurfaceTextures
fn configure_surface(
    surface: &wgpu::Surface<'static>,
    adapter: &wgpu::Adapter,
    size: winit::dpi::PhysicalSize<u32>,
    device: &wgpu::Device,
    present_mode: wgpu::PresentMode,
) -> wgpu::SurfaceConfiguration {
    let surface_capabilities = surface.get_capabilities(adapter);
    let surface_format = surface_capabilities
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_capabilities.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        // allows a texture to be an output of a render pass
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        // use a surface format compatible with our adapter
        format: surface_format,
        // The height and width should cover the entire size of the window
        width: size.width,
        height: size.height,
        present_mode,
        // https://docs.rs/wgpu/latest/wgpu/enum.CompositeAlphaMode.html
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        // not sure what this does
        view_formats: vec![],
        // just mouse over this one, it's complicated but very well explained in the docs
        desired_maximum_frame_latency: 2,
    };
    surface.configure(device, &config);
    config
}

// The instance is the first thing to create when using wgpu
// It's main purpose is to create Adapter(s) and Surface(s)
fn create_wgpu_instance() -> Instance {
    wgpu::Instance::new(&InstanceDescriptor {
        ..Default::default()
    })
}

/// Buffers are reserved memory where the CPU stores instructions for the GPU
///
/// A vertex buffer is just a vector of vertices for the GPU to render
///
/// An index buffer is created so that we can we can de-duplicate vertices
///
/// A square is made of 2 triangles, which are made of 6 vertices, but 2 of those are shared, so with indexing we save 33% memory
///
/// The buffers are populated in render()
fn create_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: BUFFER_SIZE,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Index Buffer"),
        size: BUFFER_SIZE,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    (vertex_buffer, index_buffer)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    // The shader tells the GPU what function to run for each vertex
    // Shaders can be layered, for now we just have a basic one for rendering our vertices
    // A vertex shader is used to manipulate vertices in order to transform the shape to look the way we want it
    // The vertices are then converted into fragments. Every pixel in the resulting image gets at least one fragment. A fragment shader decides what color the fragment will be
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    // A render pipeline describes all the actions the GPU will perform when acting on a set of data (vertices)
    // Our current render pipeline has one vertex shader and one fragment shader
    // Both of them are in shader.wgsl with different entrypoints
    // TODO: allow supplying custom shaders
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        // Supply the vertex shader
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        // Supply the fragment shader
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            // Here we tell WGPU that each set of 3 vertices should compose a triangle
            // Setting this to f.ex. LineList would tell wgpu that each pair of vertices composes a new line
            // It might be relevant to have multiple render pipelines when trying to draw lines efficiently
            // But maybe it's easier for now to just draw a very thin rectangle
            topology: wgpu::PrimitiveTopology::TriangleList,
            // This has no effect unless we use LineStrip or TriangleStrip topologies
            // https://docs.rs/wgpu/latest/wgpu/enum.PrimitiveTopology.html
            strip_index_format: None,
            // front_face and cull_mode tell WGPU how to determine whether a given triangle is facing forward or not
            // the current settings mean that a triangle is facing forward if the vertices are arranged in a counterclockwise direction
            // triangles that are not considered facing forward are culled (not included in the render) as specified by CullMode::Back
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than fill requires Features::NON_FILL_POYLGON
            // Tells wgpu to fill the area covered by our polygons
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            // Determines how many samples the pipeline will use
            count: 1,
            // Specifies which samples should be active, in this case we are using all of them
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // Indicates how many array layers the render attachments can have
        multiview: None,
        // Allows wgpu to cache shader compilation data. This might have an effect on performance.
        cache: None,
    });
    render_pipeline
}

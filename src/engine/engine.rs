#![allow(unused_variables)]

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Instant, Duration},
    default,
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, ElementState},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::{
        modifier_supplement::KeyEventExtModifierSupplement,
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
    },
    window::{Window, WindowId},
    dpi::{PhysicalSize},
    keyboard::{Key, PhysicalKey},
    platform::scancode::PhysicalKeyExtScancode,
};
use kira::{
    effect::panning_control::PanningControlHandle,
    sound::{PlaybackState, Sound},
    Easing, Panning, StartTime, Tween,
    sound::static_sound::{StaticSoundHandle, StaticSoundData},
    AudioManager, AudioManagerSettings, DefaultBackend,
};
use wgpu::{
    Adapter, BindGroupLayout, Buffer, CommandEncoder, Device, Instance, InstanceDescriptor, Queue,
    RenderPass, RenderPipeline, Sampler, Surface, SurfaceConfiguration, SurfaceTexture,
    TextureView, Trace, PresentMode,
};
use std::{
    str::FromStr,
    fmt::{Display, Formatter, Result as std_Result},
    io::{stdin, IsTerminal},
};
use chrono::Local;
use colored::{ColoredString, Colorize};
use smol_str::{format_smolstr, ToSmolStr};
use pollster::FutureExt;
pub use smol_str::SmolStr;

/*
*
* Core
*
*/

pub struct Engine {
    event_loop: Option<EventLoop<()>>,
    should_exit: bool,
    graphics: Option<Graphics>,
    audio: AudioManager,

    pub mouse_events: Vec<MouseEvent>,
    pub key_events: Vec<KeyEvent>,

    pub window_title: String,
    pub fullscreen: Fullscreen,
    pub window_size: Size,
    pub decorations: bool,
    pub resizable: bool,
    pub vsync: VSync,
    pub clear_color: Color,
}

impl Engine {
    pub fn new() -> Self {
        let graphics = None;
        let audio = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Cannot initialize audio");
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        Self {
            event_loop: Some(event_loop),
            should_exit: false,
            graphics,
            audio,

            mouse_events: Vec::new(),
            key_events: Vec::new(),

            window_title: String::from("wgpu-rs-demo"),
            fullscreen: Fullscreen::No,
            window_size: Size::new(1920, 1080),
            resizable: false,
            decorations: true,
            vsync: VSync::Off,
            clear_color: Color::from_linear(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn new_frame(&mut self) -> bool {
        self.mouse_events.clear();
        self.key_events.clear();

        let mut event_loop = self.event_loop.take().expect("Event loop is missing!");

        let status = event_loop.pump_app_events(Some(Duration::ZERO), self);

        self.event_loop = Some(event_loop);

        if self.should_exit {
            return false;
        }

        match status {
            PumpStatus::Exit(_) => false,
            _ => true,
        }
    }

    pub fn render(&mut self) {
        let graphics = self.graphics.as_mut().unwrap();
        let output = graphics.surface.get_current_texture();
        match output {
            Ok(current_texture) => {
                let view = create_view(&current_texture);
                let mut encoder = create_encoder(&graphics.device);
                let mut render_pass = create_render_pass(&mut encoder, view, self.clear_color);

                render_pass.set_pipeline(&graphics.render_pipeline);
                // Writes vertex and index data to buffers
                graphics.queue.write_buffer(
                    &graphics.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&graphics.command_vertex_buffer),
                );
                graphics.queue.write_buffer(
                    &graphics.index_buffer,
                    0,
                    bytemuck::cast_slice(&graphics.command_index_buffer),
                );

                // Give the buffers to the render pass
                render_pass.set_vertex_buffer(0, graphics.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(graphics.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                // Encodes a render command, which when executed by the GPU will rasterize something and execute shaders
                render_pass.draw_indexed(0..graphics.command_index_buffer.len() as u32, 0, 0..1);

                drop(render_pass);

                graphics.command_vertex_buffer.clear();
                graphics.command_index_buffer.clear();

                graphics.queue.submit(std::iter::once(encoder.finish()));
                current_texture.present();
                graphics.window.request_redraw();
            }
            Err(error) => {
                log(format!(
                    "Rendering error, could not get surface texture: {}",
                    error
                ));
            }
        }
    }

    pub fn play_sound(&mut self, resource: SoundResource) -> SoundHandle {
        let play_result = self.audio.play(resource.val);
        SoundHandle(play_result.unwrap())
    }

    pub fn draw_triangle(&mut self, positions: [Position; 3], color: Color) {
        let graphics = self.graphics.as_mut().unwrap();
        let vertices: [Vertex; 3] = [
            Vertex {
                position: [positions[0].x, positions[0].y, 0.0],
                color: Color::into_vertex(color),
            },
            Vertex {
                position: [positions[1].x, positions[1].y, 0.0],
                color: Color::into_vertex(color),
            },
            Vertex {
                position: [positions[2].x, positions[2].y, 0.0],
                color: Color::into_vertex(color),
            },
        ];

        for v in vertices {
            graphics.command_vertex_buffer.push(v);
        }

        let vbl = graphics.command_vertex_buffer.len() as u32;

        let indices: [u32; 3] = [vbl - 3, vbl - 2, vbl - 1];

        for i in indices {
            graphics.command_index_buffer.push(i);
        }
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let physical_size: winit::dpi::PhysicalSize<u32> = self.window_size.into();
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_resizable(self.resizable)
                    .with_decorations(self.decorations)
                    .with_fullscreen(self.fullscreen.into())
                    .with_inner_size(physical_size)
                    .with_title(&self.window_title),
            )
            .unwrap();

        self.graphics = Some(Graphics::new(window, self.vsync.into()));
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.graphics
            .as_ref()
            .unwrap()
            .window
            .as_ref()
            .request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let graphics = self.graphics.as_mut().unwrap();
        if self.graphics.as_ref().unwrap().window.as_ref().id() == window_id {
            match event {
                WindowEvent::CloseRequested => self.should_exit = true,
                WindowEvent::Resized(physical_size) => {
                    let graphics = self.graphics.as_mut().unwrap();
                    if physical_size.width > 0 && physical_size.height > 0 {
                        graphics.size = physical_size;
                        graphics.config.width = physical_size.width;
                        graphics.config.height = physical_size.height;
                        graphics
                            .surface
                            .configure(&graphics.device, &graphics.config);
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mouse_event = MouseEvent::from_winit_event(state, button);
                    self.mouse_events.push(mouse_event)
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let key_event = KeyEvent::from_winit_event(
                        event.state,
                        event.key_without_modifiers().as_ref(),
                        event.logical_key.as_ref(),
                        event.physical_key,
                        event.location,
                        event.repeat,
                    );
                    self.key_events.push(key_event)
                }
                _ => {}
            }
        }
    }
}

/*
*
* Audio
*
*/

#[derive(Clone)]
pub struct SoundResource {
    pub(crate) val: StaticSoundData,
}

impl SoundResource {
    pub fn from_path<P>(path: P) -> SoundResource
    where
        P: AsRef<Path>,
    {
        let sound_data_result = StaticSoundData::from_file(&path);
        return SoundResource {
            val: sound_data_result.unwrap(),
        };
    }
}

pub struct SoundHandle(pub(crate) StaticSoundHandle);
impl SoundHandle {
    const MIN_PANNING: f32 = -1.0;
    const MAX_PANNING: f32 = 1.0;

    pub fn pause(&mut self) {
        self.0.pause(new_none_tween());
    }

    pub fn pause_ease(&mut self, tween: Duration) {
        self.0.pause(new_tween(tween))
    }

    pub fn get_position_seconds(&self) -> f64 {
        self.0.position()
    }

    pub fn resume(&mut self) {
        self.0.resume(new_none_tween())
    }

    pub fn resume_ease(&mut self, tween: Duration) {
        self.0.resume(new_tween(tween))
    }

    pub fn seek_by(&mut self, seconds: f64) {
        self.0.seek_by(seconds)
    }

    pub fn seek_to(&mut self, seconds: f64) {
        self.0.seek_to(seconds)
    }

    pub fn stop(&mut self) {
        self.0.stop(new_none_tween())
    }

    pub fn stop_ease(&mut self, tween: Duration) {
        self.0.stop(new_tween(tween))
    }

    pub fn set_volume(&mut self, decibels: f32) {
        self.0.set_volume(decibels, new_none_tween());
    }

    pub fn set_volume_ease(&mut self, decibels: f32, tween: Duration) {
        self.0.set_volume(decibels, new_tween(tween));
    }

    pub fn loop_sound(&mut self) {
        self.0.set_loop_region(0.0..);
    }

    pub fn loop_sound_region(&mut self, range_seconds: Range<f64>) {
        self.0.set_loop_region(range_seconds)
    }

    pub fn set_panning(&mut self, panning: f32) {
        self.set_panning_ease(panning, Duration::from_micros(1));
    }

    pub fn set_panning_ease(&mut self, panning: f32, tween: Duration) {
        let mut resolved_panning = panning;
        if panning < Self::MIN_PANNING {
            resolved_panning = Self::MIN_PANNING
        } else if panning > Self::MAX_PANNING {
            resolved_panning = Self::MAX_PANNING
        }

        self.0.set_panning(resolved_panning, new_tween(tween));
    }
}

fn new_tween(duration: Duration) -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration,
        easing: Easing::Linear,
    }
}

fn new_none_tween() -> Tween {
    Tween {
        start_time: StartTime::Immediate,
        duration: Duration::from_micros(1),
        easing: Easing::Linear,
    }
}

/*
*
*
* Input
*
*/

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum EventAction {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    SideFront,
    SideBack,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum KeyLocation {
    Left,
    Right,
    Standard,
    Numpad,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub action: EventAction,
}

impl MouseEvent {
    pub(crate) fn from_winit_event(state: ElementState, button: winit::event::MouseButton) -> Self {
        let action = match state {
            ElementState::Pressed => EventAction::Pressed,
            ElementState::Released => EventAction::Released,
        };
        let button = match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Forward => MouseButton::SideFront,
            winit::event::MouseButton::Back => MouseButton::SideBack,
            winit::event::MouseButton::Other(value) => MouseButton::Other(value),
        };
        Self { button, action }
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Option<SmolStr>,
    pub logical_key: Option<SmolStr>,
    pub action: EventAction,
    pub scan_code: Option<u32>,
    pub is_repeat: bool,
    pub location: KeyLocation,
}

impl KeyEvent {
    pub(crate) fn from_winit_event(
        state: ElementState,
        winit_key_without_modifiers: Key<&str>,
        winit_logical_key: Key<&str>,
        winit_physical_key: PhysicalKey,
        winit_location: winit::keyboard::KeyLocation,
        is_repeat: bool,
    ) -> Self {
        let action = match state {
            ElementState::Pressed => EventAction::Pressed,
            ElementState::Released => EventAction::Released,
        };
        let key: Option<SmolStr> = match winit_key_without_modifiers {
            Key::Character(character) => Some(SmolStr::from_str(character).unwrap()),
            Key::Named(character) => Some(format_smolstr!("{:#?}", character)),
            Key::Unidentified(character) => Some(format_smolstr!("{:#?}", character)),
            Key::Dead(character) => match character {
                Some(c) => Some(c.to_smolstr()),
                None => None,
            },
        };
        let logical_key: Option<SmolStr> = match winit_logical_key {
            Key::Character(character) => Some(SmolStr::from_str(character).unwrap()),
            Key::Named(character) => Some(format_smolstr!("{:#?}", character)),
            Key::Unidentified(character) => Some(format_smolstr!("{:#?}", character)),
            Key::Dead(character) => match character {
                Some(c) => Some(c.to_smolstr()),
                None => None,
            },
        };
        let scan_code: Option<u32> = match winit_physical_key {
            PhysicalKey::Code(code) => code.to_scancode(),
            PhysicalKey::Unidentified(_) => None,
        };

        let location = match winit_location {
            winit::keyboard::KeyLocation::Left => KeyLocation::Left,
            winit::keyboard::KeyLocation::Right => KeyLocation::Right,
            winit::keyboard::KeyLocation::Standard => KeyLocation::Standard,
            winit::keyboard::KeyLocation::Numpad => KeyLocation::Numpad,
        };

        Self {
            action,
            key,
            logical_key,
            scan_code,
            is_repeat,
            location,
        }
    }
}

/*
*
*
* Logging
*
*/
#[cfg(debug_assertions)]
fn log<M>(message: M) -> ()
where
    M: AsRef<str>,
{
    use std::fmt::format;

    let formatted = format!(
        "{} :: {}",
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        message.as_ref()
    );
    println!("{}", formatted);
}

#[cfg(not(debug_assertions))]
fn log<F>(message: &str, level: LogLevel, color: F) -> ()
where
    F: FnOnce(&str) -> ColoredString,
{
}

/*
*
* Graphics
*
*/

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
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Arc<Window>,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    command_vertex_buffer: Vec<Vertex>,
    command_index_buffer: Vec<u32>,
}

impl Graphics {
    pub(crate) fn new(window: Window, present_mode: wgpu::PresentMode) -> Self {
        let window_arc: Arc<Window> = Arc::new(window);
        let size: winit::dpi::PhysicalSize<u32> = window_arc.inner_size();
        let instance = create_wgpu_instance();
        let surface = create_surface(&instance, &window_arc);
        let adapter = create_adapter(&instance, &surface);
        let (device, queue) = request_device(&adapter);
        let config = configure_surface(&surface, &adapter, size, &device, present_mode);

        let render_pipeline = create_render_pipeline(&device, &config);
        let (vertex_buffer, index_buffer) = create_buffers(&device);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window: window_arc,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            command_vertex_buffer: Vec::new(),
            command_index_buffer: Vec::new(),
        }
    }
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
fn create_render_pass(
    encoder: &mut CommandEncoder,
    view: TextureView,
    clear_color: Color,
) -> RenderPass<'_> {
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
                load: wgpu::LoadOp::Clear(Color::into_wgpu(clear_color)),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    })
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
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
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
        immediate_size: 0,
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
        // Allows wgpu to cache shader compilation data. This might have an effect on performance.
        cache: None,
        multiview_mask: None,
    });
    render_pipeline
}

#[derive(Clone, Copy)]
pub struct Color {
    pub linear: [f32; 4],
}

impl Color {
    pub fn from_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            linear: [r, g, b, a],
        }
    }

    pub(crate) fn into_wgpu(color: Color) -> wgpu::Color {
        wgpu::Color {
            r: color.linear[0] as f64,
            g: color.linear[1] as f64,
            b: color.linear[2] as f64,
            a: color.linear[3] as f64,
        }
    }

    pub(crate) fn into_vertex(color: Color) -> [f32; 3] {
        [color.linear[0], color.linear[1], color.linear[2]]
    }
}

#[derive(Clone, Copy)]
pub enum Fullscreen {
    Borderless,
    No,
}

impl Into<Option<winit::window::Fullscreen>> for Fullscreen {
    fn into(self) -> Option<winit::window::Fullscreen> {
        match self {
            Fullscreen::Borderless => Some(winit::window::Fullscreen::Borderless(None)),
            Fullscreen::No => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

impl Into<PhysicalSize<u32>> for Size {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}

#[derive(Copy, Clone)]
pub enum VSync {
    On,
    Off,
}

impl Into<PresentMode> for VSync {
    fn into(self) -> PresentMode {
        match self {
            Self::On => return PresentMode::AutoVsync,
            Self::Off => return PresentMode::AutoNoVsync,
        };
    }
}

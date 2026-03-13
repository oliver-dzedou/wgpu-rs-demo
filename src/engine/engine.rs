#![allow(unused_variables)]

use crate::{
    graphics, logger::Logger, sound::Sound, utils::Utils, Graphics, LogLevel, KeyEvent, MouseEvent,
    make_error_result, Error,
};
use std::{cell::RefCell, rc::Rc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{Window, WindowId},
    dpi::{PhysicalSize},
};
use wgpu::PresentMode;

pub struct Engine<N: Game> {
    node: N,
    logger: Rc<RefCell<Logger>>,
    graphics: Option<Graphics>,
    utils: Utils,
    sound: Sound,

    pub window_title: String,
    pub fullscreen: Fullscreen,
    pub window_size: Size,
    pub decorations: bool,
    pub resizable: bool,
    pub vsync: VSync,
}

impl<N: Game> Engine<N> {
    pub fn new(node: N) -> Self {
        let logger = Rc::new(RefCell::new(Logger::new()));
        let graphics = None;
        let utils = Utils::new();
        let sound = Sound::new(Rc::clone(&logger));

        Self {
            node,
            logger,
            graphics,
            utils,
            sound,

            // Options
            window_title: String::from("wgpu-rs-demo"),
            fullscreen: Fullscreen::No,
            window_size: Size::new(1920, 1080),
            resizable: false,
            decorations: true,
            vsync: VSync::Off,
        }
    }

    pub fn start(&mut self) {
        self.logger.borrow().log_info("Starting instance");
        env_logger::init();
        let now = Instant::now();
        self.utils.set_started_at(now);
        self.utils.set_previous_frame(now);
        self.utils.increment_frames();

        let event_loop_result: Result<EventLoop<()>, winit::error::EventLoopError> =
            EventLoop::new();
        match event_loop_result {
            Ok(event_loop) => {
                event_loop.set_control_flow(ControlFlow::Poll);
                let _ = event_loop.run_app(self);
            }
            Err(err) => {
                self.logger
                    .borrow()
                    .log_error(format!("Failed creating event loop: {}", err));
                panic!("Failed creating event loop: {}", err)
            }
        }
    }
}

impl<N: Game> ApplicationHandler for Engine<N> {
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

        self.graphics = Some(Graphics::new(
            window,
            Rc::clone(&self.logger),
            self.vsync.into(),
        ));
        self.node.init(
            self.graphics.as_mut().unwrap(),
            &mut self.sound,
            &mut self.logger.borrow_mut(),
        )
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.node.update(
            self.graphics.as_mut().unwrap(),
            &mut self.sound,
            &mut self.logger.borrow_mut(),
        );
        self.utils.increment_frames();
        self.utils.set_previous_frame(Instant::now());
        self.graphics.as_mut().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let graphics = self.graphics.as_mut().unwrap();
        if graphics.get_window_id() == window_id {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => graphics.resize(physical_size),
                WindowEvent::RedrawRequested => graphics.render(),
                WindowEvent::MouseInput { state, button, .. } => {
                    self.node.on_mouse_event(
                        graphics,
                        &mut self.sound,
                        &mut self.logger.borrow_mut(),
                        MouseEvent::from_winit_event(state, button),
                    );
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    self.node.on_key_event(
                        graphics,
                        &mut self.sound,
                        &mut self.logger.borrow_mut(),
                        KeyEvent::from_winit_event(
                            event.state,
                            event.key_without_modifiers().as_ref(),
                            event.logical_key.as_ref(),
                            event.physical_key,
                            event.location,
                            event.repeat,
                        ),
                    );
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub linear: [f32; 3],
}

impl Color {
    pub fn from_linear(r: f32, g: f32, b: f32) -> Self {
        Self { linear: [r, g, b] }
    }

    pub fn from_srgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            linear: [
                Self::from_srgb_single(r),
                Self::from_srgb_single(g),
                Self::from_srgb_single(b),
            ],
        }
    }

    pub fn from_hex<H>(hex: H) -> Result<Self, Error>
    where
        H: AsRef<str>,
    {
        let s = hex.as_ref().trim_start_matches('#');
        if s.len() != 6 {
            return make_error_result("Hex color must be exactly 6 digits", None);
        }
        let r = u8::from_str_radix(&s[0..2], 16);
        let g = u8::from_str_radix(&s[2..4], 16);
        let b = u8::from_str_radix(&s[4..6], 16);

        let r_linear = match r {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        let g_linear = match g {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        let b_linear = match b {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        Ok(Self {
            linear: [r_linear, g_linear, b_linear],
        })
    }

    fn from_srgb_single(c: u8) -> f32 {
        let fc = c as f32 / 255.0;
        if fc <= 0.04045 {
            fc / 12.92
        } else {
            ((fc + 0.055) / 1.055).powf(2.4)
        }
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

pub trait Game {
    fn init(&mut self, graphics: &mut Graphics, sound: &mut Sound, logger: &mut Logger) {}

    fn update(&mut self, graphics: &mut Graphics, sound: &mut Sound, logger: &mut Logger) {}

    fn on_mouse_event(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        logger: &mut Logger,
        event: MouseEvent,
    ) {
    }

    fn on_key_event(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        logger: &mut Logger,
        event: KeyEvent,
    ) {
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

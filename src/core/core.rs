use crate::{
    graphics, logger::Logger, save_manager::SaveManager, sound::Sound, utils::Utils, Fullscreen,
    Graphics, KeyEvent, LogLevel, MouseEvent, Node, Size, VSync,
};
use std::{cell::RefCell, rc::Rc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::{Window, WindowId},
};

/// Entrypoint to ``exlib``
pub struct Core<N: Node> {
    node: N,
    logger: Rc<RefCell<Logger>>,
    graphics: Option<Graphics>,
    utils: Utils,
    save_manager: SaveManager,
    sound: Sound,

    // Options
    pub window_title: String,
    pub fullscreen: Fullscreen,
    pub window_size: Size,
    pub decorations: bool,
    pub resizable: bool,
    pub vsync: VSync,
}

impl<N: Node> Core<N> {
    /// Creates a new instance of [Core]
    ///
    /// See [Node] and [Options] for more information on the parameters
    pub fn new(node: N) -> Self {
        let logger = Rc::new(RefCell::new(Logger::new()));
        let graphics = None;
        let utils = Utils::new();
        let save_manager = SaveManager::new();
        let sound = Sound::new(Rc::clone(&logger));

        Self {
            node,
            logger,
            graphics,
            utils,
            save_manager,
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

    /// Starts the application
    ///
    /// Calls [Node::config] on [Node] before starting the event loop
    pub fn start(&mut self) {
        self.logger.borrow().log_info("Starting instance");
        // We don't actually use ``env_logger`` for anything. The only reason this is needed is
        // because ``wgpu`` uses it under the hood. If we do not import and configure ``env_logger``,
        // then ``wgpu`` will just panic with a generic message when an error occurs
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

impl<N: Node> ApplicationHandler for Core<N> {
    /// Called when the application is resumed
    ///
    /// The application is also resumed when it starts
    ///
    /// It is possible but heavily discouraged to attempt creating a rendering surface before the first resumed
    /// call
    ///
    /// That's why we only initialize graphics here and not at start-up
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
            &mut self.save_manager,
            &mut self.logger.borrow_mut(),
        )
    }

    /// Called when all other events have been processed
    ///
    /// This is where we call [Node::update] for each registered [Node]
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.node.update(
            self.graphics.as_mut().unwrap(),
            &mut self.sound,
            &mut self.save_manager,
            &mut self.logger.borrow_mut(),
        );
        self.utils.increment_frames();
        self.utils.set_previous_frame(Instant::now());
        self.graphics.as_mut().unwrap().request_redraw();
    }

    /// Processes window events, f.ex. input
    ///
    /// Only if the window id of the event matches the window that we currently rendering to
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
                        &mut self.save_manager,
                        &mut self.logger.borrow_mut(),
                        MouseEvent::from_winit_event(state, button),
                    );
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    self.node.on_key_event(
                        graphics,
                        &mut self.sound,
                        &mut self.save_manager,
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

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use bitcode::{Decode, Encode};
use rand::Rng;
use wgpu_rs_demo::*;
use core::fmt;
use std::{
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

struct MyWorld {
    triangle_positions: [Position; 3],
    triangle_color: Color,
}

impl MyWorld {
    pub fn new() -> Self {
        Self {
            triangle_positions: [
                Position::new(0.0, 0.5),
                Position::new(-0.5, -0.5),
                Position::new(0.5, -0.5),
            ],
            triangle_color: Color::from_linear(0.0, 0.0, 0.0),
        }
    }
}

impl Game for MyWorld {
    fn init(&mut self, graphics: &mut Graphics, sound: &mut Sound, logger: &mut Logger) {
        logger.set_log_level(LogLevel::INFO);
    }

    fn update(&mut self, graphics: &mut Graphics, sound: &mut Sound, logger: &mut Logger) {
        graphics.draw_triangle(self.triangle_positions, self.triangle_color)
    }

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
        if event.key.as_ref().unwrap() == "q" {
            self.triangle_color = Color::from_linear(0.1, 0.3, 0.5);
        }
        if event.key.as_ref().unwrap() == "w" {
            self.triangle_color = Color::from_linear(0.0, 0.0, 0.0);
        }
    }
}

fn main() {
    let game = MyWorld::new();
    let mut engine = Engine::new(game);
    engine.resizable = false;
    engine.decorations = true;
    engine.window_title = String::from("Hello, World!");
    engine.window_size = Size::new(1920, 1080);
    engine.vsync = VSync::On;
    engine.start()
}

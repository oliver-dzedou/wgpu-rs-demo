#![allow(unused_variables)]
#![allow(unused_imports)]

use wgpu_rs_demo::*;

struct Game {
    triangle_color: Color,
    triangle_position: Position,
}

impl Game {
    pub fn new() -> Self {
        Self {
            triangle_color: Color::from_linear(0.40, 0.61, 0.41, 1.0),
            triangle_position: Position::new(0.0, 0.0),
        }
    }
}

fn main() {
    let mut game = Game::new();
    let mut engine = Engine::new();

    engine.resizable = false;
    engine.decorations = true;
    engine.window_title = String::from("Hello, Triangle");
    engine.window_size = Size::new(1920, 1080);
    engine.vsync = VSync::On;

    engine.clear_color = Color::from_linear(0.80, 0.14, 0.11, 1.0);

    let base_triangle = [
        Position::new(0.0, 0.1),
        Position::new(-0.1, -0.1),
        Position::new(0.1, -0.1),
    ];

    while engine.new_frame() {
        for key in &engine.key_events {
            if let Some(key_str) = key.logical_key.as_ref() {
                if key.action == EventAction::Pressed {
                    match key_str.as_str() {
                        "w" => game.triangle_position.y += 0.05,
                        "s" => game.triangle_position.y -= 0.05,
                        "a" => game.triangle_position.x -= 0.05,
                        "d" => game.triangle_position.x += 0.05,

                        "j" => game.triangle_color = Color::from_linear(0.40, 0.61, 0.41, 1.0),
                        "k" => game.triangle_color = Color::from_linear(0.84, 0.60, 0.13, 1.0),
                        _ => {}
                    }
                }
            }
        }

        engine.draw_triangle(
            [
                Position::new(
                    base_triangle[0].x + game.triangle_position.x,
                    base_triangle[0].y + game.triangle_position.y,
                ),
                Position::new(
                    base_triangle[1].x + game.triangle_position.x,
                    base_triangle[1].y + game.triangle_position.y,
                ),
                Position::new(
                    base_triangle[2].x + game.triangle_position.x,
                    base_triangle[2].y + game.triangle_position.y,
                ),
            ],
            game.triangle_color,
        );

        engine.render();
    }
}

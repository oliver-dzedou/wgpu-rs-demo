#![allow(unused_variables)]
use crate::{
    graphics::Graphics, logger::Logger, save_manager::SaveManager, sound::Sound, KeyEvent,
    MouseEvent,
};

/// Should hold game logic and state. Provides methods for running user defined code at various points in the game's runtime
///
/// Register a node when constructing [Core](crate::Core)
pub trait Node {
    /// Runs when the graphics have been initialized and the engine is ready
    fn init(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        save_manager: &mut SaveManager,
        logger: &mut Logger,
    ) {
    }

    /// Code to be executed each frame
    fn update(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        save_manager: &mut SaveManager,
        logger: &mut Logger,
    ) {
    }

    /// Code to be executed when the program receives a mouse event
    ///
    /// See [crate::MouseEvent]
    fn on_mouse_event(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        save_manager: &mut SaveManager,
        logger: &mut Logger,
        event: MouseEvent,
    ) {
    }

    /// Code to be executed when the program receives a key event
    ///
    /// See [crate::KeyEvent]
    fn on_key_event(
        &mut self,
        graphics: &mut Graphics,
        sound: &mut Sound,
        save_manager: &mut SaveManager,
        logger: &mut Logger,
        event: KeyEvent,
    ) {
    }
}

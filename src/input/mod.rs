use std::str::FromStr;

pub use smol_str::SmolStr;
use smol_str::{format_smolstr, ToSmolStr};
use winit::{
    event::ElementState,
    keyboard::{Key, PhysicalKey},
    platform::scancode::PhysicalKeyExtScancode,
};

/// Universal event action (pressed/released) for both [MouseEvent] and [KeyEvent]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum EventAction {
    Pressed,
    Released,
}

/// The mouse button that was pressed/released
///
/// SideFront and SideBack may not be present or consistent across all mice
///
/// Use [MouseButton::Other] for many-button mice
///
/// See [MouseEvent]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    SideFront,
    SideBack,
    Other(u16),
}

/// Signifies the key location for keys that occur in multiple places on a keyboard, such as Shift, Alt or the number keys
///
/// See [KeyEvent]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum KeyLocation {
    Left,
    Right,
    Standard,
    Numpad,
}

/// Occurs on a mouse button press/release
///
/// See [EventAction], [MouseButton] and [crate::Node::on_mouse_event]
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub action: EventAction,
}

impl MouseEvent {
    /// Converts [winit] mouse press information into a single [crate::MouseEvent]
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

/// Occurs on a keyboard button press/release
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
    /// Converts [winit] keystroke information into a single [crate::KeyEvent]
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
            /*
            Nasty hack relying on Debug impl to get this into a string, but it works  ¯\_(o_o)_/¯
            I don't know what the performance implications of this are, but I prefer the string interface
            over a trilion enums that the consumer would have to import, and keystrokes are not usually the bottleneck
            */
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

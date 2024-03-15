//! Processing the inputs from both windowed and terminal applications

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
// Want to define a from method for KeyEventKind

// TODO Also needs to work with modifiers

pub enum UnifiedKeyCode {
    Space,
    Q,
    H,
    J,
    K,
    L,
    U,
    D,
    Shift,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Unknown,
}

impl From<Event> for UnifiedKeyCode {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers: _, // TODO Account for this
                kind: KeyEventKind::Press,
                ..
            }) => match code {
                KeyCode::Char('q') => Self::Q,
                KeyCode::Char('h') => Self::H,
                KeyCode::Char('j') => Self::J,
                KeyCode::Char('k') => Self::K,
                KeyCode::Char('l') => Self::L,
                KeyCode::Char('u') => Self::U,
                KeyCode::Char('d') => Self::D,
                KeyCode::Char(' ') => Self::Space,
                KeyCode::Up => Self::Up,
                KeyCode::Down => Self::Down,
                KeyCode::Left => Self::Left,
                KeyCode::Right => Self::Right,
                _ => Self::Unknown,
            },
            _ => Self::Unknown,
        }
    }
}

impl<'a> From<WindowEvent<'a>> for UnifiedKeyCode {
    fn from(event: WindowEvent) -> Self {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => match keycode {
                VirtualKeyCode::Q => Self::Q,
                VirtualKeyCode::H => Self::H,
                VirtualKeyCode::J => Self::J,
                VirtualKeyCode::K => Self::K,
                VirtualKeyCode::L => Self::L,
                VirtualKeyCode::U => Self::U,
                VirtualKeyCode::D => Self::D,
                VirtualKeyCode::Space => Self::Space,
                VirtualKeyCode::Up => Self::Up,
                VirtualKeyCode::Down => Self::Down,
                VirtualKeyCode::Left => Self::Left,
                VirtualKeyCode::Right => Self::Right,
                VirtualKeyCode::LShift => Self::Shift,
                VirtualKeyCode::RShift => Self::Shift,
                _ => Self::Unknown,
            },
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventState, KeyModifiers};
    use winit::event::{DeviceId, ModifiersState};

    fn is_space(event: UnifiedKeyCode) -> bool {
        matches!(event, UnifiedKeyCode::Space)
    }

    #[test]
    pub fn test_window_event_conversion() {
        #[allow(deprecated)]
        let space_event = unsafe {
            WindowEvent::KeyboardInput {
                device_id: DeviceId::dummy(),
                input: KeyboardInput {
                    scancode: 0u32,
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    modifiers: ModifiersState::empty(),
                },
                is_synthetic: false,
            }
        };
        assert!(is_space(space_event.into()));
    }

    #[test]
    pub fn test_tui_event_conversion() {
        let space_event = Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });

        assert!(is_space(space_event.into()));

        let random_event = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });
        assert!(!is_space(random_event.into()));
    }
}

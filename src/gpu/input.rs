//! Processing the inputs from both windowed and terminal applications

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
// Want to define a from method for KeyEventKind

// TODO Also needs to work with modifiers

#[derive(Debug, Clone, Copy, Hash)]
pub struct UnifiedEvent {
    pub keycode: UnifiedKeyCode,
    pub kind: UnifiedKeyKind,
}

// TODO Consider changing `kind` to an `option` instead
#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub enum UnifiedKeyKind {
    Press,
    Release,
    Unknown,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
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

impl From<&Event> for UnifiedEvent {
    fn from(event: &Event) -> Self {
        match event {
            Event::Key(KeyEvent {
                code,
                modifiers: _, // TODO Account for this
                kind,
                ..
            }) => {
                let new_code = match code {
                    KeyCode::Char('q') => UnifiedKeyCode::Q,
                    KeyCode::Char('h') => UnifiedKeyCode::H,
                    KeyCode::Char('j') => UnifiedKeyCode::J,
                    KeyCode::Char('k') => UnifiedKeyCode::K,
                    KeyCode::Char('l') => UnifiedKeyCode::L,
                    KeyCode::Char('u') => UnifiedKeyCode::U,
                    KeyCode::Char('d') => UnifiedKeyCode::D,
                    KeyCode::Char(' ') => UnifiedKeyCode::Space,
                    KeyCode::Esc => UnifiedKeyCode::Esc,
                    KeyCode::Up => UnifiedKeyCode::Up,
                    KeyCode::Down => UnifiedKeyCode::Down,
                    KeyCode::Left => UnifiedKeyCode::Left,
                    KeyCode::Right => UnifiedKeyCode::Right,
                    _ => UnifiedKeyCode::Unknown,
                };
                let new_kind = match kind {
                    KeyEventKind::Press => UnifiedKeyKind::Press,
                    KeyEventKind::Release => UnifiedKeyKind::Release,
                    _ => UnifiedKeyKind::Unknown,
                };
                UnifiedEvent {
                    keycode: new_code,
                    kind: new_kind,
                }
            }
            _ => UnifiedEvent {
                keycode: UnifiedKeyCode::Unknown,
                kind: UnifiedKeyKind::Unknown,
            },
        }
    }
}

impl<'a> From<&WindowEvent<'a>> for UnifiedEvent {
    fn from(event: &WindowEvent) -> Self {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let new_code = match keycode {
                    VirtualKeyCode::Q => UnifiedKeyCode::Q,
                    VirtualKeyCode::H => UnifiedKeyCode::H,
                    VirtualKeyCode::J => UnifiedKeyCode::J,
                    VirtualKeyCode::K => UnifiedKeyCode::K,
                    VirtualKeyCode::L => UnifiedKeyCode::L,
                    VirtualKeyCode::U => UnifiedKeyCode::U,
                    VirtualKeyCode::D => UnifiedKeyCode::D,
                    VirtualKeyCode::Space => UnifiedKeyCode::Space,
                    VirtualKeyCode::Up => UnifiedKeyCode::Up,
                    VirtualKeyCode::Down => UnifiedKeyCode::Down,
                    VirtualKeyCode::Left => UnifiedKeyCode::Left,
                    VirtualKeyCode::Right => UnifiedKeyCode::Right,
                    VirtualKeyCode::LShift => UnifiedKeyCode::Shift,
                    VirtualKeyCode::RShift => UnifiedKeyCode::Shift,
                    _ => UnifiedKeyCode::Unknown,
                };
                let new_kind = match state {
                    ElementState::Pressed => UnifiedKeyKind::Press,
                    ElementState::Released => UnifiedKeyKind::Release,
                };
                UnifiedEvent {
                    keycode: new_code,
                    kind: new_kind,
                }
            }
            _ => UnifiedEvent {
                keycode: UnifiedKeyCode::Unknown,
                kind: UnifiedKeyKind::Unknown,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventState, KeyModifiers};
    use winit::event::{DeviceId, ModifiersState};

    fn is_space(event: UnifiedEvent) -> bool {
        matches!(event.keycode, UnifiedKeyCode::Space)
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
        assert!(is_space((&space_event).into()));
    }

    #[test]
    pub fn test_tui_event_conversion() {
        let space_event = Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });

        assert!(is_space((&space_event).into()));

        let random_event = Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });
        assert!(!is_space((&random_event).into()));
    }
}

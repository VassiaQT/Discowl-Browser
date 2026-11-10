use servo::{
    InputEvent, Key, KeyState, KeyboardEvent, Modifiers, MouseButton, MouseButtonAction,
    MouseButtonEvent, MouseMoveEvent, NamedKey, WebViewPoint, WheelDelta, WheelEvent, WheelMode,
};
use webrender_api::units::DevicePoint;
use winit::event::{ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta};
use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey as WinitNamedKey};

pub fn webview_point(x: f64, y: f64) -> WebViewPoint {
    WebViewPoint::from(DevicePoint::new(x as f32, y as f32))
}

pub fn mouse_button_event(
    state: ElementState,
    button: WinitMouseButton,
    x: f64,
    y: f64,
) -> Option<InputEvent> {
    let action = match state {
        ElementState::Pressed => MouseButtonAction::Down,
        ElementState::Released => MouseButtonAction::Up,
    };
    let button = match button {
        WinitMouseButton::Left => MouseButton::Primary,
        WinitMouseButton::Right => MouseButton::Secondary,
        WinitMouseButton::Middle => MouseButton::Auxiliary,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(v) => MouseButton::Other(v),
    };
    Some(InputEvent::MouseButton(MouseButtonEvent::new(
        action,
        button,
        webview_point(x, y),
    )))
}

pub fn mouse_move_event(x: f64, y: f64) -> InputEvent {
    InputEvent::MouseMove(MouseMoveEvent::new(webview_point(x, y)))
}

pub fn wheel_event(delta: MouseScrollDelta, x: f64, y: f64) -> InputEvent {
    let (dx, dy, mode) = match delta {
        MouseScrollDelta::LineDelta(dx, dy) => ((dx * 76.0) as f64, (dy * 76.0) as f64, WheelMode::DeltaLine),
        MouseScrollDelta::PixelDelta(delta) => (delta.x, delta.y, WheelMode::DeltaPixel),
    };
    InputEvent::Wheel(WheelEvent::new(
        WheelDelta { x: dx, y: dy, z: 0.0, mode },
        webview_point(x, y),
    ))
}

pub fn keyboard_event(event: &KeyEvent, modifiers: ModifiersState) -> InputEvent {
    let state = match event.state {
        ElementState::Pressed => KeyState::Down,
        ElementState::Released => KeyState::Up,
    };
    let key = match &event.logical_key {
        WinitKey::Named(named) => Key::Named(convert_named_key(named)),
        WinitKey::Character(s) => Key::Character(s.to_string()),
        WinitKey::Unidentified(_) | WinitKey::Dead(_) => Key::Named(NamedKey::Unidentified),
    };
    let mut servo_mods = Modifiers::empty();
    if modifiers.shift_key() { servo_mods.insert(Modifiers::SHIFT); }
    if modifiers.control_key() { servo_mods.insert(Modifiers::CONTROL); }
    if modifiers.alt_key() { servo_mods.insert(Modifiers::ALT); }
    if modifiers.super_key() { servo_mods.insert(Modifiers::META); }
    InputEvent::Keyboard(KeyboardEvent::new_without_event(
        state,
        key,
        servo::Code::Unidentified,
        servo::Location::Standard,
        servo_mods,
        event.repeat,
        false,
    ))
}

fn convert_named_key(key: &WinitNamedKey) -> NamedKey {
    match key {
        WinitNamedKey::Enter => NamedKey::Enter,
        WinitNamedKey::Tab => NamedKey::Tab,
        WinitNamedKey::Space => NamedKey::Space,
        WinitNamedKey::ArrowDown => NamedKey::ArrowDown,
        WinitNamedKey::ArrowLeft => NamedKey::ArrowLeft,
        WinitNamedKey::ArrowRight => NamedKey::ArrowRight,
        WinitNamedKey::ArrowUp => NamedKey::ArrowUp,
        WinitNamedKey::End => NamedKey::End,
        WinitNamedKey::Home => NamedKey::Home,
        WinitNamedKey::PageDown => NamedKey::PageDown,
        WinitNamedKey::PageUp => NamedKey::PageUp,
        WinitNamedKey::Backspace => NamedKey::Backspace,
        WinitNamedKey::Delete => NamedKey::Delete,
        WinitNamedKey::Escape => NamedKey::Escape,
        WinitNamedKey::F1 => NamedKey::F1,
        WinitNamedKey::F2 => NamedKey::F2,
        WinitNamedKey::F3 => NamedKey::F3,
        WinitNamedKey::F4 => NamedKey::F4,
        WinitNamedKey::F5 => NamedKey::F5,
        WinitNamedKey::F6 => NamedKey::F6,
        WinitNamedKey::F7 => NamedKey::F7,
        WinitNamedKey::F8 => NamedKey::F8,
        WinitNamedKey::F9 => NamedKey::F9,
        WinitNamedKey::F10 => NamedKey::F10,
        WinitNamedKey::F11 => NamedKey::F11,
        WinitNamedKey::F12 => NamedKey::F12,
        _ => NamedKey::Unidentified,
    }
}
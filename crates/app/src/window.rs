use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub fn create_window(event_loop: &ActiveEventLoop) -> Result<Rc<Window>, winit::error::OsError> {
    let attrs = WindowAttributes::default()
        .with_title("Discowl Browser")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .with_min_inner_size(LogicalSize::new(800.0, 600.0))
        .with_visible(true);
    let window = event_loop.create_window(attrs)?;
    Ok(Rc::new(window))
}
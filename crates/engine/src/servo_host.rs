use servo::{EventLoopWaker, RenderingContext, Servo, ServoBuilder, WebView, WebViewBuilder, WebViewDelegate, WindowRenderingContext};
use std::rc::Rc;
use std::sync::mpsc;
use tracing::info;
use url::Url;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use winit::dpi::PhysicalSize;

pub struct ServoHost {
    servo: Servo,
    event_sender: mpsc::Sender<ServoEvent>,
}

#[derive(Debug)]
pub enum ServoEvent {
    Redraw,
    Exit,
}

impl ServoHost {
    pub fn new(event_sender: mpsc::Sender<ServoEvent>) -> Result<Self, Box<dyn std::error::Error>> {
        let waker = ServoWaker::new(event_sender.clone());

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();
        // Note: servo.setup_logging() is intentionally NOT called here.
        // The application initializes tracing_subscriber in main() instead.
        // Calling setup_logging() after tracing is initialized causes "Failed to set logger" panic.

        info!("Servo initialized successfully");

        Ok(Self { servo, event_sender })
    }

    pub fn create_rendering_context(&self, window: &(impl HasWindowHandle + HasDisplayHandle), size: PhysicalSize<u32>) -> Result<Rc<dyn RenderingContext>, Box<dyn std::error::Error>> {
        let window_handle = window.window_handle()?;
        let display_handle = window.display_handle()?;
        let context = WindowRenderingContext::new(display_handle, window_handle, size)
            .map_err(|e| Box::new(std::io::Error::other(format!("{:?}", e))) as Box<dyn std::error::Error>)?;
        Ok(Rc::new(context))
    }

    pub fn create_webview(
        &self,
        rendering_context: Rc<dyn RenderingContext>,
        delegate: Rc<dyn WebViewDelegate>,
        url: Option<Url>,
    ) -> WebView {
        let mut builder = WebViewBuilder::new(&self.servo, rendering_context).delegate(delegate);
        if let Some(url) = url {
            builder = builder.url(url);
        }
        builder.build()
    }

    pub fn spin(&self) {
        self.servo.spin_event_loop();
    }

    pub fn request_redraw(&self) {
        let _ = self.event_sender.send(ServoEvent::Redraw);
    }

    pub fn servo(&self) -> &Servo {
        &self.servo
    }
}

struct ServoWaker {
    sender: mpsc::Sender<ServoEvent>,
}

impl ServoWaker {
    fn new(sender: mpsc::Sender<ServoEvent>) -> Self {
        Self { sender }
    }
}

impl EventLoopWaker for ServoWaker {
    fn wake(&self) {
        let _ = self.sender.send(ServoEvent::Redraw);
    }

    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(ServoWaker::new(self.sender.clone()))
    }
}
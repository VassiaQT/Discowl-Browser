mod window;

// Required by surfman for GPU selection on Windows
#[cfg(target_os = "windows")]
surfman::declare_surfman!();

use discowl_browser_core::BrowserCore;
use discowl_engine::{
    DiscowlWebViewDelegate, UiWebViewDelegate, ServoEvent, ServoHost, WebViewManager,
    LayoutState, Rects, HitTarget, compute, hit_test,
    convert_mouse_input, convert_mouse_move, convert_key_event, convert_wheel,
};
use discowl_ipc::{CoreEvent, TabId, UiCommand};
use discowl_storage::Profile;
use euclid::default::Size2D;
use std::rc::Rc;
use std::sync::mpsc;
use tracing::{debug, info};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::Window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Discowl Browser");

    let profile = Profile::new()?;
    let core = Rc::new(BrowserCore::new(profile)?);

    let event_loop = EventLoop::new()?;
    let proxy = event_loop.create_proxy();

    let (servo_tx, servo_rx) = mpsc::channel::<ServoEvent>();
    let (core_tx, core_rx) = mpsc::channel::<CoreEvent>();
    let (ui_cmd_tx, ui_cmd_rx) = mpsc::channel::<UiCommand>();
    let (redraw_tx, redraw_rx) = mpsc::channel::<()>();

    core.set_event_sender(core_tx.clone());

    let mut app = App {
        _proxy: proxy,
        window: None,
        servo_host: None,
        webview_manager: None,
        core,
        layout_state: LayoutState::default(),
        rects: Rects {
            toolbar: euclid::default::Rect::zero(),
            sidebar: None,
            content: euclid::default::Rect::zero(),
        },
        servo_rx,
        core_rx,
        ui_cmd_rx,
        redraw_rx,
        servo_tx,
        _core_tx: core_tx,
        ui_cmd_tx,
        redraw_tx,
        active_tab: TabId::new(),
        focused_webview: FocusedWebview::Content,
        initialized: false,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    _proxy: winit::event_loop::EventLoopProxy<()>,
    window: Option<Rc<Window>>,
    servo_host: Option<Rc<ServoHost>>,
    webview_manager: Option<Rc<WebViewManager>>,
    core: Rc<BrowserCore>,
    layout_state: LayoutState,
    rects: Rects,
    servo_rx: mpsc::Receiver<ServoEvent>,
    core_rx: mpsc::Receiver<CoreEvent>,
    ui_cmd_rx: mpsc::Receiver<UiCommand>,
    redraw_rx: mpsc::Receiver<()>,
    servo_tx: mpsc::Sender<ServoEvent>,
    _core_tx: mpsc::Sender<CoreEvent>,
    ui_cmd_tx: mpsc::Sender<UiCommand>,
    redraw_tx: mpsc::Sender<()>,
    active_tab: TabId,
    focused_webview: FocusedWebview,
    initialized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusedWebview {
    Toolbar,
    Content,
    Sidebar,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.initialized {
            return;
        }

        let window = match window::create_window(event_loop) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };
        let window_size = window.inner_size();

        let servo_host = match ServoHost::new(self.servo_tx.clone()) {
            Ok(sh) => Rc::new(sh),
            Err(e) => {
                tracing::error!("Failed to create ServoHost: {}", e);
                event_loop.exit();
                return;
            }
        };

        let rendering_context = match servo_host.create_rendering_context(window.as_ref(), window_size) {
            Ok(rc) => rc,
            Err(e) => {
                tracing::error!("Failed to create rendering context: {}", e);
                event_loop.exit();
                return;
            }
        };

        let webview_manager = Rc::new(WebViewManager::new(servo_host.clone()));

        let active_tab = self.core.tabs.get_active().unwrap_or_else(TabId::new);
        self.active_tab = active_tab;

        let delegate = Rc::new(DiscowlWebViewDelegate::new(active_tab, self.core.clone(), self.redraw_tx.clone()));
        let _content_webview = webview_manager.create_content_webview(
            active_tab,
            rendering_context.clone(),
            delegate.clone(),
            None,
        );

        let ui_delegate = Rc::new(UiWebViewDelegate::new(self.core.clone(), self.ui_cmd_tx.clone(), self.redraw_tx.clone()));
        let _ui_webview = webview_manager.create_ui_webview(
            rendering_context.clone(),
            ui_delegate.clone(),
            None,
        );

        let sidebar_delegate = Rc::new(DiscowlWebViewDelegate::new(TabId::new(), self.core.clone(), self.redraw_tx.clone()));
        let _sidebar_webview = webview_manager.create_sidebar_webview(
            rendering_context,
            sidebar_delegate.clone(),
            None,
        );

        let rects = compute(Size2D::new(window_size.width, window_size.height), &self.layout_state);
        self.rects = rects;
        webview_manager.resize_all(&self.rects);
        webview_manager.show_tab(active_tab);

        self.window = Some(window);
        self.servo_host = Some(servo_host);
        self.webview_manager = Some(webview_manager);
        self.initialized = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        let _window = match &self.window {
            Some(w) => w.clone(),
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.rects = compute(
                    Size2D::new(size.width, size.height),
                    &self.layout_state,
                );
                if let Some(wm) = &self.webview_manager {
                    wm.resize_all(&self.rects);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wm) = &self.webview_manager {
                    wm.paint_all();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(state, button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta);
            }
            WindowEvent::Focused(focused) => {
                debug!("Window focused: {}", focused);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process incoming messages
        while let Ok(cmd) = self.ui_cmd_rx.try_recv() {
            self.core.handle_command(cmd);
        }
        while let Ok(event) = self.core_rx.try_recv() {
            self.handle_core_event(event);
        }
        while let Ok(_) = self.redraw_rx.try_recv() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        while let Ok(event) = self.servo_rx.try_recv() {
            match event {
                ServoEvent::Redraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                ServoEvent::Exit => {
                    // Exit handled by event loop
                }
            }
        }
        if let Some(sh) = &self.servo_host {
            sh.spin();
        }
    }
}

impl App {
    fn handle_core_event(&mut self, event: CoreEvent) {
        use discowl_ipc::CoreEvent;

        match event {
            CoreEvent::TabsChanged { tabs: _, active } => {
                self.active_tab = active;
                if let Some(wm) = &self.webview_manager {
                    wm.show_tab(active);

                    wm.with_ui_webview(|ui_webview| {
                        let json = serde_json::to_string(&event).unwrap();
                        let script = format!("window.onCoreEvent({});", json);
                        let _ = ui_webview.evaluate_javascript(&script, |_| {});
                    });
                }
            }
            CoreEvent::TabUpdated { id, title, url, loading, can_go_back, can_go_forward } => {
                if let Some(wm) = &self.webview_manager {
                    wm.with_ui_webview(|ui_webview| {
                        let event = CoreEvent::TabUpdated { id, title, url, loading, can_go_back, can_go_forward };
                        let json = serde_json::to_string(&event).unwrap();
                        let script = format!("window.onCoreEvent({});", json);
                        let _ = ui_webview.evaluate_javascript(&script, |_| {});
                    });
                }
            }
            CoreEvent::SidebarToggled { open } => {
                self.layout_state.sidebar_open = open;
                if let Some(window) = &self.window {
                    let window_size = window.inner_size();
                    self.rects = compute(
                        Size2D::new(window_size.width, window_size.height),
                        &self.layout_state,
                    );
                    if let Some(wm) = &self.webview_manager {
                        wm.resize_all(&self.rects);

                        wm.with_ui_webview(|ui_webview| {
                            let json = serde_json::to_string(&event).unwrap();
                            let script = format!("window.onCoreEvent({});", json);
                            let _ = ui_webview.evaluate_javascript(&script, |_| {});
                        });
                    }
                }
            }
        }
    }

    fn handle_mouse_input(&mut self, state: winit::event::ElementState, button: winit::event::MouseButton) {
        if let Some(wm) = &self.webview_manager {
            wm.with_content_webview(self.active_tab, |webview| {
                let input = convert_mouse_input(state, button, 0.0, 0.0);
                if let Some(input_event) = input {
                    webview.notify_input_event(input_event);
                }
            });
        }
    }

    fn handle_cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        let target = hit_test(&self.rects, position.x as u32, position.y as u32);

        match target {
            HitTarget::Toolbar => self.focused_webview = FocusedWebview::Toolbar,
            HitTarget::Sidebar => self.focused_webview = FocusedWebview::Sidebar,
            HitTarget::Content => self.focused_webview = FocusedWebview::Content,
            _ => {}
        }

        // Forward to appropriate webview
        if let Some(wm) = &self.webview_manager {
            match self.focused_webview {
                FocusedWebview::Content => {
                    wm.with_content_webview(self.active_tab, |webview| {
                        let local_x = position.x - self.rects.content.origin.x as f64;
                        let local_y = position.y - self.rects.content.origin.y as f64;
                        let input = convert_mouse_move(local_x, local_y);
                        webview.notify_input_event(input);
                    });
                }
                FocusedWebview::Toolbar => {
                    wm.with_ui_webview(|webview| {
                        let input = convert_mouse_move(position.x, position.y);
                        webview.notify_input_event(input);
                    });
                }
                FocusedWebview::Sidebar => {
                    wm.with_sidebar_webview(|webview| {
                        let input = convert_mouse_move(position.x, position.y);
                        webview.notify_input_event(input);
                    });
                }
            }
        }
    }

    fn handle_keyboard_input(&mut self, event: winit::event::KeyEvent) {
        let modifiers = ModifiersState::empty(); // We'll track this separately if needed

        if event.state == winit::event::ElementState::Pressed {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyT) if modifiers.control_key() => {
                    self.core.handle_command(UiCommand::NewTab { url: None });
                    return;
                }
                PhysicalKey::Code(KeyCode::KeyW) if modifiers.control_key() => {
                    self.core.handle_command(UiCommand::CloseTab { id: self.active_tab });
                    return;
                }
                PhysicalKey::Code(KeyCode::KeyL) if modifiers.control_key() => {
                    // Focus URL bar - send to UI
                    if let Some(wm) = &self.webview_manager {
                        wm.with_ui_webview(|ui_webview| {
                            let _ = ui_webview.evaluate_javascript("window.focusUrlBar()", |_| {});
                        });
                    }
                    return;
                }
                _ => {}
            }
        }

        // Forward to focused webview
        let input = convert_key_event(&event, modifiers);
        if let Some(wm) = &self.webview_manager {
            match self.focused_webview {
                FocusedWebview::Content => {
                    wm.with_content_webview(self.active_tab, |webview| {
                        webview.notify_input_event(input);
                    });
                }
                FocusedWebview::Toolbar => {
                    wm.with_ui_webview(|webview| {
                        webview.notify_input_event(input.clone());
                    });
                }
                FocusedWebview::Sidebar => {
                    wm.with_sidebar_webview(|webview| {
                        webview.notify_input_event(input);
                    });
                }
            }
        }
    }

    fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        if let Some(wm) = &self.webview_manager {
            wm.with_content_webview(self.active_tab, |webview| {
                let input = convert_wheel(delta, 0.0, 0.0);
                webview.notify_input_event(input);
            });
        }
    }
}
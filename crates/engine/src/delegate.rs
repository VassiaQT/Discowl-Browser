use discowl_ipc::{TabId, UiCommand};
use servo::{WebView, WebViewDelegate};
use std::sync::mpsc;

pub struct DiscowlWebViewDelegate {
    pub tab_id: TabId,
    pub core: std::rc::Rc<discowl_browser_core::BrowserCore>,
    redraw_sender: mpsc::Sender<()>,
}

impl DiscowlWebViewDelegate {
    pub fn new(
        tab_id: TabId,
        core: std::rc::Rc<discowl_browser_core::BrowserCore>,
        redraw_sender: mpsc::Sender<()>,
    ) -> Self {
        Self {
            tab_id,
            core,
            redraw_sender,
        }
    }
}

impl WebViewDelegate for DiscowlWebViewDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        let _ = self.redraw_sender.send(());
    }
}

pub struct UiWebViewDelegate {
    pub core: std::rc::Rc<discowl_browser_core::BrowserCore>,
    pub ipc_sender: mpsc::Sender<UiCommand>,
    redraw_sender: mpsc::Sender<()>,
}

impl UiWebViewDelegate {
    pub fn new(
        core: std::rc::Rc<discowl_browser_core::BrowserCore>,
        ipc_sender: mpsc::Sender<UiCommand>,
        redraw_sender: mpsc::Sender<()>,
    ) -> Self {
        Self {
            core,
            ipc_sender,
            redraw_sender,
        }
    }
}

impl WebViewDelegate for UiWebViewDelegate {
    fn notify_new_frame_ready(&self, _webview: WebView) {
        let _ = self.redraw_sender.send(());
    }
}
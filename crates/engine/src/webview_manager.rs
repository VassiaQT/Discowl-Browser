use discowl_ipc::TabId;
use servo::{RenderingContext, WebView, WebViewDelegate};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::info;
use url::Url;
use winit::dpi::PhysicalSize;

pub struct WebViewManager {
    webviews: RefCell<HashMap<TabId, WebView>>,
    ui_webview: RefCell<Option<WebView>>,
    sidebar_webview: RefCell<Option<WebView>>,
    servo_host: Rc<crate::servo_host::ServoHost>,
}

impl WebViewManager {
    pub fn new(servo_host: Rc<crate::servo_host::ServoHost>) -> Self {
        Self {
            webviews: RefCell::new(HashMap::new()),
            ui_webview: RefCell::new(None),
            sidebar_webview: RefCell::new(None),
            servo_host,
        }
    }

    pub fn create_content_webview(
        &self,
        id: TabId,
        rendering_context: Rc<dyn RenderingContext>,
        delegate: Rc<dyn WebViewDelegate>,
        url: Option<Url>,
    ) -> WebView {
        let webview = self.servo_host.create_webview(rendering_context, delegate, url);
        self.webviews.borrow_mut().insert(id, webview.clone());
        info!("Created content webview for tab {}", id);
        webview
    }

    pub fn create_ui_webview(
        &self,
        rendering_context: Rc<dyn RenderingContext>,
        delegate: Rc<dyn WebViewDelegate>,
        url: Option<Url>,
    ) -> WebView {
        let webview = self.servo_host.create_webview(rendering_context, delegate, url);
        *self.ui_webview.borrow_mut() = Some(webview.clone());
        info!("Created UI webview");
        webview
    }

    pub fn create_sidebar_webview(
        &self,
        rendering_context: Rc<dyn RenderingContext>,
        delegate: Rc<dyn WebViewDelegate>,
        url: Option<Url>,
    ) -> WebView {
        let webview = self.servo_host.create_webview(rendering_context, delegate, url);
        *self.sidebar_webview.borrow_mut() = Some(webview.clone());
        info!("Created sidebar webview");
        webview
    }

    pub fn with_content_webview<R>(&self, id: TabId, f: impl FnOnce(&WebView) -> R) -> Option<R> {
        self.webviews.borrow().get(&id).map(f)
    }

    pub fn with_ui_webview<R>(&self, f: impl FnOnce(&WebView) -> R) -> Option<R> {
        self.ui_webview.borrow().as_ref().map(f)
    }

    pub fn with_sidebar_webview<R>(&self, f: impl FnOnce(&WebView) -> R) -> Option<R> {
        self.sidebar_webview.borrow().as_ref().map(f)
    }

    pub fn remove_content_webview(&self, id: TabId) -> bool {
        self.webviews.borrow_mut().remove(&id).is_some()
    }

    pub fn resize_all(&self, rects: &crate::layout::Rects) {
        if let Some(ui) = self.ui_webview.borrow().as_ref() {
            ui.resize(PhysicalSize::new(rects.toolbar.width().max(1), rects.toolbar.height().max(1)));
            ui.show();
        }

        if let Some(sidebar) = self.sidebar_webview.borrow().as_ref() {
            if let Some(sidebar_rect) = &rects.sidebar {
                sidebar.resize(PhysicalSize::new(
                    sidebar_rect.width().max(1),
                    sidebar_rect.height().max(1),
                ));
                sidebar.show();
            } else {
                sidebar.hide();
            }
        }

        let content_size = PhysicalSize::new(
            rects.content.width().max(1),
            rects.content.height().max(1),
        );
        for webview in self.webviews.borrow().values() {
            webview.resize(content_size);
        }
    }

    pub fn show_tab(&self, id: TabId) {
        for (tab_id, webview) in self.webviews.borrow().iter() {
            if *tab_id == id {
                webview.show();
            } else {
                webview.hide();
            }
        }
    }

    pub fn paint_all(&self) {
        if let Some(ui) = self.ui_webview.borrow().as_ref() {
            ui.paint();
        }
        if let Some(sidebar) = self.sidebar_webview.borrow().as_ref() {
            sidebar.paint();
        }
        for webview in self.webviews.borrow().values() {
            webview.paint();
        }
    }
}
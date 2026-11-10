use euclid::default::Rect;
use euclid::default::Size2D;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rects {
    pub toolbar: Rect<u32>,
    pub sidebar: Option<Rect<u32>>,
    pub content: Rect<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutState {
    pub toolbar_height: u32,
    pub sidebar_open: bool,
    pub sidebar_width: u32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            toolbar_height: 40,
            sidebar_open: false,
            sidebar_width: 300,
        }
    }
}

pub fn compute(window_size: Size2D<u32>, state: &LayoutState) -> Rects {
    let window_width = window_size.width;
    let window_height = window_size.height;
    let toolbar_height = state.toolbar_height.min(window_height);
    
    let toolbar = Rect::new(
        (0, 0).into(),
        (window_width, toolbar_height).into(),
    );

    let content_y = toolbar_height;
    let content_height = window_height.saturating_sub(toolbar_height);

    let (sidebar, content) = if state.sidebar_open {
        let sidebar_width = state.sidebar_width.min(window_width);
        let sidebar_rect = Rect::new(
            (0, content_y).into(),
            (sidebar_width, content_height).into(),
        );
        let content_rect = Rect::new(
            (sidebar_width, content_y).into(),
            (window_width.saturating_sub(sidebar_width), content_height).into(),
        );
        (Some(sidebar_rect), content_rect)
    } else {
        let content_rect = Rect::new(
            (0, content_y).into(),
            (window_width, content_height).into(),
        );
        (None, content_rect)
    };

    Rects {
        toolbar,
        sidebar,
        content,
    }
}

pub fn hit_test(rects: &Rects, x: u32, y: u32) -> HitTarget {
    let point = (x, y).into();
    
    if rects.toolbar.contains(point) {
        return HitTarget::Toolbar;
    }
    
    if let Some(sidebar) = rects.sidebar {
        if sidebar.contains(point) {
            return HitTarget::Sidebar;
        }
    }
    
    if rects.content.contains(point) {
        return HitTarget::Content;
    }
    
    HitTarget::None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitTarget {
    Toolbar,
    Sidebar,
    Content,
    None,
}
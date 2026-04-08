use std::sync::{Arc, Mutex};

pub struct WindowState {
    pub visible: Arc<Mutex<bool>>,    // 窗口是否可视
    pub allow_quit: Arc<Mutex<bool>>, // 窗口是否运行关闭
    pub minimized: Arc<Mutex<bool>>,  // 窗口是否最小化
    pub hwnd_usize: usize,
}

impl WindowState {
    pub fn new(
        visible: Arc<Mutex<bool>>,
        allow_quit: Arc<Mutex<bool>>,
        hwnd_usize: usize,
    ) -> Arc<Self> {
        let visible = Arc::clone(&visible);
        let allow_quit = Arc::clone(&allow_quit);
        let minimized = Arc::new(Mutex::new(false));
        Arc::new(Self {
            visible,
            allow_quit,
            minimized,
            hwnd_usize,
        })
    }
}

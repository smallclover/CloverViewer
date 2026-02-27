use std::sync::{Arc, Mutex};

pub struct WindowState{
    pub visible: Arc<Mutex<bool>>,// 窗口是否可视
    pub allow_quit: Arc<Mutex<bool>>,// 窗口是否运行关闭
    pub hwnd_isize: isize,
}

impl WindowState {
    pub fn new(visible: Arc<Mutex<bool>>, allow_quit: Arc<Mutex<bool>>, hwnd_isize: isize) -> Self {
        let visible = Arc::clone(&visible);
        let allow_quit = Arc::clone(&allow_quit);
        Self {
            visible,
            allow_quit,
            hwnd_isize
        }
    }
}

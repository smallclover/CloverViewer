pub struct ViewState {
    pub status_message: Option<String>,
    pub is_loading: bool, // 载入状态
    pub current_zoom: f32,// 当前缩放
    pub rotation_angle: f32, // 旋转角度
    pub dialog_open: bool, // 弹窗
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            status_message: None,
            is_loading: false,
            current_zoom: 1.0,
            rotation_angle: 0.0,
            dialog_open: false,
        }
    }
}
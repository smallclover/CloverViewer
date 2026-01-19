fn main() {
    // 编译添加快捷方式图标
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/clover_viewer.ico");
    res.compile().unwrap();
}

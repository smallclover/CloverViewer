use std::{env, fs, path::Path};

fn main() {
    // 1. 图标编译
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/images/clover_viewer.ico");
    res.compile().unwrap();

    // 2. 确定目标目录 (target/debug 或 target/release)
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Failed to find target dir");

    // 3. 定义 DLL 源路径 (项目根目录下的 lib)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_src = Path::new(&manifest_dir).join("lib");

    // 4. 执行拷贝逻辑
    let dlls = ["dav1d.dll"];
    for dll in &dlls {
        let src = lib_src.join(dll);
        let dst = target_dir.join(dll);

        if src.exists() {
            // 只有当文件不存在或有变化时才拷贝
            fs::copy(&src, &dst).ok();
        } else {
            println!("cargo:warning=DLL not found: {:?}", src);
        }
    }

    // 只要 lib 文件夹变动就触发重新构建
    println!("cargo:rerun-if-changed=lib");
}
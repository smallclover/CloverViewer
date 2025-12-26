use egui::{ColorImage, Context, TextureHandle};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use image::ImageReader;

pub enum LoadResult {
    Ok(TextureHandle),
    Err(String),
}

pub struct LoadMessage {
    pub path: PathBuf,
    pub result: LoadResult,
    pub is_priority: bool,
    pub is_thumbnail: bool, // 标记这是缩略图
}


pub struct ImageLoader {
    tx: Sender<LoadMessage>,
    pub rx: Receiver<LoadMessage>,
    pub is_loading: bool,
}

impl ImageLoader {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            is_loading: false,
        }
    }

    /// 异步加载
    /// is_priority 优先级
    pub fn load_async(&mut self, ctx: Context, path: PathBuf, is_priority: bool, size: Option<(u32, u32)>) {
        let tx = self.tx.clone();
        let path_clone = path.clone();
        let is_thumbnail = size.is_some();

        // 如果是当前正在看的图，我们标记为加载中
        if is_priority {
            self.is_loading = true;
        }

        thread::spawn(move || {
            // 根据优先级调整线程权重（OS 层面的简单优化）
            if !is_priority {
                thread::yield_now();
            }
            
            let result = match Self::decode_image(&path_clone, size) {
                Ok(color_image) => {
                    let name = if is_thumbnail {
                        format!("thumb_{}", path_clone.display())
                    }else{
                        path_clone.file_name().unwrap_or_default().to_string_lossy().into()
                    };

                    let tex = ctx.load_texture(name, color_image, Default::default());
                    LoadResult::Ok(tex)
                }
                Err(e) => LoadResult::Err(e),
            };
            let _ = tx.send(LoadMessage {
                path: path_clone,
                result,
                is_priority,
                is_thumbnail,
            });
            ctx.request_repaint(); // 唤醒 UI 渲染
        });
    }

    fn decode_image(path: &PathBuf, size: Option<(u32, u32)>) -> Result<ColorImage, String> {
        // 1. 打开文件并自动识别格式
        let img_reader = ImageReader::open(&path)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?;

        // 2. 解码图片为 DynamicImage
        let img = img_reader.decode()
            .map_err(|e| {
                eprintln!("解码失败详情: {:?}", e);
                format!("无法解析图片 (格式可能不支持): {}", e)
            })?;
        // 3. 处理缩放逻辑
        // 如果传入了 size，则调用 thumbnail 进行快速缩放。
        // thumbnail 会根据目标尺寸进行采样，比普通的 resize 快得多。
        // 如果传入了 size，则调用 thumbnail 进行快速缩放。
        // thumbnail 会根据目标尺寸进行采样，比普通的 resize 快得多。
        let processed_img = if let Some((w, h)) = size {
            img.thumbnail(w, h)
        } else {
            img
        };

        // 4. 将处理后的图片转换为 RGBA8 格式
        // 注意：这里的宽高必须取处理后(processed_img)的尺寸
        let rgba = processed_img.to_rgba8();
        let pixel_size = [rgba.width() as usize, rgba.height() as usize];

        Ok(ColorImage::from_rgba_unmultiplied(
            pixel_size,
            rgba.as_raw(),
        ))
    }
}

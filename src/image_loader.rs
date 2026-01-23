use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use egui::{ColorImage, Context, TextureHandle};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use image::ImageReader;
use exif::{In, Reader, Tag};
use image::metadata::Orientation;

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

        rayon::spawn(move || {
            // 如果是低优先级的缩略图，可以稍微让出 CPU
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

    // 将 EXIF 的数字映射到 image crate 的枚举
    fn map_exif_to_orientation(exif_val: u32) -> Orientation {
        match exif_val {
            1 => Orientation::NoTransforms,
            2 => Orientation::FlipHorizontal,
            3 => Orientation::Rotate180,
            4 => Orientation::FlipVertical,
            5 => Orientation::Rotate90FlipH,
            6 => Orientation::Rotate90,
            7 => Orientation::Rotate270FlipH,
            8 => Orientation::Rotate270,
            _ => Orientation::NoTransforms,
        }
    }

    fn decode_image(path: &PathBuf, size: Option<(u32, u32)>) -> Result<ColorImage, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);

        // A. 读取 EXIF 方向 (优化：复用 reader)
        let orientation_value = {
            let val = (|| {
                let exif = Reader::new().read_from_container(&mut reader).ok()?;
                exif.get_field(Tag::Orientation, In::PRIMARY)?
                    .value
                    .get_uint(0)
            })();
            // 无论成功与否，重置文件指针到开头
            let _ = reader.seek(SeekFrom::Start(0));
            val.unwrap_or(1)
        };

        // B. 解码图片
        let mut img = ImageReader::new(reader)
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;

        // C. 应用旋转
        let img_orient = Self::map_exif_to_orientation(orientation_value);
        img.apply_orientation(img_orient);

        // D. 后续处理...
        let processed_img = if let Some((w, h)) = size {
            img.resize_to_fill(w, h, image::imageops::FilterType::Nearest)
            // img.thumbnail(w, h)
        } else {
            img
        };

        let rgba = processed_img.to_rgba8();
        Ok(ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        ))
    }
}

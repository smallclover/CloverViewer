use std::io::{Cursor};
use egui::{ColorImage, Context, TextureHandle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use image::{DynamicImage};
use exif::{In, Reader, Tag};
use image::imageops::FilterType;
use image::metadata::Orientation;
use rayon::{ThreadPool, ThreadPoolBuilder};
use zune_jpeg::JpegDecoder;

pub struct LoadSuccess {
    pub texture: TextureHandle,
    pub raw_pixels: Arc<Vec<egui::Color32>>, // 原始像素快照
}

pub enum LoadResult {
    Ok(LoadSuccess),
    Err(String),
}

pub struct LoadMessage {
    pub path: PathBuf,
    pub result: LoadResult,
    pub is_priority: bool, // 加载优先级
    pub is_thumbnail: bool, // 缩略图
}


pub struct ImageLoader {
    tx: Sender<LoadMessage>,
    pub rx: Receiver<LoadMessage>,
    pub is_loading: bool,
    // 双线程池：主图池和缩略图池
    main_pool: ThreadPool,
    thumb_pool: ThreadPool,
}

impl ImageLoader {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        // 主池：专注当前大图，给 1-2 个线程保证绝对响应
        let main_pool = ThreadPoolBuilder::new()
            .num_threads(2)
            .thread_name(|i| format!("img-main-{}", i))
            .build()
            .expect("Failed to create main thread pool");

        // 缩略图池：利用剩余核心进行后台预加载
        let thumb_threads = (num_cpus::get() - 2).max(2);
        let thumb_pool = ThreadPoolBuilder::new()
            .num_threads(thumb_threads)
            .thread_name(|i| format!("img-thumb-{}", i))
            .build()
            .expect("Failed to create thumb thread pool");

        Self {
            tx,
            rx,
            is_loading: false,
            main_pool,
            thumb_pool,
        }
    }

    /// 异步加载
    /// is_priority 优先级
    pub fn load_async(&mut self, ctx: Context, path: PathBuf, is_priority: bool, size: Option<(u32, u32)>) {
        let tx = self.tx.clone();
        let path_clone = path.clone();
        let is_thumbnail = size.is_some();

        // 只有高优先级任务会触发全局加载状态锁定
        if is_priority {
            self.is_loading = true;
        }

        let target_pool = if is_priority {
            &self.main_pool
        } else {
            &self.thumb_pool
        };

        target_pool.spawn(move || {

            let result = match Self::decode_image(&path_clone, size) {
                Ok(color_image) => {
                    // 1. 在主线程创建纹理之前，先保留像素引用
                    let raw_pixels = Arc::new(color_image.pixels.clone());
                    let name = if is_thumbnail {
                        format!("thumb_{}", path_clone.display())
                    }else{
                        path_clone.file_name().unwrap_or_default().to_string_lossy().into()
                    };
                    // 2. 创建纹理
                    let tex = ctx.load_texture(name, color_image, Default::default());

                    // 3. 返回组合结构
                    LoadResult::Ok(LoadSuccess {
                        texture: tex,
                        raw_pixels,
                    })
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
        let data = std::fs::read(path).map_err(|e| e.to_string())?;

        let orientation_value = {
            let val = (|| {
                let exif = Reader::new()
                    .read_from_container(&mut std::io::Cursor::new(&data))
                    .ok()?;

                exif.get_field(Tag::Orientation, In::PRIMARY)?
                    .value
                    .get_uint(0)
            })();
            val.unwrap_or(1)
        };

        let is_jpeg = data.len() > 2 && data[0] == 0xFF && data[1] == 0xD8;

        let mut img = if is_jpeg {
            // --- JPEG 高速通道 ---
            let mut decoder = JpegDecoder::new(Cursor::new(&data));
            // 解码为 RGB (zune-jpeg 在 RGB 模式下极快)
            let pixels = decoder.decode().map_err(|e| e.to_string())?;
            let info = decoder.info().ok_or("Failed to get JPEG info")?;

            // 将 raw 像素封装进 image crate 的 buffer，以便后续使用 apply_orientation
            let rgb_buf = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                info.width as u32,
                info.height as u32,
                pixels
            ).ok_or("Failed to create image buffer")?;

            DynamicImage::ImageRgb8(rgb_buf)
        } else {
            // --- 通用通道 (PNG, WebP, etc.) ---
            image::load_from_memory(&data).map_err(|e| e.to_string())?
        };

        // 4. 应用旋转
        let img_orient = Self::map_exif_to_orientation(orientation_value);
        img.apply_orientation(img_orient);

        // 5. 后续处理 (缩略图/缩放)
        let processed_img = if let Some((w, h)) = size {
            // 注意：如果是生成预览图，thumbnail 算法通常比 resize_to_fill 快得多
            //img.thumbnail(w, h)
            img.resize_to_fill(w, h, FilterType::Nearest)
        } else {
            img
        };

        // 6. 转换为 egui 格式
        let rgba = processed_img.to_rgba8();
        Ok(ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        ))
    }
}

use egui::{ColorImage, Context, TextureHandle};
use std::{
    io::Cursor,
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{channel, Receiver, Sender}
    },
    fs
};
use image::{
    DynamicImage,
    imageops::FilterType,
    metadata::Orientation,
    ImageBuffer,Rgb
};
use rayon::{
    ThreadPool, ThreadPoolBuilder
};
use zune_jpeg::JpegDecoder;
use exif::Tag;
use crate::model::image_meta::ImageProperties;
use crate::os::window::load_thumbnail_windows;

pub struct LoadSuccess {
    pub texture: TextureHandle,
    pub raw_pixels: Arc<Vec<egui::Color32>>, // 原始像素快照
    pub properties: ImageProperties,
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
            let result = if is_thumbnail {
                // 尝试使用 Windows API 加载缩略图
                #[cfg(target_os = "windows")]
                {
                    match load_thumbnail_windows(&path_clone, size.unwrap()) {
                        Ok(color_image) => {
                             let raw_pixels = Arc::new(color_image.pixels.clone());
                             let tex = ctx.load_texture(format!("thumb_{}", path_clone.display()), color_image, Default::default());
                             LoadResult::Ok(LoadSuccess {
                                texture: tex,
                                raw_pixels,
                                properties: ImageProperties::default(), // 缩略图不需要详细属性
                            })
                        },
                        Err(_) => {
                            // 降级到普通加载
                            Self::load_normal(&ctx, &path_clone, size)
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Self::load_normal(&ctx, &path_clone, size)
                }
            } else {
                Self::load_normal(&ctx, &path_clone, size)
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

    fn load_normal(ctx: &Context, path: &PathBuf, size: Option<(u32, u32)>) -> LoadResult {
         match Self::decode_image(path, size) {
                Ok((color_image, properties)) => {
                    // 1. 在主线程创建纹理之前，先保留像素引用
                    let raw_pixels = Arc::new(color_image.pixels.clone());
                    let name = if size.is_some() {
                        format!("thumb_{}", path.display())
                    }else{
                        path.file_name().unwrap_or_default().to_string_lossy().into()
                    };
                    // 2. 创建纹理
                    let tex = ctx.load_texture(name, color_image, Default::default());

                    // 3. 返回组合结构
                    LoadResult::Ok(LoadSuccess {
                        texture: tex,
                        raw_pixels,
                        properties,
                    })
                }
                Err(e) => LoadResult::Err(e),
            }
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

    fn extract_exif_properties(data: &[u8], properties: &mut ImageProperties) -> u32 {
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(data)) {
            let get_val = |tag| exif.get_field(tag, exif::In::PRIMARY).map(|f| f.display_value().to_string());

            properties.date = get_val(Tag::DateTime).unwrap_or_default();
            properties.make = get_val(Tag::Make).unwrap_or_default();
            properties.model = get_val(Tag::Model).unwrap_or_default();
            properties.f_number = get_val(Tag::FNumber).unwrap_or_default();
            properties.iso = exif.get_field(Tag::PhotographicSensitivity, exif::In::PRIMARY)
                .and_then(|f| f.value.get_uint(0))
                .map(|v| v as u32);
            properties.focal_length = get_val(Tag::FocalLength).unwrap_or_default();

            return exif.get_field(Tag::Orientation, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
                .map(|v| v as u32)
                .unwrap_or(1);
        }
        1
    }


    fn decode_image(path: &PathBuf, size: Option<(u32, u32)>) -> Result<(ColorImage, ImageProperties), String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;

        let mut properties = ImageProperties {
            path: path.clone(),
            size: metadata.len(),
            name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            ..Default::default()
        };

        let orientation_value = Self::extract_exif_properties(&data, &mut properties);

        let is_jpeg = data.len() > 2 && data[0] == 0xFF && data[1] == 0xD8;

        let mut img = if is_jpeg {
            let mut decoder = JpegDecoder::new(Cursor::new(&data));
            let pixels = decoder.decode().map_err(|e| e.to_string())?;
            let info = decoder.info().ok_or("Failed to get JPEG info")?;

            let rgb_buf = ImageBuffer::<Rgb<u8>, _>::from_raw(
                info.width as u32,
                info.height as u32,
                pixels
            ).ok_or("Failed to create image buffer")?;

            DynamicImage::ImageRgb8(rgb_buf)
        } else {
            image::load_from_memory(&data).map_err(|e| e.to_string())?
        };

        let img_orient = Self::map_exif_to_orientation(orientation_value);
        img.apply_orientation(img_orient);

        properties.width = img.width();
        properties.height = img.height();
        properties.bits = Some(img.color().bits_per_pixel() as u32);


        let processed_img = if let Some((w, h)) = size {
            img.resize_to_fill(w, h, FilterType::Nearest)
        } else {
            img
        };

        let rgba = processed_img.to_rgba8();
        let color_image = ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        );

        Ok((color_image, properties))
    }
}

use image::DynamicImage;
use std::future::IntoFuture;
use image::imageops::FilterType;
// 引入 IntoFuture 特征
use windows::{
    core::{Result},
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Security::Cryptography::CryptographicBuffer,
};

pub fn recognize_text(img: DynamicImage) -> std::result::Result<String, String> {
    recognize_text_internal(img).map_err(|e| e.to_string())
}

fn recognize_text_internal(img: DynamicImage) -> Result<String> {
    // 对图片进行灰度化和按比例放大
    let gray_img = img.grayscale();
    // 2. 放大 2~3 倍（使用双线性或 Lanczos3 算法，极大提升小字识别率）
    let scaled_img = gray_img.resize(
        gray_img.width() * 2,
        gray_img.height() * 2,
        FilterType::Lanczos3
    );

    let rgba_img = scaled_img.into_rgba8();
    let width = rgba_img.width() as i32;
    let height = rgba_img.height() as i32;
    let bytes = rgba_img.into_raw();

    let buffer = CryptographicBuffer::CreateFromByteArray(&bytes)?;

    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Rgba8,
        width,
        height,
    )?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;

    // ================= 关键修改在这里 =================
    let async_op = engine.RecognizeAsync(&bitmap)?;

    // 显式调用 .into_future() 将其转换为纯 Future，再交给 block_on
    let result = futures::executor::block_on(async_op.into_future())?;
    // ==================================================

    let text = result.Text()?.to_string();

    Ok(text)
}
use image::DynamicImage;
use image::imageops::FilterType;
use std::future::IntoFuture;
use windows::{
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Security::Cryptography::CryptographicBuffer,
    core::Result,
};

pub fn recognize_text_windows(img: DynamicImage) -> std::result::Result<String, String> {
    recognize_text_internal(img).map_err(|e| e.to_string())
}

fn recognize_text_internal(img: DynamicImage) -> Result<String> {
    let gray_img = img.grayscale();
    let scaled_img = gray_img.resize(
        gray_img.width() * 2,
        gray_img.height() * 2,
        FilterType::Lanczos3,
    );

    let rgba_img = scaled_img.into_rgba8();
    let width = rgba_img.width() as i32;
    let height = rgba_img.height() as i32;
    let bytes = rgba_img.into_raw();

    let buffer = CryptographicBuffer::CreateFromByteArray(&bytes)?;

    let bitmap =
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Rgba8, width, height)?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
    let async_op = engine.RecognizeAsync(&bitmap)?;
    let result = futures::executor::block_on(async_op.into_future())?;
    let text = result.Text()?.to_string();

    Ok(text)
}

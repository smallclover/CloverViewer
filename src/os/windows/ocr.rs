use crate::i18n::lang::Language;
use image::imageops::FilterType;
use image::{DynamicImage, GrayImage, ImageBuffer, Luma};
use std::future::IntoFuture;
use windows::{
    Globalization::Language as WinLanguage,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Security::Cryptography::CryptographicBuffer,
    core::HSTRING,
    core::Result,
};

pub fn recognize_text_windows(
    img: DynamicImage,
    language: Language,
) -> std::result::Result<String, String> {
    recognize_text_internal(img, language).map_err(|e| e.to_string())
}

fn recognize_text_internal(img: DynamicImage, language: Language) -> Result<String> {
    let preprocessed_img = preprocess_for_ocr(img);
    let rgba_img = preprocessed_img.into_rgba8();
    let width = rgba_img.width() as i32;
    let height = rgba_img.height() as i32;
    let bytes = rgba_img.into_raw();

    let buffer = CryptographicBuffer::CreateFromByteArray(&bytes)?;

    let bitmap =
        SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Rgba8, width, height)?;

    let mut best_score = f64::MIN;
    let mut best_text: Option<String> = None;

    let mut consider = |text: String, preferred: Option<Language>| {
        let score = score_text(&text, preferred);
        if score > best_score {
            best_score = score;
            best_text = Some(text);
        }
    };

    if let Ok(engine) = create_engine_for_ui_language(language) {
        if let Ok(text) = recognize_with_engine(&engine, &bitmap) {
            consider(text, Some(language));
        }
    }

    if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
        if let Ok(text) = recognize_with_engine(&engine, &bitmap) {
            consider(text, None);
        }
    }

    let others = match language {
        Language::Zh => [Language::En, Language::Ja],
        Language::En => [Language::Zh, Language::Ja],
        Language::Ja => [Language::Zh, Language::En],
    };
    for lang in others {
        if let Ok(engine) = create_engine_for_ui_language(lang) {
            if let Ok(text) = recognize_with_engine(&engine, &bitmap) {
                consider(text, Some(lang));
            }
        }
    }

    if let Some(text) = best_text {
        return Ok(text);
    }

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
    recognize_with_engine(&engine, &bitmap)
}

fn create_engine_for_ui_language(language: Language) -> Result<OcrEngine> {
    let tag = match language {
        Language::Zh => "zh-Hans",
        Language::En => "en",
        Language::Ja => "ja",
    };
    let lang = WinLanguage::CreateLanguage(&HSTRING::from(tag))?;
    OcrEngine::TryCreateFromLanguage(&lang)
}

fn recognize_with_engine(engine: &OcrEngine, bitmap: &SoftwareBitmap) -> Result<String> {
    let async_op = engine.RecognizeAsync(bitmap)?;
    let result = futures::executor::block_on(async_op.into_future())?;
    Ok(result.Text()?.to_string())
}

fn score_text(text: &str, preferred: Option<Language>) -> f64 {
    if is_text_quality_bad(text) {
        return f64::MIN;
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return f64::MIN;
    }

    let mut total = 0u32;
    let mut non_ws = 0u32;
    let mut ok_chars = 0u32;
    let mut control = 0u32;
    let mut replacement = 0u32;

    let mut ascii_alnum = 0u32;
    let mut cjk = 0u32;
    let mut hiragana = 0u32;
    let mut katakana = 0u32;

    let mut code_symbols = 0u32;
    let mut newlines = 0u32;

    for ch in text.chars() {
        total += 1;

        if ch == '\n' {
            newlines += 1;
        }

        if ch == '\u{FFFD}' {
            replacement += 1;
            continue;
        }

        if !ch.is_whitespace() {
            non_ws += 1;
        }

        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            control += 1;
        } else {
            ok_chars += 1;
        }

        if ch.is_ascii_alphanumeric() {
            ascii_alnum += 1;
        }

        if ch.is_ascii() {
            match ch {
                '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>' | ';' | ':' | ',' | '.' | '_'
                | '=' | '+' | '-' | '*' | '/' | '\\' | '|' | '&' | '!' | '?' | '\'' | '"' => {
                    code_symbols += 1;
                }
                _ => {}
            }
        }

        let u = ch as u32;
        if (0x4E00..=0x9FFF).contains(&u) {
            cjk += 1;
        } else if (0x3040..=0x309F).contains(&u) {
            hiragana += 1;
        } else if (0x30A0..=0x30FF).contains(&u) {
            katakana += 1;
        }
    }

    let total_f = total.max(1) as f64;
    let ok_ratio = ok_chars as f64 / total_f;
    let mut score = (non_ws as f64) * 1.2 + ok_ratio * 120.0;
    score -= (control as f64) * 25.0;
    score -= (replacement as f64) * 60.0;
    score += (newlines as f64) * 1.5;
    score += (code_symbols as f64) * 0.8;

    match preferred {
        Some(Language::Zh) => {
            score += (cjk as f64) * 2.0;
            score += (ascii_alnum as f64) * 0.6;
        }
        Some(Language::En) => {
            score += (ascii_alnum as f64) * 2.0;
            score += (code_symbols as f64) * 0.4;
        }
        Some(Language::Ja) => {
            score += ((hiragana + katakana) as f64) * 2.4;
            score += (cjk as f64) * 1.2;
            score += (ascii_alnum as f64) * 0.4;
        }
        None => {
            score += (ascii_alnum as f64) * 0.8;
            score += (cjk as f64) * 0.8;
            score += ((hiragana + katakana) as f64) * 0.8;
        }
    }

    score
}

fn is_text_quality_bad(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    let non_ws_len = trimmed.chars().filter(|c| !c.is_whitespace()).count();
    if non_ws_len < 4 {
        return true;
    }

    let mut total = 0u32;
    let mut control = 0u32;
    for ch in text.chars() {
        if ch == '\u{FFFD}' {
            return true;
        }
        total += 1;
        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            control += 1;
        }
    }
    if total > 0 && (control as f64 / total as f64) > 0.02 {
        return true;
    }

    false
}

fn preprocess_for_ocr(img: DynamicImage) -> DynamicImage {
    let mut gray = img.grayscale().into_luma8();

    if should_invert(&gray) {
        invert_in_place(&mut gray);
    }

    gray = auto_contrast(gray, 0.01, 0.99);

    let scale = choose_scale(gray.width(), gray.height());
    if scale > 1 {
        gray = image::imageops::resize(
            &gray,
            gray.width().saturating_mul(scale),
            gray.height().saturating_mul(scale),
            FilterType::Lanczos3,
        );
    }

    gray = unsharp_mask(gray, 0.6, 0.7);

    let threshold = otsu_threshold(&gray);
    gray = binarize(gray, threshold);

    DynamicImage::ImageLuma8(gray)
}

fn choose_scale(width: u32, height: u32) -> u32 {
    let min_dim = width.min(height);
    if min_dim < 500 || height < 260 { 3 } else { 2 }
}

fn should_invert(gray: &GrayImage) -> bool {
    let target_width = 256u32;
    let (w, h) = gray.dimensions();
    if w == 0 || h == 0 {
        return false;
    }

    let (scaled_w, scaled_h) = if w > target_width {
        let scaled_h = (h as f64 * target_width as f64 / w as f64).round().max(1.0) as u32;
        (target_width, scaled_h)
    } else {
        (w, h)
    };

    let preview = if (scaled_w, scaled_h) != (w, h) {
        image::imageops::resize(gray, scaled_w, scaled_h, FilterType::Nearest)
    } else {
        gray.clone()
    };

    let sum: u64 = preview.pixels().map(|p| p[0] as u64).sum();
    let mean = sum as f64 / (preview.width() as f64 * preview.height() as f64);
    mean < 120.0
}

fn invert_in_place(gray: &mut GrayImage) {
    for p in gray.pixels_mut() {
        p[0] = 255u8.saturating_sub(p[0]);
    }
}

fn auto_contrast(mut gray: GrayImage, low_pct: f32, high_pct: f32) -> GrayImage {
    let mut hist = [0u32; 256];
    for p in gray.pixels() {
        hist[p[0] as usize] += 1;
    }

    let total: u32 = hist.iter().sum();
    if total == 0 {
        return gray;
    }

    let low_target = (total as f64 * (low_pct as f64)).round() as u32;
    let high_target = (total as f64 * (high_pct as f64)).round() as u32;

    let mut cum = 0u32;
    let mut low = 0u8;
    for (i, c) in hist.iter().enumerate() {
        cum = cum.saturating_add(*c);
        if cum >= low_target {
            low = i as u8;
            break;
        }
    }

    cum = 0u32;
    let mut high = 255u8;
    for (i, c) in hist.iter().enumerate().rev() {
        cum = cum.saturating_add(*c);
        if total.saturating_sub(cum) <= high_target {
            high = i as u8;
            break;
        }
    }

    if high <= low.saturating_add(1) {
        return gray;
    }

    let low_f = low as f32;
    let inv_range = 255.0 / ((high as f32) - low_f);
    for p in gray.pixels_mut() {
        let v = p[0] as f32;
        let stretched = ((v - low_f) * inv_range).clamp(0.0, 255.0).round() as u8;
        p[0] = stretched;
    }

    gray
}

fn unsharp_mask(gray: GrayImage, sigma: f32, amount: f32) -> GrayImage {
    let (w, h) = gray.dimensions();
    if w == 0 || h == 0 {
        return gray;
    }

    let gray_f32: ImageBuffer<Luma<f32>, Vec<f32>> =
        ImageBuffer::from_fn(w, h, |x, y| Luma([gray.get_pixel(x, y)[0] as f32]));
    let blurred = imageproc::filter::gaussian_blur_f32(&gray_f32, sigma);

    let mut out = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let o = gray_f32.get_pixel(x, y)[0];
            let b = blurred.get_pixel(x, y)[0];
            let v = (o + amount * (o - b)).round().clamp(0.0, 255.0) as u8;
            out.put_pixel(x, y, Luma([v]));
        }
    }
    out
}

fn otsu_threshold(gray: &GrayImage) -> u8 {
    let mut hist = [0u32; 256];
    for p in gray.pixels() {
        hist[p[0] as usize] += 1;
    }

    let total: u64 = hist.iter().map(|&c| c as u64).sum();
    if total == 0 {
        return 127;
    }

    let mut sum_all = 0u64;
    for (i, &c) in hist.iter().enumerate() {
        sum_all += (i as u64) * (c as u64);
    }

    let mut sum_b = 0u64;
    let mut w_b = 0u64;
    let mut max_var = -1.0f64;
    let mut threshold = 127u8;

    for t in 0..256usize {
        w_b += hist[t] as u64;
        if w_b == 0 {
            continue;
        }
        let w_f = total - w_b;
        if w_f == 0 {
            break;
        }

        sum_b += (t as u64) * (hist[t] as u64);
        let m_b = sum_b as f64 / w_b as f64;
        let m_f = (sum_all - sum_b) as f64 / w_f as f64;

        let between = (w_b as f64) * (w_f as f64) * (m_b - m_f) * (m_b - m_f);
        if between > max_var {
            max_var = between;
            threshold = t as u8;
        }
    }

    threshold
}

fn binarize(mut gray: GrayImage, threshold: u8) -> GrayImage {
    for p in gray.pixels_mut() {
        p[0] = if p[0] >= threshold { 255 } else { 0 };
    }
    gray
}

#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use egui::ColorImage;
#[cfg(target_os = "windows")]
use windows::{
    core::{Interface},
    Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
    Win32::UI::Shell::{
        SHCreateItemFromParsingName, IShellItem, IShellItemImageFactory,
        SIIGBF_RESIZETOFIT, SIIGBF_BIGGERSIZEOK,
    },
    Win32::Graphics::Gdi::{GetObjectW, BITMAP, DeleteObject, HGDIOBJ},
    Win32::Foundation::SIZE,
    core::PCWSTR,
};

// Helper struct to ensure CoUninitialize is called
#[cfg(target_os = "windows")]
struct CoUninitializeOnDrop;

#[cfg(target_os = "windows")]
impl Drop for CoUninitializeOnDrop {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

#[cfg(target_os = "windows")]
pub fn load_thumbnail_windows(path: &PathBuf, size: (u32, u32)) -> Result<ColorImage, String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().map_err(|e| e.to_string())?;
        let _guard = CoUninitializeOnDrop;

        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let shell_item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None).map_err(|e| e.to_string())?;
        let image_factory: IShellItemImageFactory = shell_item.cast().map_err(|e| e.to_string())?;

        let size_struct = SIZE { cx: size.0 as i32, cy: size.1 as i32 };
        let hbitmap = image_factory.GetImage(size_struct, SIIGBF_RESIZETOFIT | SIIGBF_BIGGERSIZEOK).map_err(|e| e.to_string())?;

        let hgdiobj: HGDIOBJ = std::mem::transmute(hbitmap);

        let mut bitmap: BITMAP = std::mem::zeroed();
        if GetObjectW(hgdiobj, std::mem::size_of::<BITMAP>() as i32, Some(&mut bitmap as *mut _ as *mut _)) == 0 {
            let _ = DeleteObject(hgdiobj);
            return Err("Failed to get bitmap object".to_string());
        }

        let width = bitmap.bmWidth as usize;
        let height = bitmap.bmHeight as usize;
        let stride = bitmap.bmWidthBytes as usize;
        let bits_ptr = bitmap.bmBits as *const u8;

        if bits_ptr.is_null() {
            let _ = DeleteObject(hgdiobj);
             return Err("Bitmap bits are null".to_string());
        }

        let mut pixels = Vec::with_capacity(width * height);
        let bits = std::slice::from_raw_parts(bits_ptr, stride * height);

        for y in 0..height {
            for x in 0..width {
                let offset = y * stride + x * 4;
                if offset + 3 < bits.len() {
                    let b = bits[offset];
                    let g = bits[offset + 1];
                    let r = bits[offset + 2];
                    let a = bits[offset + 3];
                    pixels.push(egui::Color32::from_rgba_premultiplied(r, g, b, a));
                } else {
                     pixels.push(egui::Color32::BLACK);
                }
            }
        }

        let _ = DeleteObject(hgdiobj);

        Ok(ColorImage {
            size: [width, height],
            source_size: egui::vec2(width as f32, height as f32),
            pixels,
        })
    }
}
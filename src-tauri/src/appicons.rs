use std::path::Path;

use tauri::command;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HBITMAP, HDC,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

const ICON_SIDE: i32 = 32;
const BACKDROP_BGR: [u8; 3] = [26, 19, 13];
const BASE64_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[command]
pub async fn collect_app_icons(paths: Vec<String>) -> Result<Vec<Option<String>>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let apartment = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_ok();
        let rendered: Vec<Option<String>> = paths.iter().map(|path| render_icon(path)).collect();
        if apartment {
            unsafe { CoUninitialize() };
        }
        rendered
    })
    .await
    .map_err(|err| err.to_string())
}

fn render_icon(path: &str) -> Option<String> {
    if !Path::new(path).exists() {
        return None;
    }
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut info = SHFILEINFOW::default();
    let outcome = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if outcome == 0 || info.hIcon.is_invalid() {
        return None;
    }
    let pixels = icon_pixels(info.hIcon);
    unsafe {
        let _ = DestroyIcon(info.hIcon);
    }
    pixels.map(|bgra| bmp_data_uri(&bgra))
}

fn icon_pixels(icon: HICON) -> Option<Vec<u8>> {
    let mut details = ICONINFO::default();
    unsafe { GetIconInfo(icon, &mut details) }.ok()?;
    let color = details.hbmColor;
    let mask = details.hbmMask;
    let grab = read_bitmap(color);
    unsafe {
        if !color.is_invalid() {
            let _ = DeleteObject(color);
        }
        if !mask.is_invalid() {
            let _ = DeleteObject(mask);
        }
    }
    grab
}

fn read_bitmap(source: HBITMAP) -> Option<Vec<u8>> {
    if source.is_invalid() {
        return None;
    }
    let screen: HDC = unsafe { GetDC(HWND(0)) };
    if screen.is_invalid() {
        return None;
    }
    let mut header = BITMAPINFO::default();
    header.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    header.bmiHeader.biWidth = ICON_SIDE;
    header.bmiHeader.biHeight = -ICON_SIDE;
    header.bmiHeader.biPlanes = 1;
    header.bmiHeader.biBitCount = 32;
    header.bmiHeader.biCompression = BI_RGB.0 as u32;
    let mut bgra = vec![0u8; (ICON_SIDE * ICON_SIDE * 4) as usize];
    let copied = unsafe {
        GetDIBits(
            screen,
            source,
            0,
            ICON_SIDE as u32,
            Some(bgra.as_mut_ptr() as *mut core::ffi::c_void),
            &mut header,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        ReleaseDC(HWND(0), screen);
    }
    if copied == 0 {
        None
    } else {
        Some(bgra)
    }
}

fn bmp_data_uri(bgra: &[u8]) -> String {
    let side = ICON_SIDE as usize;
    let opaque = bgra.chunks_exact(4).all(|px| px[3] == 0);
    let stride = side * 3;
    let image_size = stride * side;
    let mut file: Vec<u8> = Vec::with_capacity(54 + image_size);
    file.extend_from_slice(b"BM");
    file.extend_from_slice(&((54 + image_size) as u32).to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&54u32.to_le_bytes());
    file.extend_from_slice(&40u32.to_le_bytes());
    file.extend_from_slice(&(side as i32).to_le_bytes());
    file.extend_from_slice(&(side as i32).to_le_bytes());
    file.extend_from_slice(&1u16.to_le_bytes());
    file.extend_from_slice(&24u16.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&(image_size as u32).to_le_bytes());
    file.extend_from_slice(&2835u32.to_le_bytes());
    file.extend_from_slice(&2835u32.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    file.extend_from_slice(&0u32.to_le_bytes());
    for row in (0..side).rev() {
        for col in 0..side {
            let at = (row * side + col) * 4;
            let blue = bgra[at] as u32;
            let green = bgra[at + 1] as u32;
            let red = bgra[at + 2] as u32;
            let alpha = if opaque { 255u32 } else { bgra[at + 3] as u32 };
            let inverse = 255 - alpha;
            file.push(((blue * alpha + BACKDROP_BGR[0] as u32 * inverse) / 255) as u8);
            file.push(((green * alpha + BACKDROP_BGR[1] as u32 * inverse) / 255) as u8);
            file.push(((red * alpha + BACKDROP_BGR[2] as u32 * inverse) / 255) as u8);
        }
    }
    format!("data:image/bmp;base64,{}", encode_base64(&file))
}

fn encode_base64(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let first = chunk[0] as u32;
        let second = *chunk.get(1).unwrap_or(&0) as u32;
        let third = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (first << 16) | (second << 8) | third;
        out.push(BASE64_TABLE[(triple >> 18 & 63) as usize] as char);
        out.push(BASE64_TABLE[(triple >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            BASE64_TABLE[(triple >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            BASE64_TABLE[(triple & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

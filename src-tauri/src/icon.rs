//! Bir .exe dosyasının ikonunu yüksek çözünürlükte PNG olarak çıkarır (Windows API).
//!
//! 1. `PrivateExtractIconsW`: exe'ye gömülü ikonu doğrudan 256 px ister (en keskin sonuç).
//! 2. Yedek: Shell'in `IShellItemImageFactory` arayüzü. Windows 10 1903+ sistem exe'leri ikonlarını
//!    `SystemResources\*.mun` dosyalarında tutar; düşük seviye API bunları göremez, Shell görür.
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum IconError {
    #[error("exe içinde ikon yok")]
    NoIcon,
    #[error("Windows API hatası: {0}")]
    Win(String),
    #[error("PNG yazılamadı: {0}")]
    Image(#[from] image::ImageError),
}

const SIZE: i32 = 256;

pub fn extract_png(exe: &Path, target: &Path) -> Result<(), IconError> {
    if !exe.is_file() {
        return Err(IconError::NoIcon);
    }
    let rgba = platform::extract_rgba(exe)?;
    let size = SIZE as u32;
    let img = image::RgbaImage::from_raw(size, size, rgba).ok_or(IconError::NoIcon)?;
    if img.pixels().all(|p| p[3] == 0) {
        return Err(IconError::NoIcon);
    }
    let img = crop_transparent(&img);
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(|e| IconError::Win(e.to_string()))?;
    }
    img.save_with_format(target, image::ImageFormat::Png)?;
    Ok(())
}

#[cfg(windows)]
mod platform {
    use super::{IconError, SIZE};
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::SIZE as WinSize;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER,
        BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
    };
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{
        IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyIcon, GetIconInfo, PrivateExtractIconsW, HICON, ICONINFO,
    };

    pub fn extract_rgba(exe: &Path) -> Result<Vec<u8>, IconError> {
        match from_embedded_icon(exe) {
            Ok(px) => Ok(px),
            Err(_) => from_shell(exe),
        }
    }

    /// 32 bpp bitmap'i yukarıdan aşağı BGRA olarak okur ve RGBA'ya çevirir.
    fn read_bitmap(bmp: HBITMAP, premultiplied: bool) -> Result<Vec<u8>, IconError> {
        // SAFETY: NULL'dan uyumlu bellek DC'si; aşağıda DeleteDC ile serbest bırakılır.
        let dc = unsafe { CreateCompatibleDC(None) };
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>()).unwrap_or(40),
                biWidth: SIZE,
                biHeight: -SIZE, // negatif = yukarıdan aşağı satır düzeni
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buf = vec![0u8; (SIZE * SIZE * 4) as usize];
        // SAFETY: buf SIZE×SIZE×4 bayt; bmi 32 bpp tanımlı; bmp geçerli.
        let lines = unsafe {
            GetDIBits(
                dc,
                bmp,
                0,
                SIZE as u32,
                Some(buf.as_mut_ptr().cast()),
                &mut bmi,
                DIB_RGB_COLORS,
            )
        };
        // SAFETY: yukarıda oluşturulan DC.
        unsafe {
            let _ = DeleteDC(dc);
        }
        if lines == 0 {
            return Err(IconError::Win("GetDIBits başarısız".into()));
        }
        let has_alpha = buf.as_chunks::<4>().0.iter().any(|p| p[3] != 0);
        for px in buf.as_chunks_mut::<4>().0 {
            px.swap(0, 2); // BGRA → RGBA
            if !has_alpha {
                px[3] = 255; // eski ikonlar: alfa yok → opak
            } else if premultiplied && px[3] > 0 && px[3] < 255 {
                let a = u32::from(px[3]);
                for c in &mut px[..3] {
                    *c = u8::try_from((u32::from(*c) * 255 / a).min(255)).unwrap_or(255);
                }
            }
        }
        Ok(buf)
    }

    fn from_embedded_icon(exe: &Path) -> Result<Vec<u8>, IconError> {
        // PrivateExtractIconsW sabit 260 karakterlik (MAX_PATH) bir arabellek bekler.
        let mut path = [0u16; 260];
        let wide: Vec<u16> = exe.as_os_str().encode_wide().collect();
        if wide.len() >= path.len() {
            return Err(IconError::Win("yol çok uzun".into()));
        }
        path[..wide.len()].copy_from_slice(&wide);
        let mut icons = [HICON::default(); 1];
        let mut ids = [0u32; 1];
        // SAFETY: arabellekler çağrı boyunca geçerli; dönen HICON DestroyIcon ile serbest bırakılır.
        let count = unsafe {
            PrivateExtractIconsW(
                &path,
                0,
                SIZE,
                SIZE,
                Some(&mut icons),
                Some(ids.as_mut_ptr()),
                0,
            )
        };
        if count == 0 || count == u32::MAX || icons[0].is_invalid() {
            return Err(IconError::NoIcon);
        }
        let hicon = icons[0];
        let mut info = ICONINFO::default();
        // SAFETY: geçerli HICON; ICONINFO bitmap'leri aşağıda silinir.
        let got = unsafe { GetIconInfo(hicon, &mut info) };
        let result = match got {
            Ok(()) if !info.hbmColor.is_invalid() => read_bitmap(info.hbmColor, false),
            Ok(()) => Err(IconError::NoIcon),
            Err(e) => Err(IconError::Win(e.to_string())),
        };
        // SAFETY: GetIconInfo'nun oluşturduğu bitmap'ler ve ikon çağırana aittir.
        unsafe {
            let _ = DeleteObject(HGDIOBJ(info.hbmColor.0));
            let _ = DeleteObject(HGDIOBJ(info.hbmMask.0));
            let _ = DestroyIcon(hicon);
        }
        result
    }

    fn from_shell(exe: &Path) -> Result<Vec<u8>, IconError> {
        let wide: Vec<u16> = exe
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: COM bu iş parçacığında başlatılır ve sonunda kapatılır (zaten açıksa S_FALSE döner).
        let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let result = (|| {
            // SAFETY: wide sıfır sonlu ve çağrı boyunca yaşıyor.
            let factory: IShellItemImageFactory =
                unsafe { SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None) }
                    .map_err(|e| IconError::Win(e.to_string()))?;
            // SAFETY: geçerli COM nesnesi; dönen HBITMAP aşağıda DeleteObject ile silinir.
            let bmp = unsafe {
                factory.GetImage(
                    WinSize { cx: SIZE, cy: SIZE },
                    SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
                )
            }
            .map_err(|e| IconError::Win(e.to_string()))?;
            let px = read_bitmap(bmp, true);
            // SAFETY: GetImage'ın döndürdüğü bitmap çağırana aittir.
            unsafe {
                let _ = DeleteObject(HGDIOBJ(bmp.0));
            }
            px
        })();
        if init.is_ok() {
            // SAFETY: yukarıdaki başarılı CoInitializeEx ile eşleşir.
            unsafe { CoUninitialize() };
        }
        result
    }
}

#[cfg(not(windows))]
mod platform {
    use super::IconError;
    use std::path::Path;
    pub fn extract_rgba(_exe: &Path) -> Result<Vec<u8>, IconError> {
        Err(IconError::NoIcon)
    }
}

/// Ölçeklenmiş küçük ikonların etrafındaki boş alanı kırpar (kare korunur).
fn crop_transparent(img: &image::RgbaImage) -> image::RgbaImage {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 8 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if x0 > x1 || y0 > y1 {
        return img.clone();
    }
    let side = (x1 - x0 + 1).max(y1 - y0 + 1).min(w).min(h);
    let cx = (x0 + x1) / 2;
    let cy = (y0 + y1) / 2;
    let left = cx.saturating_sub(side / 2).min(w - side);
    let top = cy.saturating_sub(side / 2).min(h - side);
    image::imageops::crop_imm(img, left, top, side, side).to_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn extracts_icon_from_real_system_exe() {
        // notepad.exe'nin ikonu Win10 1903+'da .mun dosyasındadır → Shell yedeği test edilir.
        let tmp = tempfile::tempdir().unwrap();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        for exe in ["notepad.exe", "regedit.exe"] {
            let target = tmp.path().join(format!("{exe}.png"));
            extract_png(&Path::new(&windir).join(exe), &target).unwrap();
            let img = image::open(&target).unwrap();
            assert!(img.width() >= 16 && img.width() == img.height(), "{exe}");
        }
    }

    #[test]
    fn missing_file_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(extract_png(&tmp.path().join("yok.exe"), &tmp.path().join("y.png")).is_err());
    }

    #[test]
    fn crop_keeps_square_around_content() {
        let mut img = image::RgbaImage::new(10, 10);
        img.put_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
        img.put_pixel(6, 5, image::Rgba([255, 0, 0, 255]));
        let c = crop_transparent(&img);
        assert_eq!(c.width(), c.height());
        assert_eq!(c.width(), 3);
    }
}

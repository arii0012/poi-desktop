//! Установка и восстановление обоев Windows.
//! Работает даже на неактивированной Windows.

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
};

const BACKUP_NAME: &str = "poi_desktop_backup_wallpaper.txt";

fn backup_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PoiDesktop")
        .join(BACKUP_NAME)
}

fn ensure_dir() {
    if let Some(p) = backup_path().parent() {
        let _ = fs::create_dir_all(p);
    }
}

/// Сохранить текущие обои
pub fn backup_current() -> Result<(), String> {
    ensure_dir();
    #[cfg(windows)]
    {
        use windows::core::w;
        use windows::Win32::System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ, REG_VALUE_TYPE,
        };

        unsafe {
            let mut hkey = std::mem::zeroed();
            if RegOpenKeyExW(
                HKEY_CURRENT_USER,
                w!("Control Panel\\Desktop"),
                0,
                KEY_READ,
                &mut hkey,
            )
            .is_err()
            {
                return Err("Не удалось открыть реестр".into());
            }

            let mut buf = [0u16; 512];
            let mut size = (buf.len() * 2) as u32;
            let mut ty = REG_VALUE_TYPE(0);
            let q = RegQueryValueExW(
                hkey,
                w!("Wallpaper"),
                None,
                Some(&mut ty),
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut size),
            );
            let _ = RegCloseKey(hkey);

            if q.is_ok() {
                let len = (size as usize / 2).saturating_sub(1);
                let path = String::from_utf16_lossy(&buf[..len]);
                fs::write(backup_path(), path.trim()).map_err(|e| e.to_string())?;
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = fs::write(backup_path(), "");
    }
    Ok(())
}

/// Вернуть предыдущие обои
pub fn restore_backup() -> Result<(), String> {
    let path = backup_path();
    if !path.exists() {
        return Err("Нет сохранённых обоев".into());
    }
    let old = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let old = old.trim();
    if old.is_empty() {
        return Err("Путь к старым обоям пустой".into());
    }
    set_wallpaper(Path::new(old))
}

/// Установить файл как обои
pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Файл не найден: {}", path.display()));
    }
    let abs = path.canonicalize().map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        let _ = backup_current();

        let wide: Vec<u16> = abs
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let result = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide.as_ptr() as *mut core::ffi::c_void),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
            if result.is_err() {
                return Err("SystemParametersInfoW failed".into());
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = abs;
        return Err("Установка обоев поддерживается только на Windows".into());
    }

    Ok(())
}

pub fn output_dir() -> PathBuf {
    let p = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PoiDesktop")
        .join("wallpapers");
    let _ = fs::create_dir_all(&p);
    p
}

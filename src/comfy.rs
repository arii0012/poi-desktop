//! Автоустановка ComfyUI из setup/ComfyUI-master.zip + pip + опциональная модель.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy, PartialEq)]
pub enum VramMode {
    Normal,
    Low,
    NoVram,
}

impl VramMode {
    pub fn label(self) -> &'static str {
        match self {
            VramMode::Normal => "Normal (12GB+)",
            VramMode::Low => "Low VRAM (--lowvram, 6–8GB)",
            VramMode::NoVram => "No VRAM (--novram)",
        }
    }
    pub fn args(self) -> Vec<&'static str> {
        match self {
            VramMode::Normal => vec![],
            VramMode::Low => vec!["--lowvram"],
            VramMode::NoVram => vec!["--novram"],
        }
    }
}

pub fn poi_ai_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PoiDesktop")
        .join("ai")
}

pub fn comfy_dir() -> PathBuf {
    poi_ai_dir().join("ComfyUI")
}

pub fn is_comfy_installed() -> bool {
    comfy_root().join("main.py").exists()
}

pub fn comfy_root() -> PathBuf {
    let d = comfy_dir();
    if d.join("main.py").exists() {
        d
    } else if d.join("ComfyUI").join("main.py").exists() {
        d.join("ComfyUI")
    } else {
        d
    }
}

pub fn models_dir() -> PathBuf {
    comfy_root().join("models").join("checkpoints")
}

pub fn list_models() -> Vec<String> {
    let mut out = Vec::new();
    let _ = fs::create_dir_all(models_dir());
    if let Ok(rd) = fs::read_dir(models_dir()) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".safetensors") || n.ends_with(".ckpt") || n.ends_with(".gguf") {
                out.push(n);
            }
        }
    }
    out
}

fn find_bundled_zip() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("setup/ComfyUI-master.zip"),
        PathBuf::from("./setup/ComfyUI-master.zip"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("setup/ComfyUI-master.zip")))
            .unwrap_or_default(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("ComfyUI-master.zip")))
            .unwrap_or_default(),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }
    None
}

/// Полная автоустановка: ZIP → папка → pip install
pub fn auto_install_all() -> Result<String, String> {
    let mut log = String::new();
    fs::create_dir_all(poi_ai_dir()).map_err(|e| e.to_string())?;
    fs::create_dir_all(models_dir()).map_err(|e| e.to_string())?;

    if is_comfy_installed() {
        log.push_str("ComfyUI уже установлен.\n");
    } else {
        let zip = find_bundled_zip().ok_or_else(|| {
            "Не найден setup/ComfyUI-master.zip рядом с exe. Положи ZIP в папку setup/".to_string()
        })?;
        log.push_str(&format!("Распаковка {} …\n", zip.display()));
        extract_zip_to_comfy(&zip)?;
        log.push_str("ComfyUI распакован.\n");
    }

    if !is_comfy_installed() {
        return Err(format!(
            "{log}После распаковки main.py не найден в {}",
            comfy_root().display()
        ));
    }

    log.push_str("pip install -r requirements.txt …\n");
    match pip_install_requirements() {
        Ok(m) => log.push_str(&m),
        Err(e) => log.push_str(&format!("pip: {e}\n")),
    }

    Ok(log)
}

fn extract_zip_to_comfy(zip_path: &Path) -> Result<(), String> {
    // Use PowerShell Expand-Archive on Windows, unzip on unix
    let dest_parent = poi_ai_dir();
    let tmp = dest_parent.join("_tmp_extract");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                    zip_path.display(),
                    tmp.display()
                ),
            ])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("Expand-Archive failed".into());
        }
    }
    #[cfg(not(windows))]
    {
        let status = Command::new("unzip")
            .args(["-q", "-o", zip_path.to_str().unwrap_or(""), "-d", tmp.to_str().unwrap_or("")])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("unzip failed".into());
        }
    }

    // Find folder with main.py
    let mut src = tmp.join("ComfyUI-master");
    if !src.join("main.py").exists() {
        // search one level
        if let Ok(rd) = fs::read_dir(&tmp) {
            for e in rd.flatten() {
                if e.path().join("main.py").exists() {
                    src = e.path();
                    break;
                }
            }
        }
    }
    if !src.join("main.py").exists() {
        return Err("main.py не найден в архиве".into());
    }

    let target = comfy_dir();
    let _ = fs::remove_dir_all(&target);
    // copy_dir recursive
    copy_dir_all(&src, &target).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&tmp);
    let _ = fs::create_dir_all(models_dir());
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

fn python_cmd() -> Result<&'static str, String> {
    if Command::new("python").arg("--version").output().is_ok() {
        Ok("python")
    } else if Command::new("py").arg("--version").output().is_ok() {
        Ok("py")
    } else if Command::new("python3").arg("--version").output().is_ok() {
        Ok("python3")
    } else {
        Err("Python не найден. Установи с python.org (галочка PATH).".into())
    }
}

pub fn pip_install_requirements() -> Result<String, String> {
    let root = comfy_root();
    let req = root.join("requirements.txt");
    if !req.exists() {
        return Err("requirements.txt не найден".into());
    }
    let py = python_cmd()?;
    let status = Command::new(py)
        .args(["-m", "pip", "install", "-r"])
        .arg(&req)
        .current_dir(&root)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok("pip OK\n".into())
    } else {
        Err("pip завершился с ошибкой (смотри консоль)".into())
    }
}

pub fn launch_comfy(mode: VramMode) -> Result<String, String> {
    let root = comfy_root();
    if !root.join("main.py").exists() {
        return Err("ComfyUI не установлен. Нажми «Автоустановка AI».".into());
    }
    let py = python_cmd()?;
    let mut args: Vec<String> = vec![
        "main.py".to_string(),
        "--listen".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        "8188".to_string(),
    ];
    for a in mode.args() {
        args.push(a.to_string());
    }
    Command::new(py)
        .args(&args)
        .current_dir(&root)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(format!("ComfyUI: http://127.0.0.1:8188 · {}", mode.label()))
}

/// SD 1.5 (~4GB) — публичный файл, проще для авто-загрузки чем Flux 20GB
pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/Comfy-Org/stable-diffusion-v1-5-archive/resolve/main/v1-5-pruned-emaonly.safetensors";
pub const DEFAULT_MODEL_NAME: &str = "v1-5-pruned-emaonly.safetensors";

pub fn download_model(url: &str, filename: &str) -> Result<String, String> {
    let url = if url.trim().is_empty() {
        DEFAULT_MODEL_URL
    } else {
        url.trim()
    };
    let name = if filename.trim().is_empty() {
        Path::new(url)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(DEFAULT_MODEL_NAME)
            .to_string()
    } else {
        filename.trim().to_string()
    };
    let dir = models_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(&name);
    if dest.exists() {
        return Ok(format!("Модель уже есть: {}", dest.display()));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600 * 12))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| e.to_string())?;

    let mut resp = client
        .get(url)
        .header("User-Agent", "PoiDesktop/4.5")
        .send()
        .map_err(|e| format!("Сеть: {e}"))?
        .error_for_status()
        .map_err(|e| format!("HTTP: {e}"))?;

    let mut file = File::create(&dest).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut buf = [0u8; 512 * 1024];
    loop {
        let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        total += n as u64;
    }
    Ok(format!(
        "Скачано {:.1} МБ → {}",
        total as f64 / 1_000_000.0,
        dest.display()
    ))
}

pub fn install_comfy_git() -> Result<String, String> {
    // fallback if no zip
    auto_install_all()
}

fn open_url(url: &str) {
    #[cfg(windows)]
    {
        let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}

pub fn open_comfy_github() {
    open_url("https://github.com/comfyanonymous/ComfyUI");
}
pub fn open_flux_models_page() {
    open_url("https://huggingface.co/black-forest-labs/FLUX.1-dev");
}
pub fn open_quant_models_page() {
    open_url("https://huggingface.co/models?search=flux+gguf");
}

pub fn write_setup_readme() -> Result<PathBuf, String> {
    let p = poi_ai_dir().join("README_SETUP.txt");
    fs::create_dir_all(poi_ai_dir()).map_err(|e| e.to_string())?;
    fs::write(&p, "См. кнопку Автоустановка AI в PoiDesktop.\n").map_err(|e| e.to_string())?;
    Ok(p)
}

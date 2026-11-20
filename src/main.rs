// PoiDesktop 5.4 — один файл: фигуры, сцены/«шейдеры», анимации, виджеты, обои
// CSS здесь НЕ используется (это не браузер). Эффекты — в коде ниже.

#![allow(dead_code)]

use eframe::egui;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ===================== SHAPES =====================
const COLS: usize = 40;
const ROWS: usize = 48;
const SHAPE_NAMES: &[&str] = &[
    "Шар", "Сердце", "Звезда", "Планета", "Ракета", "Луна", "Котик", "Молния", "Алмаз", "Огонь",
    "Череп", "Глаз", "НЛО", "Капля",
];
const CODE_CHARS: &[char] = &['0','1','{','}','<','>','/','*','#','@','x','o','.',':','A','B','X','Y'];

struct Mask { on: Vec<bool>, depth: Vec<f32> }

fn idx(x: usize, y: usize) -> usize { y * COLS + x }

fn fill_circle(on: &mut [bool], depth: &mut [f32], cx: f32, cy: f32, r: f32) {
    let r2 = r * r;
    for y in 0..ROWS {
        for x in 0..COLS {
            let px = x as f32 / COLS as f32;
            let py = y as f32 / ROWS as f32;
            let d2 = (px - cx).powi(2) + (py - cy).powi(2);
            if d2 <= r2 {
                let i = idx(x, y);
                on[i] = true;
                depth[i] = depth[i].max(1.0 - d2.sqrt() / r);
            }
        }
    }
}
fn fill_ellipse(on: &mut [bool], depth: &mut [f32], cx: f32, cy: f32, rx: f32, ry: f32) {
    for y in 0..ROWS {
        for x in 0..COLS {
            let px = (x as f32 / COLS as f32 - cx) / rx.max(0.01);
            let py = (y as f32 / ROWS as f32 - cy) / ry.max(0.01);
            let d2 = px * px + py * py;
            if d2 <= 1.0 {
                let i = idx(x, y);
                on[i] = true;
                depth[i] = depth[i].max(1.0 - d2.sqrt());
            }
        }
    }
}
fn fill_poly(on: &mut [bool], depth: &mut [f32], pts: &[[f32; 2]]) {
    if pts.len() < 3 { return; }
    let (mut cx, mut cy) = (0.0_f32, 0.0_f32);
    for p in pts { cx += p[0]; cy += p[1]; }
    cx /= pts.len() as f32; cy /= pts.len() as f32;
    for y in 0..ROWS {
        for x in 0..COLS {
            let px = x as f32 / COLS as f32;
            let py = y as f32 / ROWS as f32;
            let mut inside = false;
            let n = pts.len();
            let mut j = n - 1;
            for i in 0..n {
                let (xi, yi) = (pts[i][0], pts[i][1]);
                let (xj, yj) = (pts[j][0], pts[j][1]);
                if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi + 1e-6) + xi) {
                    inside = !inside;
                }
                j = i;
            }
            if inside {
                let i = idx(x, y);
                on[i] = true;
                depth[i] = depth[i].max(1.0 - ((px - cx).hypot(py - cy) * 2.0).min(1.0));
            }
        }
    }
}

fn generate_mask(shape: usize) -> Mask {
    use std::f32::consts::PI;
    let mut on = vec![false; COLS * ROWS];
    let mut depth = vec![0.0; COLS * ROWS];
    match shape {
        1 => {
            fill_circle(&mut on, &mut depth, 0.35, 0.38, 0.17);
            fill_circle(&mut on, &mut depth, 0.65, 0.38, 0.17);
            fill_poly(&mut on, &mut depth, &[[0.18, 0.42], [0.82, 0.42], [0.50, 0.88]]);
        }
        2 => {
            let mut pts = Vec::new();
            for i in 0..10 {
                let a = -PI / 2.0 + i as f32 * PI / 5.0;
                let r = if i % 2 == 0 { 0.40 } else { 0.16 };
                pts.push([0.5 + r * a.cos(), 0.5 + r * a.sin()]);
            }
            fill_poly(&mut on, &mut depth, &pts);
        }
        3 => {
            fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.26);
            fill_ellipse(&mut on, &mut depth, 0.5, 0.5, 0.44, 0.11);
            fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.26);
        }
        4 => {
            fill_poly(&mut on, &mut depth, &[[0.50,0.08],[0.64,0.30],[0.64,0.62],[0.50,0.90],[0.36,0.62],[0.36,0.30]]);
            fill_poly(&mut on, &mut depth, &[[0.36,0.58],[0.20,0.78],[0.36,0.70]]);
            fill_poly(&mut on, &mut depth, &[[0.64,0.58],[0.80,0.78],[0.64,0.70]]);
        }
        5 => fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.34),
        6 => {
            fill_circle(&mut on, &mut depth, 0.5, 0.55, 0.26);
            fill_poly(&mut on, &mut depth, &[[0.30,0.40],[0.25,0.16],[0.42,0.38]]);
            fill_poly(&mut on, &mut depth, &[[0.70,0.40],[0.75,0.16],[0.58,0.38]]);
        }
        7 => fill_poly(&mut on, &mut depth, &[[0.55,0.08],[0.30,0.48],[0.48,0.48],[0.38,0.92],[0.72,0.42],[0.52,0.42]]),
        8 => fill_poly(&mut on, &mut depth, &[[0.50,0.10],[0.80,0.40],[0.50,0.90],[0.20,0.40]]),
        9 => {
            for i in 0..10 {
                let t = i as f32 / 9.0;
                fill_ellipse(&mut on, &mut depth, 0.5 + 0.03*(t*5.0).sin(), 0.85 - t*0.55, 0.08+t*0.10, 0.05);
            }
        }
        10 => {
            fill_circle(&mut on, &mut depth, 0.5, 0.38, 0.22);
            fill_ellipse(&mut on, &mut depth, 0.5, 0.62, 0.16, 0.14);
        }
        11 => {
            fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.30);
            fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.12);
        }
        12 => {
            fill_ellipse(&mut on, &mut depth, 0.5, 0.48, 0.34, 0.12);
            fill_ellipse(&mut on, &mut depth, 0.5, 0.40, 0.16, 0.09);
        }
        13 => fill_ellipse(&mut on, &mut depth, 0.5, 0.45, 0.16, 0.28),
        _ => fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.36),
    }
    if !on.iter().any(|&b| b) {
        fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.3);
    }
    Mask { on, depth }
}

// ===================== WALLPAPER WIN =====================
mod wallpaper {
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

}
use wallpaper::{output_dir, restore_backup, set_wallpaper};

// ===================== APP =====================

const NAV: &[&str] = &["Обзор", "Фигуры", "Вид", "Шейдеры", "Виджеты", "Обои"];
const ANIM: &[&str] = &["Пульс", "Вращение", "Парение", "Волна", "Glow", "Orbit", "Статично"];
const SHADERS: &[&str] = &[
    "Нет", "Звёзды", "Plasma", "Matrix", "Aurora", "Ripple", "Neon Fog", "Cyber Grid",
    "Vaporwave", "Scanlines", "Tunnel", "Ember",
];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("PoiDesktop"),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("PoiDesktop", options, Box::new(|cc| {
        let mut v = egui::Visuals::dark();
        v.panel_fill = egui::Color32::from_rgb(14, 16, 22);
        cc.egui_ctx.set_visuals(v);
        Box::new(App::new())
    }))
}

struct App {
    tab: usize,
    shape: usize,
    color: egui::Color32,
    accent: egui::Color32,
    bg: egui::Color32,
    shader: usize,
    anim: usize,
    speed: f32,
    scale: f32,
    glow: f32,
    density: f32,
    time: Instant,
    status: String,
    mask: Mask,
    show_clock: bool,
    show_date: bool,
    show_text: bool,
    w_text: String,
    w_size: f32,
    w_y: f32,
    stars: Vec<[f32; 3]>,
}

impl App {
    fn new() -> Self {
        let shape = 3;
        Self {
            tab: 0,
            shape,
            color: egui::Color32::from_rgb(80, 255, 180),
            accent: egui::Color32::from_rgb(120, 160, 255),
            bg: egui::Color32::from_rgb(6, 8, 14),
            shader: 1,
            anim: 0,
            speed: 1.0,
            scale: 1.2,
            glow: 1.5,
            density: 1.0,
            time: Instant::now(),
            status: "PoiDesktop · фигуры + шейдеры + обои".into(),
            mask: generate_mask(shape),
            show_clock: true,
            show_date: true,
            show_text: false,
            w_text: "Poi".into(),
            w_size: 40.0,
            w_y: 0.08,
            stars: (0..160).map(|i| [
                ((i * 47) % 100) as f32 / 100.0,
                ((i * 31) % 100) as f32 / 100.0,
                0.3 + ((i * 13) % 70) as f32 / 100.0,
            ]).collect(),
        }
    }

    fn t(&self) -> f32 { self.time.elapsed().as_secs_f32() * self.speed }

    fn set_shape(&mut self, s: usize) {
        self.shape = s;
        self.mask = generate_mask(s);
    }

    fn preset(&mut self, id: usize) {
        match id {
            0 => { self.set_shape(3); self.color = egui::Color32::from_rgb(100,230,255); self.shader = 1; self.anim = 1; }
            1 => { self.set_shape(1); self.color = egui::Color32::from_rgb(255,80,140); self.shader = 2; self.anim = 0; }
            2 => { self.set_shape(0); self.color = egui::Color32::from_rgb(40,255,100); self.shader = 3; self.anim = 3; }
            3 => { self.set_shape(4); self.color = egui::Color32::from_rgb(255,160,50); self.shader = 1; self.anim = 2; }
            4 => { self.set_shape(5); self.color = egui::Color32::from_rgb(200,210,255); self.shader = 1; }
            5 => { self.set_shape(6); self.color = egui::Color32::from_rgb(255,200,140); self.shader = 2; }
            6 => { self.set_shape(2); self.color = egui::Color32::from_rgb(255,220,80); self.shader = 5; self.anim = 4; }
            _ => { self.set_shape(7); self.color = egui::Color32::from_rgb(100,200,255); self.shader = 7; self.anim = 5; }
        }
        self.status = format!("Пресет → {}", SHAPE_NAMES[self.shape]);
    }

    fn pos(&self, x: usize, y: usize, ox: f32, oy: f32, cell: f32, t: f32) -> (f32, f32) {
        let mut px = ox + x as f32 * cell + cell * 0.5;
        let mut py = oy + y as f32 * cell + cell * 0.5;
        let cx = ox + COLS as f32 * cell * 0.5;
        let cy = oy + ROWS as f32 * cell * 0.5;
        let dx = px - cx;
        let dy = py - cy;
        match self.anim {
            0 => { let s = 1.0 + 0.07 * (t * 2.2).sin(); px = cx + dx * s; py = cy + dy * s; }
            1 => { let a = t * 0.5; px = cx + dx * a.cos() - dy * a.sin(); py = cy + dx * a.sin() + dy * a.cos(); }
            2 => { py += (t * 1.2).sin() * cell * 1.2; }
            3 => { py += (t * 2.0 + x as f32 * 0.3).sin() * cell * 0.9; }
            4 => { let s = 1.0 + 0.05 * (t * 1.8).sin(); px = cx + dx * s; py = cy + dy * s; }
            5 => {
                let a = t * 0.32;
                px = cx + dx * a.cos() - dy * a.sin();
                py = cy + dx * a.sin() + dy * a.cos() + (t * 0.9).sin() * cell * 0.4;
            }
            _ => {}
        }
        (px, py)
    }

    fn col(&self, depth: f32, t: f32) -> egui::Color32 {
        let l = 0.55 + 0.45 * depth;
        let br = if self.anim == 4 { 1.0 + 0.12 * (t * 1.8).sin() } else { 1.0 };
        let r = (self.color.r() as f32 * l * br).min(255.0) as u8;
        let g = (self.color.g() as f32 * l * br).min(255.0) as u8;
        let b = (self.color.b() as f32 * l * br).min(255.0) as u8;
        egui::Color32::from_rgb(r, g, b)
    }

    fn apply_wallpaper(&mut self) {
        let path = output_dir().join("poi_current.png");
        match self.render_png(&path, 1920, 1080) {
            Ok(()) => match set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Обои: {e}"),
            },
            Err(e) => self.status = format!("Рендер: {e}"),
        }
    }

    fn render_png(&self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba, RgbaImage};
        let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([self.bg.r(), self.bg.g(), self.bg.b(), 255]));
        let t = if self.anim == 6 { 0.0 } else { self.t() };
        // simple stars for export
        for s in &self.stars {
            let x = (s[0] * w as f32) as u32 % w;
            let y = (s[1] * h as f32) as u32 % h;
            img.put_pixel(x, y, Rgba([170, 180, 200, 255]));
        }
        let cell = (h as f32 * 0.014 * self.scale).max(8.0);
        let ox = w as f32 * 0.5 - COLS as f32 * cell * 0.5;
        let oy = h as f32 * 0.48 - ROWS as f32 * cell * 0.5;
        for y in 0..ROWS {
            for x in 0..COLS {
                let i = y * COLS + x;
                if !self.mask.on[i] { continue; }
                if self.density < 0.99 {
                    let st = (1.0 / self.density.max(0.3)) as usize;
                    if (x + y) % st.max(1) != 0 { continue; }
                }
                let depth = self.mask.depth[i];
                let (px, py) = self.pos(x, y, ox, oy, cell, t);
                let c = self.col(depth, t);
                let rad = cell * 0.55;
                let r0 = ((rad * (1.0 + self.glow * 0.5)) as i32).max(2);
                for dy in -r0..=r0 {
                    for dx in -r0..=r0 {
                        let d = ((dx * dx + dy * dy) as f32).sqrt() / rad;
                        if d > 1.0 + self.glow * 0.4 { continue; }
                        let a = ((1.0 - d / (1.0 + self.glow * 0.4)).max(0.0).powf(1.5) * (0.5 + depth * 0.5)).clamp(0.0, 1.0);
                        let xx = px as i32 + dx;
                        let yy = py as i32 + dy;
                        if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 { continue; }
                        let p = img.get_pixel_mut(xx as u32, yy as u32);
                        p.0[0] = ((1.0 - a) * p.0[0] as f32 + a * c.r() as f32) as u8;
                        p.0[1] = ((1.0 - a) * p.0[1] as f32 + a * c.g() as f32) as u8;
                        p.0[2] = ((1.0 - a) * p.0[2] as f32 + a * c.b() as f32) as u8;
                    }
                }
            }
        }
        img.save(path).map_err(|e| e.to_string())
    }

    fn draw_shader(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        let name = SHADERS.get(self.shader).copied().unwrap_or("Нет");
        if name == "Нет" { return; }
        let c = self.accent;

        if name == "Звёзды" || name == "Ember" || name == "Tunnel" {
            for s in &self.stars {
                let y = (s[1] + t * 0.01 * s[2]) % 1.0;
                let col = if name == "Ember" {
                    egui::Color32::from_rgb(255, 160, 60)
                } else {
                    egui::Color32::from_gray((140.0 + 100.0 * s[2]) as u8)
                };
                painter.circle_filled(
                    egui::pos2(rect.left() + s[0] * rect.width(), rect.top() + y * rect.height()),
                    1.0 + s[2],
                    col,
                );
            }
        }
        if name == "Plasma" || name == "Neon Fog" || name == "Vaporwave" {
            let step = 12.0;
            let mut y = rect.top();
            while y < rect.bottom() {
                let mut x = rect.left();
                while x < rect.right() {
                    let nx = (x - rect.left()) / rect.width();
                    let ny = (y - rect.top()) / rect.height();
                    let v = ((nx * 5.0 + t).sin() + (ny * 6.0 - t * 0.6).cos()) * 0.25 + 0.5;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(step, step)),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (v * 55.0) as u8),
                    );
                    x += step;
                }
                y += step;
            }
        }
        if name == "Matrix" {
            let cols = 22;
            let cw = rect.width() / cols as f32;
            for i in 0..cols {
                let head = ((t * (0.4 + i as f32 * 0.03) + i as f32 * 0.2) % 1.3) - 0.15;
                for k in 0..12 {
                    let yy = head - k as f32 * 0.04;
                    if !(0.0..=1.0).contains(&yy) { continue; }
                    let ch = CODE_CHARS[(i * 3 + k) % CODE_CHARS.len()];
                    painter.text(
                        egui::pos2(rect.left() + i as f32 * cw, rect.top() + yy * rect.height()),
                        egui::Align2::LEFT_TOP, ch, egui::FontId::monospace(11.0),
                        egui::Color32::from_rgba_unmultiplied(self.color.r(), self.color.g(), self.color.b(), (160 - k * 10) as u8),
                    );
                }
            }
        }
        if name == "Aurora" {
            for band in 0..5 {
                let base = 0.25 + band as f32 * 0.1;
                for i in 0..40 {
                    let u = i as f32 / 40.0;
                    let x = rect.left() + u * rect.width();
                    let y = rect.top() + rect.height() * (base + 0.06 * ((u * 8.0 + t + band as f32).sin()));
                    painter.circle_filled(egui::pos2(x, y), 3.0,
                        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 35));
                }
            }
        }
        if name == "Ripple" {
            for i in 0..12 {
                let phase = t * 0.55 + i as f32 * 0.4;
                let rad = (phase % 1.0) * rect.height() * 0.45;
                let a = ((1.0 - phase % 1.0) * 70.0) as u8;
                painter.circle_stroke(rect.center(), rad, egui::Stroke::new(2.0_f32,
                    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)));
            }
        }
        if name == "Cyber Grid" {
            let step = 36.0;
            let mut x = rect.left();
            while x < rect.right() {
                painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 30)));
                x += step;
            }
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 30)));
                y += step;
            }
        }
        if name == "Scanlines" {
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_black_alpha(45)));
                y += 3.0;
            }
        }
        if name == "Tunnel" {
            let (cx, cy) = (rect.center().x, rect.center().y);
            for i in 0..24 {
                let a = i as f32 / 24.0 * std::f32::consts::TAU + t * 0.2;
                let len = rect.height() * 0.4 * (0.3 + 0.7 * ((t * 2.0 + i as f32).sin().abs()));
                painter.line_segment(
                    [egui::pos2(cx, cy), egui::pos2(cx + a.cos() * len, cy + a.sin() * len)],
                    egui::Stroke::new(1.5_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 50)),
                );
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.anim != 6 { ctx.request_repaint(); }

        egui::TopBottomPanel::bottom("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, n) in NAV.iter().enumerate() {
                    if ui.selectable_label(self.tab == i, *n).clicked() { self.tab = i; }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Установить обои").clicked() { self.apply_wallpaper(); }
                });
            });
            ui.label(&self.status);
        });

        egui::SidePanel::left("side").default_width(320.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                0 => {
                    ui.heading("PoiDesktop");
                    ui.label("Пресеты");
                    ui.horizontal_wrapped(|ui| {
                        for (n, id) in [("Планета",0usize),("Сердце",1),("Matrix",2),("Ракета",3),("Луна",4),("Котик",5),("Звезда",6),("Молния",7)] {
                            if ui.button(n).clicked() { self.preset(id); }
                        }
                    });
                    if ui.button("✨ Установить обои").clicked() { self.apply_wallpaper(); }
                    if ui.button("↩ Вернуть старые обои").clicked() {
                        match restore_backup() {
                            Ok(()) => self.status = "↩ Обои возвращены".into(),
                            Err(e) => self.status = e,
                        }
                    }
                    ui.label(egui::RichText::new("GLSL/wgpu — отдельный GPU-движок. Здесь пресеты эффектов во вкладке Шейдеры.").small().weak());
                }
                1 => {
                    ui.label("Фигуры");
                    for (i, n) in SHAPE_NAMES.iter().enumerate() {
                        if ui.selectable_label(self.shape == i, *n).clicked() { self.set_shape(i); }
                    }
                }
                2 => {
                    ui.label("Анимация");
                    for (i, n) in ANIM.iter().enumerate() {
                        if ui.selectable_label(self.anim == i, *n).clicked() { self.anim = i; }
                    }
                    ui.add(egui::Slider::new(&mut self.speed, 0.2..=2.5).text("Скорость"));
                    ui.add(egui::Slider::new(&mut self.scale, 0.7..=1.8).text("Масштаб"));
                    ui.add(egui::Slider::new(&mut self.glow, 0.0..=2.5).text("Свечение"));
                    ui.add(egui::Slider::new(&mut self.density, 0.3..=1.0).text("Плотность"));
                    ui.horizontal(|ui| {
                        ui.label("Цвет"); ui.color_edit_button_srgba(&mut self.color);
                        ui.label("Акцент"); ui.color_edit_button_srgba(&mut self.accent);
                        ui.label("Фон"); ui.color_edit_button_srgba(&mut self.bg);
                    });
                }
                3 => {
                    ui.label(egui::RichText::new("Шейдеры / сцены").strong());
                    ui.label("Эффекты фона (не GLSL-файлы, а пресеты в коде)");
                    for (i, n) in SHADERS.iter().enumerate() {
                        if ui.selectable_label(self.shader == i, *n).clicked() {
                            self.shader = i;
                            self.status = format!("Шейдер: {n}");
                        }
                    }
                }
                4 => {
                    ui.checkbox(&mut self.show_clock, "Часы");
                    ui.checkbox(&mut self.show_date, "Дата");
                    ui.checkbox(&mut self.show_text, "Текст");
                    if self.show_text { ui.text_edit_singleline(&mut self.w_text); }
                    ui.add(egui::Slider::new(&mut self.w_size, 18.0..=64.0).text("Размер"));
                    ui.add(egui::Slider::new(&mut self.w_y, 0.02..=0.4).text("Позиция Y"));
                }
                _ => {
                    if ui.button("Установить обои").clicked() { self.apply_wallpaper(); }
                    if ui.button("↩ Вернуть старые").clicked() {
                        match restore_backup() {
                            Ok(()) => self.status = "↩ Ок".into(),
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button("Сохранить PNG").clicked() {
                        let p = output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%H%M%S")));
                        match self.render_png(&p, 1920, 1080) {
                            Ok(()) => self.status = format!("{}", p.display()),
                            Err(e) => self.status = e,
                        }
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let t = if self.anim == 6 { 0.0 } else { self.t() };

            painter.rect_filled(rect, 0.0, self.bg);
            self.draw_shader(&painter, rect, t);

            let cell = (rect.height().min(rect.width()) * 0.016 * self.scale).max(10.0);
            let ox = rect.center().x - COLS as f32 * cell * 0.5;
            let oy = rect.center().y - ROWS as f32 * cell * 0.5;
            let font = egui::FontId::monospace((cell * 0.85).clamp(10.0, 20.0));
            let mut drawn = 0u32;

            for y in 0..ROWS {
                for x in 0..COLS {
                    let i = y * COLS + x;
                    if !self.mask.on[i] { continue; }
                    if self.density < 0.99 {
                        let st = (1.0 / self.density.max(0.3)) as usize;
                        if (x + y) % st.max(1) != 0 { continue; }
                    }
                    let depth = self.mask.depth[i];
                    let (px, py) = self.pos(x, y, ox, oy, cell, t);
                    let c = self.col(depth, t);
                    let pos = egui::pos2(px, py);
                    let rad = (cell * 0.48).max(3.0);
                    if self.glow > 0.2 {
                        painter.circle_filled(pos, rad * (1.8 + self.glow * 0.3),
                            egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 40));
                    }
                    painter.circle_filled(pos, rad, c);
                    let ch = CODE_CHARS[(x + y * 3) % CODE_CHARS.len()];
                    painter.text(pos, egui::Align2::CENTER_CENTER, ch, font.clone(), egui::Color32::WHITE);
                    drawn += 1;
                }
            }

            painter.text(
                rect.left_top() + egui::vec2(12.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("{} · {} · точек:{} · {}", SHAPE_NAMES[self.shape], SHADERS[self.shader], drawn, ANIM[self.anim]),
                egui::FontId::proportional(13.0),
                egui::Color32::from_rgb(180, 255, 210),
            );

            let wy = rect.top() + rect.height() * self.w_y;
            if self.show_clock {
                painter.text(egui::pos2(rect.center().x, wy), egui::Align2::CENTER_TOP,
                    chrono::Local::now().format("%H:%M:%S").to_string(),
                    egui::FontId::monospace(self.w_size),
                    egui::Color32::from_rgba_unmultiplied(230, 255, 240, 230));
            }
            if self.show_date {
                painter.text(egui::pos2(rect.center().x, wy + self.w_size + 4.0), egui::Align2::CENTER_TOP,
                    chrono::Local::now().format("%d.%m.%Y").to_string(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgba_unmultiplied(200, 220, 230, 160));
            }
            if self.show_text {
                painter.text(egui::pos2(rect.center().x, rect.bottom() - 32.0), egui::Align2::CENTER_BOTTOM,
                    &self.w_text, egui::FontId::proportional(18.0), egui::Color32::WHITE);
            }
        });
    }
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shapes;
mod wallpaper;
mod config;
mod comfy;

use config::PoiCfg;
use eframe::egui;
use shapes::{code_variants, generate_mask, mask_from_image, CODE_CHARS, Mask, SHAPE_NAMES, COLS};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([1024.0, 700.0])
            .with_title("PoiDesktop"),
        ..Default::default()
    };
    eframe::run_native("PoiDesktop", options, Box::new(|cc| Box::new(App::new(cc))))
}

// ---- catalogs ----
const RENDER_STYLES: &[&str] = &[
    "Хакерский (код)", "Пиксельный", "Обычный", "Нейронный", "Каркас", "Неон-линии", "Точки", "ASCII",
];
const SHADER_PRESETS: &[&str] = &[
    "Нет",
    // classic generative
    "Plasma", "Plasma 2", "FBM Noise", "Domain Warp", "Flow Field",
    "Matrix", "Matrix Blue", "Rain", "Digital Rain", "Codefall",
    "Aurora", "Aurora Borealis", "Northern Lights",
    "Ripple", "Ripple Multi", "Pulse Rings", "Shockwave",
    "Aura", "Aura Soft", "Glow Bloom",
    "Neon Fog", "Fog Layer", "Mist",
    "Cyber Grid", "Hex Field", "Tri Grid", "Perspective Grid",
    "Starfield", "Deep Space", "Galaxy Spiral", "Nebula",
    "Ember", "Fireflies", "Sparks", "Lava Lamp",
    "Ocean", "Waves", "Caustics",
    "Vaporwave", "Synthwave Sun", "Retro Grid",
    "Glitch FX", "RGB Split", "Scanlines", "CRT",
    "Hologram", "Wire Holo", "Prism", "Chromatic",
    "Smoke", "Cloud Noise", "Ink",
    "Snow", "Blizzard", "Ash",
    "DNA Helix", "Tunnel", "Warp Speed", "Hyperspace",
    "Orbital", "Crystal", "Void", "Black Hole",
    "Checker Warp", "Mandala", "Kaleidoscope",
    "Lightning", "Energy Field", "Force Field",
    "Pixel Sort", "Dither", "Bayer",
    "Gradient Mesh", "Color Bands", "Hue Rotate",
];
const GEN_STYLES: &[&str] = &[
    "4K фото-обои", "Aura", "Plasma", "Пиксель", "Неон", "Аниме", "Минимал", "Космос",
    "Закат", "Океан", "Лес", "Город неон", "Абстракция", "Пастель",
];
const ANIM_NAMES: &[&str] = &["Вращение", "Пульс", "Волна", "Дрейф", "Статично"];
const W_STYLES: &[&str] = &["Чистый", "Неон", "Печать", "Глитч", "Минимал"];
const W_ANIMS: &[&str] = &["Нет", "Пульс", "Fade", "Bounce", "Slide", "Печать", "Глитч"];

#[derive(Clone, Copy, PartialEq)]
enum Tab { Home, Figure, Style, Shaders, Widgets, Gen, Files }

struct App {
    tab: Tab,
    shape: usize,
    variant: usize,
    color: egui::Color32,
    accent: egui::Color32,
    bg: egui::Color32,
    speed: f32,
    scale: f32,
    glow: bool,
    rainbow: bool,
    anim: usize,
    scene: usize, // shader preset index
    render_style: usize,
    char_size: f32,
    density: f32,
    time: Instant,
    status: String,
    custom: Option<Mask>,
    custom_name: String,
    thr: u8,
    mask_cache: Option<(usize, usize, Mask)>,
    // widgets
    show_clock: bool,
    show_date: bool,
    show_text: bool,
    show_battery: bool,
    show_fps: bool,
    text_val: String,
    w_size: f32,
    w_color: egui::Color32,
    w_style: usize,
    w_x: f32,
    w_y: f32,
    w_anim: usize,
    w_anim_speed: f32,
    w_anim_amt: f32,
    w_3d: bool,
    // gen
    prompt: String,
    gen_style: usize,
    stars: Vec<[f32; 2]>,
    drag_widget: bool,
    /// quality: 0 low 1 med 2 high
    quality: usize,
    last_mesh_frame: u64,
    frame_n: u64,
    export_res: usize, // 0=1080p 1=1440p 2=4k
    api_url: String,
    api_key: String,
    model_url: String,
    model_filename: String,
    vram_mode: comfy::VramMode,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut st = (*cc.egui_ctx.style()).clone();
        st.visuals = egui::Visuals::dark();
        st.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(st);
        let stars = (0..100).map(|i| [((i * 47) % 100) as f32 / 100.0, ((i * 31) % 100) as f32 / 100.0]).collect();
        Self {
            tab: Tab::Home,
            shape: 15, variant: 0,
            color: egui::Color32::from_rgb(80, 255, 160),
            accent: egui::Color32::from_rgb(100, 180, 255),
            bg: egui::Color32::from_rgb(6, 8, 14),
            speed: 1.0, scale: 1.0, glow: true, rainbow: false,
            anim: 0, scene: 1, render_style: 0,
            char_size: 11.0, density: 1.0,
            time: Instant::now(),
            status: "PoiDesktop · стиль / шейдеры / виджеты / .poi / генерация".into(),
            custom: None, custom_name: String::new(), thr: 100, mask_cache: None,
            show_clock: true, show_date: true, show_text: false, show_battery: false, show_fps: false,
            text_val: "PoiDesktop".into(),
            w_size: 40.0, w_color: egui::Color32::from_rgb(230, 255, 240),
            w_style: 1, w_x: 0.5, w_y: 0.1,
            w_anim: 1, w_anim_speed: 1.0, w_anim_amt: 0.5, w_3d: false,
            prompt: String::new(), gen_style: 0, stars, drag_widget: false,
            quality: 1, last_mesh_frame: 0, frame_n: 0,
            export_res: 0,
            api_url: "https://api.stability.ai".into(),
            api_key: String::new(),
            model_url: String::new(),
            model_filename: String::new(),
            vram_mode: comfy::VramMode::Low,
        }
    }

    fn t(&self) -> f32 { self.time.elapsed().as_secs_f32() * self.speed }

    fn mask(&mut self) -> Mask {
        if let Some(ref c) = self.custom { return c.clone(); }
        if let Some((s, v, ref m)) = self.mask_cache {
            if s == self.shape && v == self.variant { return m.clone(); }
        }
        let m = generate_mask(self.shape, self.variant);
        self.mask_cache = Some((self.shape, self.variant, m.clone()));
        m
    }

    fn to_cfg(&self) -> PoiCfg {
        PoiCfg {
            shape: self.shape, variant: self.variant,
            color: [self.color.r(), self.color.g(), self.color.b()],
            accent: [self.accent.r(), self.accent.g(), self.accent.b()],
            bg: [self.bg.r(), self.bg.g(), self.bg.b()],
            speed: self.speed, scale: self.scale, glow: self.glow, rainbow: self.rainbow,
            anim: self.anim, scene: self.scene, render_style: self.render_style,
            shader_id: self.scene, char_size: self.char_size, density: self.density,
            show_clock: self.show_clock, show_date: self.show_date, show_text: self.show_text,
            text_val: self.text_val.clone(),
            w_size: self.w_size, w_color: [self.w_color.r(), self.w_color.g(), self.w_color.b()],
            w_style: self.w_style, w_x: self.w_x, w_y: self.w_y,
            w_anim_mode: self.w_anim, w_anim_speed: self.w_anim_speed, w_anim_amount: self.w_anim_amt,
            prompt: self.prompt.clone(), gen_style: self.gen_style,
        }
    }

    fn from_cfg(&mut self, c: PoiCfg) {
        self.shape = c.shape.min(SHAPE_NAMES.len() - 1);
        self.variant = c.variant;
        self.color = egui::Color32::from_rgb(c.color[0], c.color[1], c.color[2]);
        self.accent = egui::Color32::from_rgb(c.accent[0], c.accent[1], c.accent[2]);
        self.bg = egui::Color32::from_rgb(c.bg[0], c.bg[1], c.bg[2]);
        self.speed = c.speed; self.scale = c.scale; self.glow = c.glow; self.rainbow = c.rainbow;
        self.anim = c.anim.min(4); self.scene = c.scene.min(SHADER_PRESETS.len() - 1);
        self.render_style = c.render_style.min(RENDER_STYLES.len() - 1);
        self.char_size = c.char_size; self.density = c.density;
        self.show_clock = c.show_clock; self.show_date = c.show_date; self.show_text = c.show_text;
        self.text_val = c.text_val;
        self.w_size = c.w_size;
        self.w_color = egui::Color32::from_rgb(c.w_color[0], c.w_color[1], c.w_color[2]);
        self.w_style = c.w_style; self.w_x = c.w_x; self.w_y = c.w_y;
        self.w_anim = c.w_anim_mode; self.w_anim_speed = c.w_anim_speed; self.w_anim_amt = c.w_anim_amount;
        self.prompt = c.prompt; self.gen_style = c.gen_style;
        self.custom = None; self.mask_cache = None;
    }

    fn surprise(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let r = h.finish();
        self.shape = (r as usize) % SHAPE_NAMES.len();
        self.variant = ((r >> 8) as usize) % code_variants(self.shape).len().max(1);
        self.scene = ((r >> 16) as usize) % SHADER_PRESETS.len();
        self.render_style = ((r >> 20) as usize) % RENDER_STYLES.len();
        self.anim = ((r >> 24) as usize) % 5;
        self.color = egui::Color32::from_rgb((r >> 4) as u8, (r >> 12) as u8, (r >> 18) as u8 | 80);
        self.accent = egui::Color32::from_rgb((r >> 6) as u8 | 40, (r >> 14) as u8, (r >> 2) as u8 | 100);
        self.rainbow = r % 5 == 0;
        self.w_anim = ((r >> 10) as usize) % W_ANIMS.len();
        self.custom = None; self.mask_cache = None;
        self.status = format!("🎲 Удиви меня: {} + {}", SHAPE_NAMES[self.shape], SHADER_PRESETS[self.scene]);
    }

    fn apply_prompt_gen(&mut self) {
        let p = self.prompt.to_lowercase();
        // keyword → figure / colors / scene (procedural "neural" styling)
        if p.contains("аниме") || p.contains("тян") || p.contains("девушка") || p.contains("girl") {
            self.shape = 23;
            self.color = egui::Color32::from_rgb(255, 140, 190);
            self.accent = egui::Color32::from_rgb(180, 120, 255);
        } else if p.contains("космос") || p.contains("планет") || p.contains("space") || p.contains("galaxy") {
            self.shape = 15; self.variant = 3; self.scene = 8;
            self.color = egui::Color32::from_rgb(120, 200, 255);
            self.accent = egui::Color32::from_rgb(200, 120, 255);
        } else if p.contains("сердц") || p.contains("love") || p.contains("heart") {
            self.shape = 2; self.anim = 1;
            self.color = egui::Color32::from_rgb(255, 70, 130);
        } else if p.contains("хакер") || p.contains("matrix") || p.contains("код") || p.contains("cyber") {
            self.render_style = 0; self.scene = 2;
            self.color = egui::Color32::from_rgb(40, 255, 90);
        } else if p.contains("ракет") || p.contains("rocket") {
            self.shape = 17; self.scene = 9;
            self.color = egui::Color32::from_rgb(255, 150, 60);
        } else if p.contains("огонь") || p.contains("fire") || p.contains("лава") {
            self.shape = 12; self.scene = 9;
            self.color = egui::Color32::from_rgb(255, 100, 30);
            self.accent = egui::Color32::from_rgb(255, 200, 50);
        } else if p.contains("луна") || p.contains("moon") || p.contains("ночь") {
            self.shape = 7; self.scene = 8;
            self.color = egui::Color32::from_rgb(200, 210, 255);
            self.bg = egui::Color32::from_rgb(4, 6, 16);
        } else if p.contains("океан") || p.contains("море") || p.contains("water") {
            self.shape = 13; self.scene = 3;
            self.color = egui::Color32::from_rgb(40, 160, 255);
            self.accent = egui::Color32::from_rgb(20, 220, 200);
        } else if p.contains("кот") || p.contains("cat") {
            self.shape = 23;
            self.color = egui::Color32::from_rgb(255, 190, 120);
        } else if p.contains("череп") || p.contains("skull") {
            self.shape = 4;
            self.color = egui::Color32::from_rgb(220, 220, 230);
            self.bg = egui::Color32::from_rgb(10, 8, 12);
        } else if p.contains("закат") || p.contains("sunset") {
            self.color = egui::Color32::from_rgb(255, 120, 60);
            self.accent = egui::Color32::from_rgb(255, 60, 120);
            self.bg = egui::Color32::from_rgb(20, 10, 30);
            self.scene = 3;
        } else {
            let h = p.bytes().map(|b| b as usize).sum::<usize>();
            self.shape = h % SHAPE_NAMES.len();
            self.color = egui::Color32::from_rgb(((h * 3) % 200 + 40) as u8, ((h * 7) % 200 + 40) as u8, ((h * 11) % 200 + 55) as u8);
        }
        match self.gen_style {
            0 => { self.render_style = 2; self.scene = 0; self.glow = true; } // photo smooth
            1 => { self.scene = 5; self.glow = true; self.render_style = 2; }
            2 => { self.scene = 1; self.render_style = 2; }
            3 => { self.render_style = 1; }
            4 => { self.render_style = 5; self.glow = true; }
            5 => { self.color = egui::Color32::from_rgb(255, 150, 200); self.scene = 11; self.render_style = 2; }
            6 => { self.render_style = 2; self.scene = 0; self.glow = false; }
            7 => { self.scene = 8; self.shape = 15; self.render_style = 2; }
            8 => { // sunset
                self.color = egui::Color32::from_rgb(255, 110, 50);
                self.accent = egui::Color32::from_rgb(255, 50, 100);
                self.bg = egui::Color32::from_rgb(25, 8, 30);
                self.render_style = 2; self.scene = 3;
            }
            9 => {
                self.color = egui::Color32::from_rgb(30, 140, 255);
                self.accent = egui::Color32::from_rgb(20, 230, 200);
                self.bg = egui::Color32::from_rgb(4, 20, 40);
                self.render_style = 2;
            }
            10 => {
                self.color = egui::Color32::from_rgb(80, 200, 100);
                self.bg = egui::Color32::from_rgb(8, 20, 10);
                self.render_style = 2;
            }
            11 => {
                self.color = egui::Color32::from_rgb(255, 50, 180);
                self.accent = egui::Color32::from_rgb(50, 200, 255);
                self.bg = egui::Color32::from_rgb(8, 4, 20);
                self.scene = 2; self.render_style = 5;
            }
            12 => { self.render_style = 2; self.scene = 3; self.rainbow = true; }
            13 => {
                self.color = egui::Color32::from_rgb(255, 180, 200);
                self.accent = egui::Color32::from_rgb(180, 200, 255);
                self.bg = egui::Color32::from_rgb(30, 25, 35);
                self.render_style = 2; self.scene = 0;
            }
            _ => { self.render_style = 2; }
        }
        self.custom = None;
        self.mask_cache = None;
        self.status = format!("✨ «{}» → {} · {}", self.prompt, SHAPE_NAMES[self.shape], GEN_STYLES.get(self.gen_style).unwrap_or(&"?"));
    }

    fn apply_wallpaper(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        let (w, h) = match self.export_res {
            1 => (2560, 1440),
            2 => (3840, 2160),
            _ => (1920, 1080),
        };
        match self.render_png(&path, w, h) {
            Ok(()) => match wallpaper::set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Ошибка: {e}"),
            },
            Err(e) => self.status = format!("Рендер: {e}"),
        }
    }

    fn render_png(&mut self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba, RgbaImage};
        let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([self.bg.r(), self.bg.g(), self.bg.b(), 255]));
        let mask = self.mask();
        let t = if self.anim == 4 { 0.0 } else { self.t() };
        let smooth = self.render_style == 2 || self.gen_style == 0;

        // gradient background for photo-like wallpapers
        if smooth {
            for y in 0..h {
                let v = y as f32 / h as f32;
                let r = ((1.0 - v) * self.bg.r() as f32 + v * self.accent.r() as f32 * 0.25) as u8;
                let g = ((1.0 - v) * self.bg.g() as f32 + v * self.accent.g() as f32 * 0.25) as u8;
                let b = ((1.0 - v) * self.bg.b() as f32 + v * self.accent.b() as f32 * 0.25) as u8;
                for x in 0..w {
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }

        // stars
        for s in &self.stars {
            let x = (s[0] * w as f32) as u32 % w;
            let y = (s[1] * h as f32) as u32 % h;
            let p = img.get_pixel_mut(x, y);
            p.0[0] = p.0[0].saturating_add(40);
            p.0[1] = p.0[1].saturating_add(40);
            p.0[2] = p.0[2].saturating_add(50);
        }

        let cell = (h as f32 * 0.011 * self.scale).max(5.0);
        let ox = w as f32 * 0.5 - COLS as f32 * cell * 0.5;
        let oy = h as f32 * 0.48 - mask.rows as f32 * cell * 0.5;

        if smooth {
            // soft volumetric blobs from mask — photo wallpaper look
            for y in (0..mask.rows).step_by(1) {
                for x in (0..COLS).step_by(1) {
                    if !mask.on[y * COLS + x] { continue; }
                    let depth = mask.depth[y * COLS + x];
                    let (px, py) = self.cell_pos(x, y, ox, oy, cell, t, mask.rows);
                    let (r, g, b) = self.color_at(depth, x, y, t);
                    let radius = cell * (1.1 + depth * 0.8);
                    soft_stamp(&mut img, w, h, px, py, radius, r, g, b, 0.55 + depth * 0.4);
                }
            }
            // vignette
            for y in 0..h {
                for x in 0..w {
                    let nx = x as f32 / w as f32 * 2.0 - 1.0;
                    let ny = y as f32 / h as f32 * 2.0 - 1.0;
                    let d = (nx * nx + ny * ny).sqrt();
                    let v = (1.0 - (d - 0.65).max(0.0) * 1.1).clamp(0.35, 1.0);
                    let p = img.get_pixel_mut(x, y);
                    p.0[0] = (p.0[0] as f32 * v) as u8;
                    p.0[1] = (p.0[1] as f32 * v) as u8;
                    p.0[2] = (p.0[2] as f32 * v) as u8;
                }
            }
        } else {
            for y in 0..mask.rows {
                for x in 0..COLS {
                    if !mask.on[y * COLS + x] { continue; }
                    let depth = mask.depth[y * COLS + x];
                    let (px, py) = self.cell_pos(x, y, ox, oy, cell, t, mask.rows);
                    let (r, g, b) = self.color_at(depth, x, y, t);
                    let size = (cell * 0.8) as i32;
                    for dy in -size / 2..=size / 2 {
                        for dx in -size / 2..=size / 2 {
                            if self.render_style == 1 {
                                if dx.abs() > size / 3 || dy.abs() > size / 3 { continue; }
                            } else if dx * dx + dy * dy > (size / 2) * (size / 2) {
                                continue;
                            }
                            let xx = px as i32 + dx;
                            let yy = py as i32 + dy;
                            if xx >= 0 && yy >= 0 && (xx as u32) < w && (yy as u32) < h {
                                let p = img.get_pixel_mut(xx as u32, yy as u32);
                                let a = 0.7 + 0.3 * depth;
                                p.0[0] = ((1.0 - a) * p.0[0] as f32 + a * r as f32) as u8;
                                p.0[1] = ((1.0 - a) * p.0[1] as f32 + a * g as f32) as u8;
                                p.0[2] = ((1.0 - a) * p.0[2] as f32 + a * b as f32) as u8;
                            }
                        }
                    }
                }
            }
        }

        img.save(path).map_err(|e| e.to_string())
    }

    fn cell_pos(&self, x: usize, y: usize, ox: f32, oy: f32, cell: f32, t: f32, rows: usize) -> (f32, f32) {
        let mut px = ox + x as f32 * cell + cell * 0.5;
        let mut py = oy + y as f32 * cell + cell * 0.5;
        let nx = x as f32 / COLS as f32 - 0.5;
        let ny = y as f32 / rows as f32 - 0.5;
        let cx = ox + COLS as f32 * cell * 0.5;
        let cy = oy + rows as f32 * cell * 0.5;
        match self.anim {
            0 => {
                let a = t * 0.4;
                px = cx + (nx * a.cos() - ny * a.sin()) * COLS as f32 * cell;
                py = cy + (nx * a.sin() + ny * a.cos()) * rows as f32 * cell;
            }
            1 => {
                let s = 1.0 + 0.06 * (t * 2.0).sin();
                px = cx + (px - cx) * s;
                py = cy + (py - cy) * s;
            }
            2 => py += (t * 2.0 + x as f32 * 0.3).sin() * cell * 0.8,
            3 => {
                px += (t * 0.7 + y as f32 * 0.2).sin() * cell * 0.5;
                py += (t * 0.5).cos() * cell * 0.4;
            }
            _ => {}
        }
        (px, py)
    }

    fn color_at(&self, depth: f32, x: usize, y: usize, t: f32) -> (u8, u8, u8) {
        let light = 0.45 + 0.55 * depth;
        if self.rainbow || self.render_style == 3 {
            let h = ((x + y) as f32 * 0.03 + t * 0.15 + if self.render_style == 3 { depth } else { 0.0 }) % 1.0;
            (
                ((h * 255.0) as u8 as f32 * light) as u8,
                ((((h + 0.33) % 1.0) * 220.0) as u8 as f32 * light) as u8,
                ((((h + 0.66) % 1.0) * 255.0) as u8 as f32 * light) as u8,
            )
        } else {
            let r = (self.color.r() as f32 * (1.0 - depth * 0.3) + self.accent.r() as f32 * depth * 0.3) * light;
            let g = (self.color.g() as f32 * (1.0 - depth * 0.3) + self.accent.g() as f32 * depth * 0.3) * light;
            let b = (self.color.b() as f32 * (1.0 - depth * 0.3) + self.accent.b() as f32 * depth * 0.3) * light;
            (r as u8, g as u8, b as u8)
        }
    }
}


fn soft_stamp(img: &mut image::RgbaImage, w: u32, h: u32, cx: f32, cy: f32, radius: f32, r: u8, g: u8, b: u8, strength: f32) {
    let rad = radius.max(1.0);
    let min_x = ((cx - rad * 1.6).floor() as i32).max(0) as u32;
    let max_x = ((cx + rad * 1.6).ceil() as i32).min(w as i32 - 1) as u32;
    let min_y = ((cy - rad * 1.6).floor() as i32).max(0) as u32;
    let max_y = ((cy + rad * 1.6).ceil() as i32).min(h as i32 - 1) as u32;
    let inv = 1.0 / rad;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let d = ((x as f32 - cx).hypot(y as f32 - cy)) * inv;
            if d > 1.6 { continue; }
            let a = ((1.0 - d / 1.6).max(0.0).powf(1.8) * strength).clamp(0.0, 1.0);
            if a < 0.01 { continue; }
            let p = img.get_pixel_mut(x, y);
            p.0[0] = ((1.0 - a) * p.0[0] as f32 + a * r as f32) as u8;
            p.0[1] = ((1.0 - a) * p.0[1] as f32 + a * g as f32) as u8;
            p.0[2] = ((1.0 - a) * p.0[2] as f32 + a * b as f32) as u8;
        }
    }
}

fn chip_row(ui: &mut egui::Ui, items: &[&str], selected: usize, on_pick: &mut dyn FnMut(usize)) {
    ui.horizontal_wrapped(|ui| {
        for (i, n) in items.iter().enumerate() {
            if ui.selectable_label(selected == i, *n).clicked() {
                on_pick(i);
            }
        }
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.anim != 4 { ctx.request_repaint(); }

        // top nav like phone tabs
        egui::TopBottomPanel::top("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Poi");
                ui.separator();
                for (t, n) in [
                    (Tab::Home, "⌂ Главная"),
                    (Tab::Figure, "Фигуры"),
                    (Tab::Style, "Стиль"),
                    (Tab::Shaders, "Шейдеры"),
                    (Tab::Widgets, "Виджеты"),
                    (Tab::Gen, "Генерация"),
                    (Tab::Files, "Файлы"),
                ] {
                    if ui.selectable_label(self.tab == t, n).clicked() { self.tab = t; }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🎲 Удиви меня").clicked() { self.surprise(); }
                    if ui.button("✨ Обои").clicked() { self.apply_wallpaper(); }
                });
            });
        });
        egui::TopBottomPanel::bottom("st").show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.status).color(egui::Color32::from_rgb(140, 255, 180)));
        });

        egui::SidePanel::left("side").default_width(360.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Home => {
                    ui.label(egui::RichText::new("Быстрый старт").strong().size(16.0));
                    ui.label("1. Фигура → 2. Стиль → 3. Шейдер → 4. Обои");
                    ui.separator();
                    if ui.button("🎲 Удиви меня").clicked() { self.surprise(); }
                    if ui.button("✨ Установить обои").clicked() { self.apply_wallpaper(); }
                    if ui.button("↩ Вернуть старые").clicked() {
                        match wallpaper::restore_backup() {
                            Ok(()) => self.status = "↩ Ок".into(),
                            Err(e) => self.status = e,
                        }
                    }
                    ui.separator();
                    ui.label(format!("Сейчас: {} · {} · {}", SHAPE_NAMES[self.shape], RENDER_STYLES[self.render_style], SHADER_PRESETS[self.scene]));
                }
                Tab::Figure => {
                    ui.label(egui::RichText::new("Модели / фигуры").strong());
                    if self.custom.is_some() {
                        ui.horizontal(|ui| {
                            ui.label(format!("📷 {}", self.custom_name));
                            if ui.button("Сброс").clicked() { self.custom = None; self.mask_cache = None; }
                        });
                    }
                    chip_row(ui, SHAPE_NAMES, self.shape, &mut |i| {
                        self.shape = i; self.variant = 0; self.custom = None; self.mask_cache = None;
                    });
                    ui.separator();
                    ui.label("Код-варианты (зависят от фигуры)");
                    let vars: Vec<&str> = code_variants(self.shape).to_vec();
                    chip_row(ui, &vars, self.variant, &mut |i| { self.variant = i; self.mask_cache = None; });
                    ui.separator();
                    ui.add(egui::Slider::new(&mut self.thr, 40..=180).text("Порог фото"));
                    if ui.button("📂 Фото → фигура").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("Img", &["png","jpg","jpeg","webp"]).pick_file() {
                            match mask_from_image(&p, self.thr) {
                                Ok(m) => {
                                    self.custom_name = p.file_stem().map(|s| s.to_string_lossy().into()).unwrap_or_else(|| "Фото".into());
                                    self.custom = Some(m); self.mask_cache = None;
                                    self.status = format!("📷 {}", self.custom_name);
                                }
                                Err(e) => self.status = e,
                            }
                        }
                    }
                }
                Tab::Style => {
                    ui.label(egui::RichText::new("Стиль фигуры").strong());
                    chip_row(ui, RENDER_STYLES, self.render_style, &mut |i| self.render_style = i);
                    ui.separator();
                    ui.label("Анимация фигуры");
                    chip_row(ui, ANIM_NAMES, self.anim, &mut |i| self.anim = i);
                    ui.add(egui::Slider::new(&mut self.speed, 0.2..=2.5).text("Скорость"));
                    ui.add(egui::Slider::new(&mut self.scale, 0.6..=1.6).text("Масштаб"));
                    ui.add(egui::Slider::new(&mut self.char_size, 8.0..=18.0).text("Размер элементов"));
                    ui.add(egui::Slider::new(&mut self.density, 0.3..=1.0).text("Плотность"));
                    ui.horizontal(|ui| {
                        ui.label("Цвет"); ui.color_edit_button_srgba(&mut self.color);
                        ui.label("Акцент"); ui.color_edit_button_srgba(&mut self.accent);
                        ui.label("Фон"); ui.color_edit_button_srgba(&mut self.bg);
                    });
                    ui.checkbox(&mut self.rainbow, "Радуга");
                    ui.checkbox(&mut self.glow, "Свечение");
                    ui.separator();
                    ui.label(egui::RichText::new("Оптимизация рендера").strong());
                    ui.label("Качество (ниже = быстрее)");
                    for (q, n) in [(0usize, "Низкое (быстро)"), (1, "Среднее"), (2, "Высокое")] {
                        if ui.selectable_label(self.quality == q, n).clicked() {
                            self.quality = q;
                        }
                    }
                    ui.label(egui::RichText::new("WGPU/GLSL: полный GPU-пайплайн — следующий этап; сейчас CPU + лимиты draw.").weak().small());
                    ui.separator();
                    ui.label(egui::RichText::new("Качество экспорта обоев").strong());
                    for (i, n) in [(0usize, "Full HD 1080p"), (1, "QHD 1440p"), (2, "4K UHD")].iter() {
                        if ui.selectable_label(self.export_res == *i, *n).clicked() {
                            self.export_res = *i;
                        }
                    }
                    ui.label("Выше разрешение = дольше сохранение PNG.");
                }
                Tab::Shaders => {
                    ui.label(egui::RichText::new("Шейдеры / эффекты").strong().size(15.0));
                    ui.label("Пресеты в духе GLSL (plasma, FBM, domain warp, matrix…)");
                    ui.label(egui::RichText::new("Настоящие GPU GLSL в egui — через wgpu callback; здесь CPU-аналоги высокого качества.").weak().small());
                    ui.label(egui::RichText::new("3D-модели: готовь в Blender → экспорт PNG/силуэт → «Фото → фигура».").weak().small());
                    ui.separator();
                    chip_row(ui, SHADER_PRESETS, self.scene.min(SHADER_PRESETS.len()-1), &mut |i| {
                        self.scene = i;
                        self.status = format!("Шейдер: {}", SHADER_PRESETS[i]);
                    });
                }
                Tab::Widgets => {
                    ui.label(egui::RichText::new("Виджеты").strong());
                    ui.checkbox(&mut self.show_clock, "Часы");
                    ui.checkbox(&mut self.show_date, "Дата");
                    ui.checkbox(&mut self.show_text, "Текст");
                    ui.checkbox(&mut self.show_battery, "Батарея (инфо)");
                    ui.checkbox(&mut self.show_fps, "FPS");
                    if self.show_text { ui.text_edit_singleline(&mut self.text_val); }
                    ui.separator();
                    ui.label("Стиль виджета");
                    chip_row(ui, W_STYLES, self.w_style, &mut |i| self.w_style = i);
                    ui.label("Анимация виджета");
                    chip_row(ui, W_ANIMS, self.w_anim, &mut |i| self.w_anim = i);
                    ui.add(egui::Slider::new(&mut self.w_anim_speed, 0.2..=3.0).text("Скорость аним."));
                    ui.add(egui::Slider::new(&mut self.w_anim_amt, 0.1..=1.0).text("Сила аним."));
                    ui.checkbox(&mut self.w_3d, "3D-эффект (параллакс масштаба)");
                    ui.separator();
                    ui.label("Позиция (можно также тащить на превью)");
                    ui.add(egui::Slider::new(&mut self.w_x, 0.05..=0.95).text("X"));
                    ui.add(egui::Slider::new(&mut self.w_y, 0.02..=0.9).text("Y"));
                    ui.add(egui::Slider::new(&mut self.w_size, 16.0..=72.0).text("Размер"));
                    ui.horizontal(|ui| { ui.label("Цвет"); ui.color_edit_button_srgba(&mut self.w_color); });
                }
                Tab::Gen => {
                    ui.label(egui::RichText::new("Генерация обоев").strong());
                    ui.label("Опиши сцену — подберу фигуру, цвета, шейдер. Стиль «4K фото-обои» = гладкий PNG.");
                    ui.label(egui::RichText::new("Локальная SD/Flux не встроена (нужны ГБ модели или API). Здесь сильный процедурный движок.").weak().small());
                    ui.text_edit_multiline(&mut self.prompt);
                    ui.label("Стиль генерации");
                    chip_row(ui, GEN_STYLES, self.gen_style, &mut |i| self.gen_style = i);
                    if ui.button("✨ Сгенерировать превью").clicked() {
                        if self.prompt.trim().is_empty() {
                            self.status = "Введи описание".into();
                        } else {
                            self.apply_prompt_gen();
                        }
                    }
                    if ui.button("✨ Сгенерировать и поставить обои").clicked() {
                        if self.prompt.trim().is_empty() {
                            self.status = "Введи описание".into();
                        } else {
                            self.apply_prompt_gen();
                            self.apply_wallpaper();
                        }
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("AI / ComfyUI (авто)").strong());
                    ui.label("Нужен только Python (PATH). Git не обязателен — ZIP встроен в setup/.");
                    let installed = comfy::is_comfy_installed();
                    ui.label(if installed {
                        format!("✅ {}", comfy::comfy_root().display())
                    } else {
                        "❌ ComfyUI ещё не установлен".into()
                    });
                    if ui.button("🚀 АВТОУСТАНОВКА: ComfyUI + pip").clicked() {
                        self.status = "Установка ComfyUI… подожди (минуты).".into();
                        match comfy::auto_install_all() {
                            Ok(m) => self.status = m,
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button("⬇ Скачать стартовую модель SD1.5 (~4GB)").clicked() {
                        self.status = "Качаю SD1.5… UI зависнет до конца (долго).".into();
                        match comfy::download_model(comfy::DEFAULT_MODEL_URL, comfy::DEFAULT_MODEL_NAME) {
                            Ok(m) => self.status = m,
                            Err(e) => self.status = e,
                        }
                    }
                    ui.label(egui::RichText::new("Flux 10–20GB — вручную или свой URL ниже (не качаем молча при старте).").weak().small());
                    for m in [comfy::VramMode::Normal, comfy::VramMode::Low, comfy::VramMode::NoVram] {
                        if ui.selectable_label(self.vram_mode == m, m.label()).clicked() {
                            self.vram_mode = m;
                        }
                    }
                    if ui.button("▶ Запустить ComfyUI").clicked() {
                        match comfy::launch_comfy(self.vram_mode) {
                            Ok(m) => {
                                self.status = m;
                                #[cfg(windows)]
                                {
                                    let _ = std::process::Command::new("cmd")
                                        .args(["/C", "start", "", "http://127.0.0.1:8188"])
                                        .spawn();
                                }
                            }
                            Err(e) => self.status = e,
                        }
                    }
                    ui.separator();
                    ui.label("Свой URL модели (Flux GGUF / SDXL / …)");
                    ui.text_edit_singleline(&mut self.model_url);
                    ui.text_edit_singleline(&mut self.model_filename);
                    if ui.button("⬇ Скачать по URL").clicked() {
                        self.status = "Загрузка…".into();
                        match comfy::download_model(&self.model_url, &self.model_filename) {
                            Ok(m) => self.status = m,
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button("🌐 Flux / GGUF на HF").clicked() {
                        comfy::open_flux_models_page();
                    }
                    let models = comfy::list_models();
                    ui.label(format!("Модели: {}", models.len()));
                    for m in models.iter().take(6) {
                        ui.monospace(m);
                    }
                    ui.separator();
                    ui.label(egui::RichText::new("Облачный API (SDXL / Flux)").strong());
                    ui.label("URL API");
                    ui.text_edit_singleline(&mut self.api_url);
                    ui.label("API Key");
                    ui.text_edit_singleline(&mut self.api_key);
                    if ui.button("🌐 Проверить ключ (заготовка)").clicked() {
                        if self.api_key.trim().is_empty() {
                            self.status = "Нужен API key (Stability / Together / Fireworks)".into();
                        } else {
                            self.status = format!("Ключ задан · URL {}", self.api_url);
                        }
                    }
                }
                Tab::Files => {
                    ui.label(egui::RichText::new("Конфиги .poi").strong());
                    if ui.button("💾 Сохранить .poi").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("Poi", &["poi"]).set_file_name("setup.poi").save_file() {
                            match config::save_poi(&p, &self.to_cfg()) {
                                Ok(()) => self.status = format!("Сохранено {}", p.display()),
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    if ui.button("📂 Загрузить .poi").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("Poi", &["poi","json"]).pick_file() {
                            match config::load_poi(&p) {
                                Ok(c) => { self.from_cfg(c); self.status = format!("Загружено {}", p.display()); }
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    ui.separator();
                    if ui.button("💾 PNG 1080p").clicked() {
                        let p = wallpaper::output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%H%M%S")));
                        match self.render_png(&p, 1920, 1080) {
                            Ok(()) => self.status = format!("{}", p.display()),
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button("✨ Установить обои").clicked() { self.apply_wallpaper(); }
                    if ui.button("↩ Вернуть обои").clicked() {
                        let _ = wallpaper::restore_backup().map(|_| self.status = "↩ Ок".into());
                    }
                }
            });
        });

        // preview
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let (rect, resp) = ui.allocate_exact_size(avail, egui::Sense::click_and_drag());
            // drag widgets
            if resp.dragged() {
                let p = resp.interact_pointer_pos().unwrap_or(rect.center());
                self.w_x = ((p.x - rect.left()) / rect.width()).clamp(0.05, 0.95);
                self.w_y = ((p.y - rect.top()) / rect.height()).clamp(0.02, 0.9);
            }
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, self.bg);
            let t = if self.anim == 4 { 0.0 } else { self.t() };

            // shader backgrounds
            self.draw_shader(&painter, rect, t);

            let mask = self.mask();
            let cell = (rect.height() * 0.011 * self.scale).max(7.0) * self.char_size / 11.0;
            let ox = rect.center().x - COLS as f32 * cell * 0.5;
            let oy = rect.center().y - mask.rows as f32 * cell * 0.5 - 8.0;

            // --- optimized figure render ---
            self.frame_n = self.frame_n.wrapping_add(1);
            let step = match self.quality {
                0 => 2usize, // low: every 2nd cell
                2 => 1usize,
                _ => 1usize,
            };
            let max_glyphs = match self.quality {
                0 => 900usize,
                1 => 1800,
                _ => 4000,
            };
            let mut drawn = 0usize;
            let use_text = matches!(self.render_style, 0 | 3 | 7);
            let font = egui::FontId::monospace((cell * 0.95).clamp(8.0, 20.0));

            'rows: for y in (0..mask.rows).step_by(step) {
                for x in (0..COLS).step_by(step) {
                    if drawn >= max_glyphs {
                        break 'rows;
                    }
                    let i = y * COLS + x;
                    if !mask.on[i] {
                        continue;
                    }
                    if self.density < 0.99 {
                        let dens_step = (1.0 / self.density.max(0.25)) as usize;
                        if (x + y * 3) % dens_step.max(1) != 0 {
                            continue;
                        }
                    }
                    let depth = mask.depth[i];
                    let (px, py) = self.cell_pos(x, y, ox, oy, cell, t, mask.rows);
                    // frustum cull
                    if px < rect.left() - 20.0
                        || px > rect.right() + 20.0
                        || py < rect.top() - 20.0
                        || py > rect.bottom() + 20.0
                    {
                        continue;
                    }
                    let (r, g, b) = self.color_at(depth, x, y, t);
                    let col = egui::Color32::from_rgb(r, g, b);
                    let pos = egui::pos2(px, py);
                    match self.render_style {
                        1 => {
                            painter.rect_filled(
                                egui::Rect::from_center_size(pos, egui::vec2(cell * 0.85, cell * 0.85)),
                                0.0,
                                col,
                            );
                        }
                        2 => {
                            painter.circle_filled(pos, cell * 0.45, col);
                        }
                        4 => {
                            painter.circle_stroke(pos, cell * 0.4, egui::Stroke::new(1.0_f32, col));
                        }
                        5 => {
                            painter.line_segment(
                                [pos - egui::vec2(cell * 0.3, 0.0), pos + egui::vec2(cell * 0.3, 0.0)],
                                egui::Stroke::new(2.0_f32, col),
                            );
                        }
                        6 => {
                            painter.circle_filled(pos, cell * 0.25, col);
                        }
                        _ => {
                            let ch = CODE_CHARS[(x * 7 + y * 13 + (t as usize)) % CODE_CHARS.len()];
                            if self.glow && self.quality >= 1 {
                                painter.text(
                                    pos,
                                    egui::Align2::CENTER_CENTER,
                                    ch,
                                    font.clone(),
                                    egui::Color32::from_rgba_unmultiplied(r, g, b, 40),
                                );
                            }
                            painter.text(pos, egui::Align2::CENTER_CENTER, ch, font.clone(), col);
                        }
                    }
                    drawn += 1;
                }
            }
            let _ = use_text;

            self.draw_widgets(&painter, rect, t);
        });
    }
}

impl App {
    fn draw_shader(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        let name = SHADER_PRESETS.get(self.scene).copied().unwrap_or("Нет");
        if name == "Нет" {
            return;
        }
        let n = name.to_lowercase();
        let c = self.accent;
        let c2 = self.color;

        // --- star layers ---
        if n.contains("star") || n.contains("space") || n.contains("galaxy") || n.contains("nebula")
            || n.contains("void") || n.contains("hyperspace") || n.contains("warp")
        {
            for (i, s) in self.stars.iter().enumerate() {
                let spd = 0.01 + (i % 5) as f32 * 0.004;
                let y = (s[1] + t * spd) % 1.0;
                let bright = 120 + (i % 100) as u8;
                painter.circle_filled(
                    egui::pos2(rect.left() + s[0] * rect.width(), rect.top() + y * rect.height()),
                    if i % 7 == 0 { 1.8 } else { 1.0 },
                    egui::Color32::from_gray(bright),
                );
            }
        }

        // --- plasma / fbm / noise field / domain warp (GLSL-like) ---
        if n.contains("plasma")
            || n.contains("fbm")
            || n.contains("noise")
            || n.contains("domain")
            || n.contains("flow")
            || n.contains("fog")
            || n.contains("mist")
            || n.contains("smoke")
            || n.contains("cloud")
            || n.contains("ink")
            || n.contains("lava")
            || n.contains("caustic")
            || n.contains("vapor")
            || n.contains("nebula")
            || n.contains("hue")
            || n.contains("gradient")
            || n.contains("color band")
        {
            let step = if n.contains("fbm") || n.contains("domain") { 8.0 } else { 11.0 };
            let mut y = rect.top();
            while y < rect.bottom() {
                let mut x = rect.left();
                while x < rect.right() {
                    let mut nx = (x - rect.left()) / rect.width();
                    let mut ny = (y - rect.top()) / rect.height();
                    // domain warp
                    if n.contains("domain") || n.contains("flow") || n.contains("warp") {
                        let wx = (ny * 4.0 + t * 0.4).sin() * 0.15;
                        let wy = (nx * 4.0 - t * 0.3).cos() * 0.15;
                        nx += wx;
                        ny += wy;
                    }
                    let v = if n.contains("fbm") {
                        let mut a = 0.0;
                        let mut amp = 0.5;
                        let mut fre = 1.0;
                        for _ in 0..4 {
                            a += amp * ((nx * fre * 6.0 + t).sin() * (ny * fre * 5.0 - t * 0.7).cos());
                            amp *= 0.5;
                            fre *= 2.0;
                        }
                        a * 0.5 + 0.5
                    } else {
                        ((nx * 5.0 + t).sin() + (ny * 6.0 - t * 0.6).cos() + ((nx + ny) * 3.0 + t * 0.5).sin()) * 0.2 + 0.5
                    };
                    let mixc = if n.contains("vapor") || n.contains("hue") {
                        egui::Color32::from_rgb(
                            ((v * 255.0) as u8).wrapping_add(c.r() / 3),
                            c2.g(),
                            ((1.0 - v) * 200.0) as u8,
                        )
                    } else {
                        c
                    };
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(step + 0.5, step + 0.5)),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(mixc.r(), mixc.g(), mixc.b(), (v * 48.0) as u8),
                    );
                    x += step;
                }
                y += step;
            }
        }

        // --- matrix / rain / codefall ---
        if n.contains("matrix") || n.contains("rain") || n.contains("codefall") || n.contains("digital") {
            let cols = 26;
            let cw = rect.width() / cols as f32;
            let blue = n.contains("blue");
            for i in 0..cols {
                let head = ((t * (0.35 + (i as f32) * 0.025) + i as f32 * 0.17) % 1.35) - 0.2;
                for k in 0..14 {
                    let yy = head - k as f32 * 0.04;
                    if !(0.0..=1.0).contains(&yy) {
                        continue;
                    }
                    let ch = CODE_CHARS[(i * 3 + k + (t as usize)) % CODE_CHARS.len()];
                    let (rr, gg, bb) = if blue {
                        (80u8, 160, 255)
                    } else {
                        (c2.r(), c2.g(), c2.b())
                    };
                    painter.text(
                        egui::pos2(rect.left() + i as f32 * cw, rect.top() + yy * rect.height()),
                        egui::Align2::LEFT_TOP,
                        ch,
                        egui::FontId::monospace(11.0),
                        egui::Color32::from_rgba_unmultiplied(rr, gg, bb, (170 - k * 9) as u8),
                    );
                }
            }
        }

        // --- rings / ripple / aura / shockwave / force ---
        if n.contains("ripple")
            || n.contains("pulse")
            || n.contains("aura")
            || n.contains("shock")
            || n.contains("force")
            || n.contains("orbital")
            || n.contains("glow bloom")
        {
            for i in 0..14 {
                let phase = t * 0.45 + i as f32 * 0.35;
                let rad = (phase % 1.0) * rect.height() * 0.5;
                let a = ((1.0 - phase % 1.0) * 70.0) as u8;
                painter.circle_stroke(
                    rect.center(),
                    rad,
                    egui::Stroke::new(2.0_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)),
                );
            }
        }

        // --- grids ---
        if n.contains("grid") || n.contains("hex") || n.contains("retro") || n.contains("perspective") {
            let step = if n.contains("hex") { 42.0 } else { 34.0 };
            let mut x = rect.left();
            while x < rect.right() {
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 28)),
                );
                x += step;
            }
            let mut y = rect.top();
            while y < rect.bottom() {
                let skew = if n.contains("perspective") || n.contains("retro") {
                    (y - rect.top()) / rect.height() * 20.0
                } else {
                    0.0
                };
                painter.line_segment(
                    [
                        egui::pos2(rect.left() - skew, y),
                        egui::pos2(rect.right() + skew, y),
                    ],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 28)),
                );
                y += step * (if n.contains("perspective") { 0.7 + (y - rect.top()) / rect.height() } else { 1.0 });
            }
        }

        // --- scanlines / CRT / glitch / rgb split ---
        if n.contains("scan") || n.contains("crt") {
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(1.0_f32, egui::Color32::from_black_alpha(40)),
                );
                y += 3.0;
            }
        }
        if n.contains("glitch") || n.contains("rgb") {
            if (t * 7.0).sin() > 0.7 {
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(rect.left() + rect.width() * 0.1, rect.top() + rect.height() * ((t * 3.0).sin().abs())),
                        egui::vec2(rect.width() * 0.8, 8.0),
                    ),
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(255, 50, 100, 40),
                );
            }
        }

        // --- snow / ash / sparks / fireflies ---
        if n.contains("snow") || n.contains("blizzard") || n.contains("ash") || n.contains("spark") || n.contains("firefly") || n.contains("ember") {
            for (i, s) in self.stars.iter().enumerate() {
                let y = (s[1] + t * (0.05 + (i % 4) as f32 * 0.02)) % 1.0;
                let x = (s[0] + (if n.contains("blizzard") { t * 0.03 } else { 0.0 })) % 1.0;
                painter.circle_filled(
                    egui::pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height()),
                    if n.contains("firefly") { 2.0 } else { 1.3 },
                    if n.contains("ember") || n.contains("spark") {
                        egui::Color32::from_rgb(255, 160, 60)
                    } else {
                        egui::Color32::from_gray(220)
                    },
                );
            }
        }

        // --- tunnel / hyperspace lines ---
        if n.contains("tunnel") || n.contains("hyperspace") || n.contains("warp speed") {
            let cx = rect.center().x;
            let cy = rect.center().y;
            for i in 0..24 {
                let a = i as f32 / 24.0 * std::f32::consts::TAU + t * 0.2;
                let len = rect.height() * 0.45 * (0.3 + 0.7 * ((t * 2.0 + i as f32).sin().abs()));
                painter.line_segment(
                    [
                        egui::pos2(cx, cy),
                        egui::pos2(cx + a.cos() * len, cy + a.sin() * len),
                    ],
                    egui::Stroke::new(1.5_f32, egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 50)),
                );
            }
        }

        // --- kaleidoscope / mandala ---
        if n.contains("kaleido") || n.contains("mandala") || n.contains("crystal") {
            let cx = rect.center().x;
            let cy = rect.center().y;
            for ring in 1..8 {
                let rad = ring as f32 * rect.height() * 0.05;
                let segs = ring * 6;
                for i in 0..segs {
                    let a = i as f32 / segs as f32 * std::f32::consts::TAU + t * 0.3 * (ring as f32).sqrt();
                    painter.circle_filled(
                        egui::pos2(cx + a.cos() * rad * 3.0, cy + a.sin() * rad * 3.0),
                        2.0,
                        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 70),
                    );
                }
            }
        }

        // --- black hole / void swirl ---
        if n.contains("black hole") || n.contains("void") {
            for i in 0..40 {
                let u = i as f32 / 39.0;
                let a = u * std::f32::consts::TAU * 3.0 + t;
                let r = 20.0 + u * rect.height() * 0.35;
                painter.circle_filled(
                    egui::pos2(rect.center().x + a.cos() * r, rect.center().y + a.sin() * r * 0.7),
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 60),
                );
            }
        }
    }

    fn draw_widgets(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        let ta = t * self.w_anim_speed;
        let mut scale = 1.0;
        let mut ox = 0.0;
        let mut oy = 0.0;
        let mut alpha = 1.0;
        match self.w_anim {
            1 => scale = 1.0 + 0.05 * self.w_anim_amt * (ta * 2.0).sin(),
            2 => alpha = 0.5 + 0.5 * (0.5 + 0.5 * (ta * 1.5).sin()),
            3 => oy = (ta * 2.5).sin().abs() * 8.0 * self.w_anim_amt,
            4 => ox = (ta * 1.2).sin() * 20.0 * self.w_anim_amt,
            6 => { if (ta * 9.0).sin() > 0.9 { ox = 5.0 * self.w_anim_amt; } }
            _ => {}
        }
        if self.w_3d { scale *= 1.0 + 0.04 * (ta).sin(); }
        let size = self.w_size * scale;
        let col = egui::Color32::from_rgba_unmultiplied(self.w_color.r(), self.w_color.g(), self.w_color.b(), (alpha * 255.0) as u8);
        let pos = egui::pos2(rect.left() + self.w_x * rect.width() + ox, rect.top() + self.w_y * rect.height() + oy);
        let font = egui::FontId::monospace(size);
        if self.show_clock {
            let now = chrono::Local::now().format("%H:%M:%S").to_string();
            let text = if self.w_anim == 5 {
                let n = ((ta * 5.0) as usize) % (now.len() + 1);
                now.chars().take(n).collect()
            } else { now };
            if self.w_style == 1 {
                painter.text(pos, egui::Align2::CENTER_TOP, &text, font.clone(), egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 50));
            }
            painter.text(pos, egui::Align2::CENTER_TOP, text, font, col);
        }
        if self.show_date {
            painter.text(egui::pos2(pos.x, pos.y + size + 2.0), egui::Align2::CENTER_TOP,
                chrono::Local::now().format("%d.%m.%Y").to_string(), egui::FontId::proportional(13.0),
                egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 160));
        }
        if self.show_text {
            painter.text(egui::pos2(pos.x, rect.bottom() - 32.0), egui::Align2::CENTER_BOTTOM, &self.text_val, egui::FontId::proportional(16.0), col);
        }
        if self.show_battery {
            painter.text(egui::pos2(rect.right() - 12.0, rect.top() + 12.0), egui::Align2::RIGHT_TOP, "BAT ---%", egui::FontId::monospace(12.0), col);
        }
        if self.show_fps {
            painter.text(egui::pos2(rect.left() + 12.0, rect.bottom() - 12.0), egui::Align2::LEFT_BOTTOM, "60 FPS", egui::FontId::monospace(11.0), egui::Color32::from_gray(140));
        }
    }
}

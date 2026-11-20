#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PoiDesktop 5.1 — живые обои + анимации в духе CSS (pulse / float / glow / spin)

mod shapes;
mod wallpaper;

use eframe::egui;
use shapes::{code_variants, generate_mask, CODE_CHARS, Mask, SHAPE_NAMES, COLS};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("PoiDesktop"),
        ..Default::default()
    };
    eframe::run_native("PoiDesktop", options, Box::new(|cc| Box::new(App::new(cc))))
}

const NAV: &[&str] = &["Обзор", "Фигуры", "Вид", "Сцены", "Виджеты", "Обои"];
const BG_MODES: &[&str] = &["Чёрный", "Градиент", "Неон-дым", "Звёзды", "Закат", "Океан"];
const ANIM: &[&str] = &[
    "Spin (вращение)",
    "Pulse (пульс)",
    "Float (парение)",
    "Wave (волна)",
    "Glow breathe",
    "Orbit",
    "Статично",
];

/// CSS-like easing
fn ease_in_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

struct App {
    tab: usize,
    shape: usize,
    variant: usize,
    color: egui::Color32,
    accent: egui::Color32,
    bg: egui::Color32,
    bg_mode: usize,
    speed: f32,
    scale: f32,
    target_scale: f32,
    glow: f32,
    density: f32,
    anim: usize,
    time: Instant,
    status: String,
    mask_cache: Option<(usize, usize, Mask)>,
    show_clock: bool,
    show_date: bool,
    w_text: String,
    show_text: bool,
    w_size: f32,
    w_y: f32,
    stars: Vec<[f32; 3]>,
    // UI animation
    ui_pulse: f32,
    btn_hover: f32,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut st = (*cc.egui_ctx.style()).clone();
        st.visuals = egui::Visuals::dark();
        st.visuals.panel_fill = egui::Color32::from_rgb(12, 14, 20);
        st.visuals.selection.bg_fill = egui::Color32::from_rgb(40, 180, 140);
        st.spacing.item_spacing = egui::vec2(10.0, 8.0);
        cc.egui_ctx.set_style(st);

        let mut stars = Vec::new();
        for i in 0..200 {
            stars.push([
                ((i * 97) % 1000) as f32 / 1000.0,
                ((i * 53) % 1000) as f32 / 1000.0,
                0.25 + ((i * 17) % 75) as f32 / 100.0,
            ]);
        }

        Self {
            tab: 0,
            shape: 15,
            variant: 3,
            color: egui::Color32::from_rgb(90, 255, 180),
            accent: egui::Color32::from_rgb(100, 160, 255),
            bg: egui::Color32::from_rgb(4, 6, 12),
            bg_mode: 3,
            speed: 0.9,
            scale: 1.05,
            target_scale: 1.05,
            glow: 1.5,
            density: 1.0,
            anim: 0,
            time: Instant::now(),
            status: "PoiDesktop 5.1 · CSS-like анимации".into(),
            mask_cache: None,
            show_clock: true,
            show_date: true,
            w_text: "Poi".into(),
            show_text: false,
            w_size: 44.0,
            w_y: 0.1,
            stars,
            ui_pulse: 0.0,
            btn_hover: 0.0,
        }
    }

    fn t(&self) -> f32 {
        self.time.elapsed().as_secs_f32() * self.speed
    }

    fn mask(&mut self) -> Mask {
        if let Some((s, v, ref m)) = self.mask_cache {
            if s == self.shape && v == self.variant {
                return m.clone();
            }
        }
        let m = generate_mask(self.shape, self.variant);
        self.mask_cache = Some((self.shape, self.variant, m.clone()));
        m
    }

    fn apply_preset(&mut self, id: usize) {
        self.mask_cache = None;
        match id {
            0 => {
                self.shape = 15; self.variant = 3; self.bg_mode = 3;
                self.color = egui::Color32::from_rgb(120, 220, 255);
                self.accent = egui::Color32::from_rgb(180, 100, 255);
                self.anim = 0; self.glow = 1.7; self.target_scale = 1.08;
            }
            1 => {
                self.shape = 2; self.bg_mode = 1;
                self.color = egui::Color32::from_rgb(255, 70, 130);
                self.accent = egui::Color32::from_rgb(255, 40, 80);
                self.anim = 1; self.glow = 1.9;
            }
            2 => {
                self.shape = 1; self.bg_mode = 0;
                self.color = egui::Color32::from_rgb(40, 255, 100);
                self.anim = 3; self.glow = 1.3;
            }
            3 => {
                self.shape = 17; self.bg_mode = 3;
                self.color = egui::Color32::from_rgb(255, 160, 60);
                self.anim = 2;
            }
            4 => {
                self.shape = 7; self.bg_mode = 3;
                self.color = egui::Color32::from_rgb(200, 210, 255);
                self.anim = 2; self.glow = 1.6;
            }
            5 => {
                self.shape = 13; self.bg_mode = 4;
                self.color = egui::Color32::from_rgb(255, 140, 80);
                self.anim = 4;
            }
            6 => {
                self.shape = 23; self.bg_mode = 1;
                self.color = egui::Color32::from_rgb(255, 200, 140);
                self.anim = 2;
            }
            _ => {
                self.shape = 8; self.bg_mode = 2;
                self.color = egui::Color32::from_rgb(180, 220, 255);
                self.anim = 5; self.glow = 1.8;
            }
        }
        self.status = "Пресет + анимация применены".into();
    }

    fn apply_wallpaper(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        match self.render_png(&path, 1920, 1080) {
            Ok(()) => match wallpaper::set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Ошибка: {e}"),
            },
            Err(e) => self.status = format!("Рендер: {e}"),
        }
    }

    /// CSS-like transforms: spin / pulse / float / wave / glow / orbit
    fn cell_pos(&self, x: usize, y: usize, ox: f32, oy: f32, cell: f32, t: f32, rows: usize) -> (f32, f32, f32) {
        let mut px = ox + x as f32 * cell + cell * 0.5;
        let mut py = oy + y as f32 * cell + cell * 0.5;
        let mut extra_scale = 1.0;
        let nx = x as f32 / COLS as f32 - 0.5;
        let ny = y as f32 / rows as f32 - 0.5;
        let cx = ox + COLS as f32 * cell * 0.5;
        let cy = oy + rows as f32 * cell * 0.5;

        match self.anim {
            0 => {
                // spin — rotate 360 linear
                let a = t * 0.4;
                px = cx + (nx * a.cos() - ny * a.sin()) * COLS as f32 * cell;
                py = cy + (nx * a.sin() + ny * a.cos()) * rows as f32 * cell;
            }
            1 => {
                // pulse — scale ease-in-out via sin
                let p = ease_in_out((t * 1.1).sin() * 0.5 + 0.5);
                extra_scale = lerp(0.94, 1.08, p);
                px = cx + (px - cx) * extra_scale;
                py = cy + (py - cy) * extra_scale;
            }
            2 => {
                // float — translateY
                py += (t * 1.15).sin() * cell * 1.1;
                px += (t * 0.7).cos() * cell * 0.35;
            }
            3 => {
                // wave
                py += (t * 2.2 + x as f32 * 0.28).sin() * cell * 0.85;
            }
            4 => {
                // glow breathe — slight scale + stronger glow handled outside
                extra_scale = 1.0 + 0.04 * (t * 1.8).sin();
                px = cx + (px - cx) * extra_scale;
                py = cy + (py - cy) * extra_scale;
            }
            5 => {
                // orbit — rotate around center slower + float
                let a = t * 0.28;
                px = cx + (nx * a.cos() - ny * a.sin()) * COLS as f32 * cell;
                py = cy + (nx * a.sin() + ny * a.cos()) * rows as f32 * cell;
                py += (t * 0.9).sin() * cell * 0.5;
            }
            _ => {}
        }
        (px, py, extra_scale)
    }

    fn color_at(&self, depth: f32, x: usize, y: usize, t: f32) -> (u8, u8, u8) {
        let light = 0.5 + 0.5 * depth;
        // CSS-like brightness animation on glow-breathe
        let breathe = if self.anim == 4 {
            1.0 + 0.12 * (t * 1.8).sin()
        } else {
            1.0 + 0.06 * ((x + y) as f32 * 0.12 + t * 2.0).sin()
        };
        let m = depth;
        let r = (self.color.r() as f32 * (1.0 - m * 0.35) + self.accent.r() as f32 * m * 0.35) * light * breathe;
        let g = (self.color.g() as f32 * (1.0 - m * 0.35) + self.accent.g() as f32 * m * 0.35) * light * breathe;
        let b = (self.color.b() as f32 * (1.0 - m * 0.35) + self.accent.b() as f32 * m * 0.35) * light * breathe;
        (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    }

    fn render_png(&mut self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba, RgbaImage};
        let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([self.bg.r(), self.bg.g(), self.bg.b(), 255]));
        let t = if self.anim == 6 { 0.0 } else { self.t() };
        self.paint_bg_img(&mut img, w, h, t);
        let mask = self.mask();
        let cell = (h as f32 * 0.012 * self.scale).max(6.0);
        let ox = w as f32 * 0.5 - COLS as f32 * cell * 0.5;
        let oy = h as f32 * 0.48 - mask.rows as f32 * cell * 0.5;

        for y in 0..mask.rows {
            for x in 0..COLS {
                if !mask.on[y * COLS + x] { continue; }
                if self.density < 0.99 {
                    let step = (1.0 / self.density.max(0.3)) as usize;
                    if (x + y * 2) % step.max(1) != 0 { continue; }
                }
                let depth = mask.depth[y * COLS + x];
                let (px, py, _) = self.cell_pos(x, y, ox, oy, cell, t, mask.rows);
                let (r, g, b) = self.color_at(depth, x, y, t);
                let gmul = if self.anim == 4 { 1.0 + 0.35 * (t * 1.8).sin().abs() } else { 1.0 };
                soft_blob(&mut img, w, h, px, py, cell * (0.9 + depth * 0.5), r, g, b, 0.55 + depth * 0.4, self.glow * gmul);
            }
        }
        for y in 0..h {
            for x in 0..w {
                let nx = x as f32 / w as f32 * 2.0 - 1.0;
                let ny = y as f32 / h as f32 * 2.0 - 1.0;
                let d = (nx * nx + ny * ny).sqrt();
                let v = (1.0 - (d - 0.7).max(0.0) * 1.15).clamp(0.4, 1.0);
                let p = img.get_pixel_mut(x, y);
                p.0[0] = (p.0[0] as f32 * v) as u8;
                p.0[1] = (p.0[1] as f32 * v) as u8;
                p.0[2] = (p.0[2] as f32 * v) as u8;
            }
        }
        img.save(path).map_err(|e| e.to_string())
    }

    fn paint_bg_img(&self, img: &mut image::RgbaImage, w: u32, h: u32, t: f32) {
        use image::Rgba;
        match self.bg_mode {
            1 => {
                for y in 0..h {
                    let v = y as f32 / h as f32;
                    let r = mix(self.bg.r(), self.accent.r(), v * 0.4);
                    let g = mix(self.bg.g(), self.accent.g(), v * 0.4);
                    let b = mix(self.bg.b(), self.accent.b(), v * 0.4);
                    for x in 0..w {
                        img.put_pixel(x, y, Rgba([r, g, b, 255]));
                    }
                }
            }
            2 => {
                for y in (0..h).step_by(2) {
                    for x in (0..w).step_by(2) {
                        let nx = x as f32 / w as f32;
                        let ny = y as f32 / h as f32;
                        let v = ((nx * 4.0 + t).sin() + (ny * 5.0 - t * 0.5).cos()) * 0.25 + 0.5;
                        let c = self.accent;
                        let p = img.get_pixel_mut(x, y);
                        p.0[0] = p.0[0].saturating_add((v * c.r() as f32 * 0.28) as u8);
                        p.0[1] = p.0[1].saturating_add((v * c.g() as f32 * 0.28) as u8);
                        p.0[2] = p.0[2].saturating_add((v * c.b() as f32 * 0.28) as u8);
                    }
                }
            }
            3 => {
                for s in &self.stars {
                    let x = (s[0] * w as f32) as u32 % w;
                    let y = (((s[1] + t * 0.012 * s[2]) % 1.0) * h as f32) as u32 % h;
                    let b = (140.0 + 100.0 * s[2]) as u8;
                    img.put_pixel(x, y, Rgba([b, b, b.saturating_add(30), 255]));
                }
            }
            4 => {
                for y in 0..h {
                    let v = y as f32 / h as f32;
                    for x in 0..w {
                        img.put_pixel(x, y, Rgba([mix(30, 255, v * 0.9), mix(10, 90, v), mix(40, 60, 1.0 - v), 255]));
                    }
                }
            }
            5 => {
                for y in 0..h {
                    let v = y as f32 / h as f32;
                    for x in 0..w {
                        img.put_pixel(x, y, Rgba([mix(4, 20, v), mix(20, 120, v), mix(40, 200, v), 255]));
                    }
                }
            }
            _ => {}
        }
    }
}

fn mix(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t) * a as f32 + t * b as f32) as u8
}

fn soft_blob(
    img: &mut image::RgbaImage,
    w: u32, h: u32, cx: f32, cy: f32, radius: f32,
    r: u8, g: u8, b: u8, strength: f32, glow: f32,
) {
    let rad = radius * (1.0 + glow * 0.28);
    let min_x = ((cx - rad).floor() as i32).max(0) as u32;
    let max_x = ((cx + rad).ceil() as i32).min(w as i32 - 1) as u32;
    let min_y = ((cy - rad).floor() as i32).max(0) as u32;
    let max_y = ((cy + rad).ceil() as i32).min(h as i32 - 1) as u32;
    let inv = 1.0 / radius.max(1.0);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let d = ((x as f32 - cx).hypot(y as f32 - cy)) * inv;
            if d > 1.0 + glow * 0.45 { continue; }
            let core = (1.0 - d).max(0.0);
            let soft = (1.0 - d / (1.0 + glow * 0.45)).max(0.0).powf(1.7);
            let a = (core * 0.5 + soft * 0.5) * strength;
            if a < 0.02 { continue; }
            let p = img.get_pixel_mut(x, y);
            p.0[0] = ((1.0 - a) * p.0[0] as f32 + a * r as f32) as u8;
            p.0[1] = ((1.0 - a) * p.0[1] as f32 + a * g as f32) as u8;
            p.0[2] = ((1.0 - a) * p.0[2] as f32 + a * b as f32) as u8;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.anim != 6 {
            ctx.request_repaint();
        }
        let dt = ctx.input(|i| i.stable_dt).min(0.05);
        // CSS-like transition on scale
        self.scale = lerp(self.scale, self.target_scale, 1.0 - (-dt * 6.0).exp());
        self.ui_pulse = (self.ui_pulse + dt * 2.0) % 6.2832;

        egui::TopBottomPanel::bottom("nav")
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 12, 18)).inner_margin(egui::Margin::symmetric(8.0, 6.0)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    for (i, n) in NAV.iter().enumerate() {
                        let on = self.tab == i;
                        let label = if on {
                            egui::RichText::new(*n).strong().color(egui::Color32::from_rgb(90, 255, 200))
                        } else {
                            egui::RichText::new(*n).color(egui::Color32::from_gray(140))
                        };
                        if ui.selectable_label(on, label).clicked() {
                            self.tab = i;
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let pulse = 1.0 + 0.03 * self.ui_pulse.sin();
                        let btn = egui::Button::new(egui::RichText::new("  Установить обои  ").strong())
                            .fill(egui::Color32::from_rgb(30, 140, 110))
                            .rounding(10.0);
                        if ui.add_sized([140.0 * pulse, 32.0], btn).clicked() {
                            self.apply_wallpaper();
                        }
                    });
                });
                ui.label(egui::RichText::new(&self.status).small().color(egui::Color32::from_rgb(130, 220, 170)));
            });

        egui::SidePanel::left("side")
            .default_width(340.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(14, 16, 24)).inner_margin(egui::Margin::same(14.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                    0 => self.ui_home(ui),
                    1 => self.ui_shapes(ui),
                    2 => self.ui_look(ui),
                    3 => self.ui_scenes(ui),
                    4 => self.ui_widgets(ui),
                    _ => self.ui_wall(ui),
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.bg))
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                let t = if self.anim == 6 { 0.0 } else { self.t() };

                self.paint_bg_preview(&painter, rect, t);

                let mask = self.mask();
                let cell = (rect.height() * 0.012 * self.scale).max(7.5);
                let ox = rect.center().x - COLS as f32 * cell * 0.5;
                let oy = rect.center().y - mask.rows as f32 * cell * 0.5 - 6.0;
                let font = egui::FontId::monospace((cell * 0.92).clamp(9.0, 18.0));
                let glow_boost = if self.anim == 4 {
                    1.0 + 0.4 * (t * 1.8).sin().abs()
                } else {
                    1.0
                };

                for y in 0..mask.rows {
                    for x in 0..COLS {
                        if !mask.on[y * COLS + x] { continue; }
                        if self.density < 0.99 {
                            let step = (1.0 / self.density.max(0.3)) as usize;
                            if (x + y * 2) % step.max(1) != 0 { continue; }
                        }
                        let depth = mask.depth[y * COLS + x];
                        let (px, py, _) = self.cell_pos(x, y, ox, oy, cell, t, mask.rows);
                        let (r, g, b) = self.color_at(depth, x, y, t);
                        let pos = egui::pos2(px, py);
                        let rad = cell * (0.42 + depth * 0.2);

                        // multi-layer glow ≈ CSS box-shadow
                        if self.glow > 0.15 {
                            painter.circle_filled(
                                pos,
                                rad * (2.0 + self.glow * 0.4) * glow_boost,
                                egui::Color32::from_rgba_unmultiplied(r, g, b, 18),
                            );
                            painter.circle_filled(
                                pos,
                                rad * 1.35 * glow_boost,
                                egui::Color32::from_rgba_unmultiplied(r, g, b, 48),
                            );
                        }
                        let ch = CODE_CHARS[(x * 7 + y * 13 + (t as usize)) % CODE_CHARS.len()];
                        painter.text(pos, egui::Align2::CENTER_CENTER, ch, font.clone(), egui::Color32::from_rgb(r, g, b));
                        painter.circle_filled(
                            pos - egui::vec2(rad * 0.25, rad * 0.25),
                            rad * 0.2,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 30),
                        );
                    }
                }

                let wy = rect.top() + rect.height() * self.w_y;
                if self.show_clock {
                    let now = chrono::Local::now().format("%H:%M").to_string();
                    // CSS pulse on clock
                    let pulse = 1.0 + 0.02 * (t * 2.0).sin();
                    painter.text(
                        egui::pos2(rect.center().x, wy),
                        egui::Align2::CENTER_TOP,
                        now,
                        egui::FontId::monospace(self.w_size * pulse),
                        egui::Color32::from_rgba_unmultiplied(240, 255, 250, 220),
                    );
                }
                if self.show_date {
                    painter.text(
                        egui::pos2(rect.center().x, wy + self.w_size + 4.0),
                        egui::Align2::CENTER_TOP,
                        chrono::Local::now().format("%d.%m.%Y").to_string(),
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgba_unmultiplied(200, 220, 230, 160),
                    );
                }
                if self.show_text {
                    painter.text(
                        egui::pos2(rect.center().x, rect.bottom() - 36.0),
                        egui::Align2::CENTER_BOTTOM,
                        &self.w_text,
                        egui::FontId::proportional(18.0),
                        egui::Color32::from_white_alpha(200),
                    );
                }

                let title = format!(
                    "{} · {} · {}",
                    SHAPE_NAMES.get(self.shape).unwrap_or(&"?"),
                    code_variants(self.shape).get(self.variant).unwrap_or(&""),
                    ANIM.get(self.anim).unwrap_or(&"")
                );
                painter.text(
                    rect.left_top() + egui::vec2(16.0, 14.0),
                    egui::Align2::LEFT_TOP,
                    title,
                    egui::FontId::proportional(13.0),
                    egui::Color32::from_white_alpha(140),
                );
            });
    }
}

impl App {
    fn paint_bg_preview(&self, painter: &egui::Painter, rect: egui::Rect, t: f32) {
        match self.bg_mode {
            1 => {
                for i in 0..40 {
                    let v = i as f32 / 40.0;
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.left(), rect.top() + rect.height() * v),
                            egui::vec2(rect.width(), rect.height() / 40.0 + 1.0),
                        ),
                        0.0,
                        egui::Color32::from_rgb(
                            mix(self.bg.r(), self.accent.r(), v * 0.45),
                            mix(self.bg.g(), self.accent.g(), v * 0.45),
                            mix(self.bg.b(), self.accent.b(), v * 0.45),
                        ),
                    );
                }
            }
            2 => {
                painter.rect_filled(rect, 0.0, self.bg);
                let step = 14.0;
                let mut y = rect.top();
                while y < rect.bottom() {
                    let mut x = rect.left();
                    while x < rect.right() {
                        let nx = (x - rect.left()) / rect.width();
                        let ny = (y - rect.top()) / rect.height();
                        let v = ((nx * 4.0 + t).sin() + (ny * 5.0 - t * 0.5).cos()) * 0.25 + 0.5;
                        painter.rect_filled(
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(step, step)),
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(self.accent.r(), self.accent.g(), self.accent.b(), (v * 50.0) as u8),
                        );
                        x += step;
                    }
                    y += step;
                }
            }
            3 => {
                painter.rect_filled(rect, 0.0, self.bg);
                for s in &self.stars {
                    let y = (s[1] + t * 0.012 * s[2]) % 1.0;
                    // twinkle
                    let tw = 0.6 + 0.4 * ((t * 3.0 + s[0] * 10.0).sin());
                    painter.circle_filled(
                        egui::pos2(rect.left() + s[0] * rect.width(), rect.top() + y * rect.height()),
                        (0.8 + s[2] * 1.2) * tw,
                        egui::Color32::from_gray((150.0 + 90.0 * s[2] * tw) as u8),
                    );
                }
            }
            4 => {
                for i in 0..50 {
                    let v = i as f32 / 50.0;
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.left(), rect.top() + rect.height() * v),
                            egui::vec2(rect.width(), rect.height() / 50.0 + 1.0),
                        ),
                        0.0,
                        egui::Color32::from_rgb(mix(25, 255, v), mix(8, 80, v), mix(35, 50, 1.0 - v)),
                    );
                }
            }
            5 => {
                for i in 0..50 {
                    let v = i as f32 / 50.0;
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.left(), rect.top() + rect.height() * v),
                            egui::vec2(rect.width(), rect.height() / 50.0 + 1.0),
                        ),
                        0.0,
                        egui::Color32::from_rgb(mix(4, 25, v), mix(18, 140, v), mix(35, 210, v)),
                    );
                }
            }
            _ => painter.rect_filled(rect, 0.0, self.bg),
        }
    }

    fn ui_home(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("PoiDesktop").size(22.0).strong().color(egui::Color32::from_rgb(90, 255, 200)));
        ui.label("Живые обои · анимации Spin / Pulse / Float / Wave / Glow / Orbit");
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Пресеты").strong());
        let presets = [
            ("🪐 Сатурн", 0usize), ("💚 Сердце", 1), ("⌨️ Matrix", 2), ("🚀 Ракета", 3),
            ("🌙 Луна", 4), ("🌅 Закат", 5), ("🐱 Котик", 6), ("💎 Кристалл", 7),
        ];
        ui.horizontal_wrapped(|ui| {
            for (name, id) in presets {
                if ui.add_sized([100.0, 44.0], egui::Button::new(name).rounding(12.0)).clicked() {
                    self.apply_preset(id);
                }
            }
        });
        ui.add_space(10.0);
        if ui
            .add_sized(
                [ui.available_width(), 48.0],
                egui::Button::new(egui::RichText::new("✨ Установить обои").strong())
                    .fill(egui::Color32::from_rgb(30, 140, 110))
                    .rounding(12.0),
            )
            .clicked()
        {
            self.apply_wallpaper();
        }
        if ui.button("↩ Вернуть старые обои").clicked() {
            match wallpaper::restore_backup() {
                Ok(()) => self.status = "↩ Обои возвращены".into(),
                Err(e) => self.status = e,
            }
        }
    }

    fn ui_shapes(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Фигура").strong());
        ui.horizontal_wrapped(|ui| {
            for (i, n) in SHAPE_NAMES.iter().enumerate() {
                if ui.selectable_label(self.shape == i, *n).clicked() {
                    self.shape = i; self.variant = 0; self.mask_cache = None;
                }
            }
        });
        ui.separator();
        ui.label("Код-варианты");
        let vars: Vec<&str> = code_variants(self.shape).to_vec();
        ui.horizontal_wrapped(|ui| {
            for (i, v) in vars.iter().enumerate() {
                if ui.selectable_label(self.variant == i, *v).clicked() {
                    self.variant = i; self.mask_cache = None;
                }
            }
        });
    }

    fn ui_look(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Анимация (CSS-идеи)").strong());
        for (i, n) in ANIM.iter().enumerate() {
            if ui.selectable_label(self.anim == i, *n).clicked() {
                self.anim = i;
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Цвет"); ui.color_edit_button_srgba(&mut self.color);
            ui.label("Акцент"); ui.color_edit_button_srgba(&mut self.accent);
            ui.label("Фон"); ui.color_edit_button_srgba(&mut self.bg);
        });
        ui.add(egui::Slider::new(&mut self.speed, 0.2..=2.2).text("Скорость"));
        ui.add(egui::Slider::new(&mut self.target_scale, 0.6..=1.6).text("Масштаб"));
        ui.add(egui::Slider::new(&mut self.glow, 0.0..=2.5).text("Свечение"));
        ui.add(egui::Slider::new(&mut self.density, 0.35..=1.0).text("Плотность"));
    }

    fn ui_scenes(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Фон").strong());
        for (i, n) in BG_MODES.iter().enumerate() {
            if ui.selectable_label(self.bg_mode == i, *n).clicked() {
                self.bg_mode = i;
            }
        }
    }

    fn ui_widgets(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.show_clock, "Часы (pulse)");
        ui.checkbox(&mut self.show_date, "Дата");
        ui.checkbox(&mut self.show_text, "Текст");
        if self.show_text { ui.text_edit_singleline(&mut self.w_text); }
        ui.add(egui::Slider::new(&mut self.w_size, 20.0..=72.0).text("Размер часов"));
        ui.add(egui::Slider::new(&mut self.w_y, 0.03..=0.4).text("Позиция Y"));
    }

    fn ui_wall(&mut self, ui: &mut egui::Ui) {
        if ui.add_sized([ui.available_width(), 48.0],
            egui::Button::new("✨ Установить обои").fill(egui::Color32::from_rgb(30, 140, 110)).rounding(12.0)
        ).clicked() {
            self.apply_wallpaper();
        }
        if ui.button("↩ Вернуть").clicked() {
            let _ = wallpaper::restore_backup().map(|_| self.status = "↩ Ок".into());
        }
        if ui.button("💾 PNG").clicked() {
            let p = wallpaper::output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%H%M%S")));
            match self.render_png(&p, 1920, 1080) {
                Ok(()) => self.status = format!("{}", p.display()),
                Err(e) => self.status = e,
            }
        }
    }
}

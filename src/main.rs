// console mode for debug

mod shapes;
mod wallpaper;

use eframe::egui;
use shapes::{code_variants, generate_mask, CODE_CHARS, Mask, SHAPE_NAMES, COLS, ROWS};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("PoiDesktop"),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "PoiDesktop",
        options,
        Box::new(|cc| {
            let mut vis = egui::Visuals::dark();
            vis.panel_fill = egui::Color32::from_rgb(14, 16, 22);
            cc.egui_ctx.set_visuals(vis);
            Box::new(App::new())
        }),
    )
}

const NAV: &[&str] = &["Обзор", "Фигуры", "Вид", "Сцены", "Обои"];
const ANIM: &[&str] = &["Пульс", "Вращение", "Парение", "Волна", "Статично"];
const BG: &[&str] = &["Тёмный", "Градиент", "Звёзды", "Закат"];

struct App {
    tab: usize,
    shape: usize,
    variant: usize,
    color: egui::Color32,
    accent: egui::Color32,
    bg: egui::Color32,
    bg_mode: usize,
    anim: usize,
    speed: f32,
    scale: f32,
    glow: f32,
    time: Instant,
    status: String,
    mask: Mask,
    show_clock: bool,
    stars: Vec<[f32; 2]>,
}

impl App {
    fn new() -> Self {
        let shape = 3; // planet
        Self {
            tab: 0,
            shape,
            variant: 0,
            color: egui::Color32::from_rgb(80, 255, 180),
            accent: egui::Color32::from_rgb(100, 180, 255),
            bg: egui::Color32::from_rgb(8, 10, 16),
            bg_mode: 2,
            anim: 0,
            speed: 1.0,
            scale: 1.15,
            glow: 1.6,
            time: Instant::now(),
            status: "Готово · выбери пресет или установи обои".into(),
            mask: generate_mask(shape),
            show_clock: true,
            stars: (0..120)
                .map(|i| [((i * 47) % 100) as f32 / 100.0, ((i * 31) % 100) as f32 / 100.0])
                .collect(),
        }
    }

    fn t(&self) -> f32 {
        self.time.elapsed().as_secs_f32() * self.speed
    }

    fn set_shape(&mut self, s: usize) {
        self.shape = s;
        self.variant = 0;
        self.mask = generate_mask(s);
    }

    fn preset(&mut self, id: usize) {
        match id {
            0 => {
                self.set_shape(3);
                self.color = egui::Color32::from_rgb(100, 230, 255);
                self.bg_mode = 2;
                self.anim = 1;
            }
            1 => {
                self.set_shape(1);
                self.color = egui::Color32::from_rgb(255, 80, 140);
                self.anim = 0;
                self.bg_mode = 1;
            }
            2 => {
                self.set_shape(0);
                self.color = egui::Color32::from_rgb(40, 255, 100);
                self.anim = 3;
                self.bg_mode = 0;
            }
            3 => {
                self.set_shape(4);
                self.color = egui::Color32::from_rgb(255, 160, 50);
                self.anim = 2;
            }
            4 => {
                self.set_shape(5);
                self.color = egui::Color32::from_rgb(200, 210, 255);
                self.bg_mode = 2;
            }
            _ => {
                self.set_shape(6);
                self.color = egui::Color32::from_rgb(255, 200, 140);
            }
        }
        self.status = format!("Пресет: {}", SHAPE_NAMES[self.shape]);
    }

    fn apply_wallpaper(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        match self.render_png(&path, 1920, 1080) {
            Ok(()) => match wallpaper::set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Ошибка обоев: {e}"),
            },
            Err(e) => self.status = format!("Рендер: {e}"),
        }
    }

    fn pos(&self, x: usize, y: usize, ox: f32, oy: f32, cell: f32, t: f32) -> (f32, f32) {
        let mut px = ox + x as f32 * cell + cell * 0.5;
        let mut py = oy + y as f32 * cell + cell * 0.5;
        let cx = ox + COLS as f32 * cell * 0.5;
        let cy = oy + ROWS as f32 * cell * 0.5;
        let dx = px - cx;
        let dy = py - cy;
        match self.anim {
            0 => {
                let s = 1.0 + 0.07 * (t * 2.2).sin();
                px = cx + dx * s;
                py = cy + dy * s;
            }
            1 => {
                let a = t * 0.5;
                px = cx + dx * a.cos() - dy * a.sin();
                py = cy + dx * a.sin() + dy * a.cos();
            }
            2 => {
                py += (t * 1.2).sin() * cell * 1.2;
            }
            3 => {
                py += (t * 2.0 + x as f32 * 0.3).sin() * cell * 0.9;
            }
            _ => {}
        }
        (px, py)
    }

    fn col(&self, depth: f32) -> egui::Color32 {
        let l = 0.55 + 0.45 * depth;
        let r = (self.color.r() as f32 * l).min(255.0) as u8;
        let g = (self.color.g() as f32 * l).min(255.0) as u8;
        let b = (self.color.b() as f32 * l).min(255.0) as u8;
        egui::Color32::from_rgb(r, g, b)
    }

    fn render_png(&self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba, RgbaImage};
        let mut img: RgbaImage = ImageBuffer::from_pixel(w, h, Rgba([self.bg.r(), self.bg.g(), self.bg.b(), 255]));
        let t = if self.anim == 4 { 0.0 } else { self.t() };
        // stars
        if self.bg_mode == 2 {
            for s in &self.stars {
                let x = (s[0] * w as f32) as u32 % w;
                let y = (s[1] * h as f32) as u32 % h;
                img.put_pixel(x, y, Rgba([180, 190, 210, 255]));
            }
        }
        let cell = (h as f32 * 0.014 * self.scale).max(8.0);
        let ox = w as f32 * 0.5 - COLS as f32 * cell * 0.5;
        let oy = h as f32 * 0.48 - ROWS as f32 * cell * 0.5;
        for y in 0..ROWS {
            for x in 0..COLS {
                let i = y * COLS + x;
                if !self.mask.on[i] {
                    continue;
                }
                let depth = self.mask.depth[i];
                let (px, py) = self.pos(x, y, ox, oy, cell, t);
                let c = self.col(depth);
                let rad = cell * 0.55;
                let r0 = ((rad * (1.0 + self.glow * 0.5)) as i32).max(2);
                for dy in -r0..=r0 {
                    for dx in -r0..=r0 {
                        let d = ((dx * dx + dy * dy) as f32).sqrt() / rad;
                        if d > 1.0 + self.glow * 0.4 {
                            continue;
                        }
                        let a = ((1.0 - d / (1.0 + self.glow * 0.4)).max(0.0).powf(1.5) * (0.5 + depth * 0.5)).clamp(0.0, 1.0);
                        let xx = px as i32 + dx;
                        let yy = py as i32 + dy;
                        if xx < 0 || yy < 0 || xx >= w as i32 || yy >= h as i32 {
                            continue;
                        }
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.anim != 4 {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::bottom("nav").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, n) in NAV.iter().enumerate() {
                    if ui.selectable_label(self.tab == i, *n).clicked() {
                        self.tab = i;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Установить обои").clicked() {
                        self.apply_wallpaper();
                    }
                });
            });
            ui.label(&self.status);
        });

        egui::SidePanel::left("side").default_width(300.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                0 => {
                    ui.heading("PoiDesktop");
                    ui.label("Пресеты:");
                    ui.horizontal_wrapped(|ui| {
                        for (name, id) in [
                            ("Планета", 0usize),
                            ("Сердце", 1),
                            ("Шар", 2),
                            ("Ракета", 3),
                            ("Луна", 4),
                            ("Котик", 5),
                        ] {
                            if ui.button(name).clicked() {
                                self.preset(id);
                            }
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button("✨ Установить обои").clicked() {
                        self.apply_wallpaper();
                    }
                    if ui.button("↩ Вернуть обои").clicked() {
                        match wallpaper::restore_backup() {
                            Ok(()) => self.status = "Обои возвращены".into(),
                            Err(e) => self.status = e,
                        }
                    }
                    ui.checkbox(&mut self.show_clock, "Часы");
                }
                1 => {
                    ui.label("Фигура");
                    for (i, n) in SHAPE_NAMES.iter().enumerate() {
                        if ui.selectable_label(self.shape == i, *n).clicked() {
                            self.set_shape(i);
                        }
                    }
                    ui.separator();
                    ui.label("Вариант");
                    for (i, v) in code_variants(self.shape).iter().enumerate() {
                        if ui.selectable_label(self.variant == i, *v).clicked() {
                            self.variant = i;
                        }
                    }
                }
                2 => {
                    ui.label("Анимация");
                    for (i, n) in ANIM.iter().enumerate() {
                        if ui.selectable_label(self.anim == i, *n).clicked() {
                            self.anim = i;
                        }
                    }
                    ui.add(egui::Slider::new(&mut self.speed, 0.2..=2.5).text("Скорость"));
                    ui.add(egui::Slider::new(&mut self.scale, 0.7..=1.8).text("Масштаб"));
                    ui.add(egui::Slider::new(&mut self.glow, 0.0..=2.5).text("Свечение"));
                    ui.horizontal(|ui| {
                        ui.label("Цвет");
                        ui.color_edit_button_srgba(&mut self.color);
                        ui.label("Фон");
                        ui.color_edit_button_srgba(&mut self.bg);
                    });
                }
                3 => {
                    ui.label("Фон");
                    for (i, n) in BG.iter().enumerate() {
                        if ui.selectable_label(self.bg_mode == i, *n).clicked() {
                            self.bg_mode = i;
                        }
                    }
                }
                _ => {
                    if ui.button("Установить обои").clicked() {
                        self.apply_wallpaper();
                    }
                    if ui.button("Сохранить PNG").clicked() {
                        let p = wallpaper::output_dir()
                            .join(format!("poi_{}.png", chrono::Local::now().format("%H%M%S")));
                        match self.render_png(&p, 1920, 1080) {
                            Ok(()) => self.status = format!("PNG: {}", p.display()),
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button("Вернуть старые").clicked() {
                        let _ = wallpaper::restore_backup().map(|_| self.status = "Ок".into());
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let t = if self.anim == 4 { 0.0 } else { self.t() };

            // background always
            painter.rect_filled(rect, 0.0, self.bg);
            if self.bg_mode == 1 {
                for i in 0..30 {
                    let v = i as f32 / 30.0;
                    let y = rect.top() + rect.height() * v;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(rect.left(), y), egui::vec2(rect.width(), rect.height() / 30.0 + 1.0)),
                        0.0,
                        egui::Color32::from_rgb(
                            ((1.0 - v) * self.bg.r() as f32 + v * self.accent.r() as f32 * 0.4) as u8,
                            ((1.0 - v) * self.bg.g() as f32 + v * self.accent.g() as f32 * 0.4) as u8,
                            ((1.0 - v) * self.bg.b() as f32 + v * self.accent.b() as f32 * 0.4) as u8,
                        ),
                    );
                }
            }
            if self.bg_mode == 2 {
                for s in &self.stars {
                    painter.circle_filled(
                        egui::pos2(rect.left() + s[0] * rect.width(), rect.top() + s[1] * rect.height()),
                        1.4,
                        egui::Color32::from_gray(180),
                    );
                }
            }
            if self.bg_mode == 3 {
                for i in 0..30 {
                    let v = i as f32 / 30.0;
                    let y = rect.top() + rect.height() * v;
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(rect.left(), y), egui::vec2(rect.width(), rect.height() / 30.0 + 1.0)),
                        0.0,
                        egui::Color32::from_rgb(
                            (30.0 + 200.0 * v) as u8,
                            (10.0 + 70.0 * v) as u8,
                            (40.0 + 20.0 * (1.0 - v)) as u8,
                        ),
                    );
                }
            }

            // figure — large bright dots
            let cell = (rect.height().min(rect.width()) * 0.016 * self.scale).max(10.0);
            let ox = rect.center().x - COLS as f32 * cell * 0.5;
            let oy = rect.center().y - ROWS as f32 * cell * 0.5;
            let font = egui::FontId::monospace((cell * 0.85).clamp(10.0, 20.0));
            let mut drawn = 0u32;

            for y in 0..ROWS {
                for x in 0..COLS {
                    let i = y * COLS + x;
                    if !self.mask.on[i] {
                        continue;
                    }
                    let depth = self.mask.depth[i];
                    let (px, py) = self.pos(x, y, ox, oy, cell, t);
                    let c = self.col(depth);
                    let pos = egui::pos2(px, py);
                    let rad = (cell * 0.48).max(3.0);

                    if self.glow > 0.2 {
                        painter.circle_filled(
                            pos,
                            rad * (1.8 + self.glow * 0.3),
                            egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 40),
                        );
                    }
                    painter.circle_filled(pos, rad, c);
                    let ch = CODE_CHARS[(x + y * 3) % CODE_CHARS.len()];
                    painter.text(pos, egui::Align2::CENTER_CENTER, ch, font.clone(), egui::Color32::WHITE);
                    drawn += 1;
                }
            }

            // debug label so user always sees something
            painter.text(
                rect.left_top() + egui::vec2(12.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!(
                    "{} · точек: {} · {}",
                    SHAPE_NAMES.get(self.shape).unwrap_or(&"?"),
                    drawn,
                    ANIM[self.anim]
                ),
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(180, 255, 210),
            );

            if self.show_clock {
                let now = chrono::Local::now().format("%H:%M:%S").to_string();
                painter.text(
                    egui::pos2(rect.center().x, rect.top() + 40.0),
                    egui::Align2::CENTER_TOP,
                    now,
                    egui::FontId::monospace(36.0),
                    egui::Color32::from_rgba_unmultiplied(230, 255, 240, 230),
                );
            }

            if drawn == 0 {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Нет точек маски — смени фигуру",
                    egui::FontId::proportional(18.0),
                    egui::Color32::RED,
                );
            }
        });
    }
}

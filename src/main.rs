#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shapes;
mod wallpaper;

use eframe::egui;
use shapes::{code_variants, generate_mesh, mesh_from_image, ShapeMesh, SHAPE_NAMES};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 820.0])
            .with_min_inner_size([1000.0, 660.0])
            .with_title("PoiDesktop"),
        ..Default::default()
    };
    eframe::run_native(
        "PoiDesktop",
        options,
        Box::new(|cc| Box::new(PoiApp::new(cc))),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Tab { Design, Motion, Widgets, Scene, Export }

#[derive(Clone, Copy, PartialEq)]
enum Anim { Spin, Pulse, Float, Swing, Morph, Still }

struct PoiApp {
    tab: Tab,
    shape: usize,
    variant: usize,
    color: egui::Color32,
    accent: egui::Color32,
    bg: egui::Color32,
    speed: f32,
    scale: f32,
    quality: usize,
    glow: f32,
    soft: f32,
    rainbow: bool,
    anim: Anim,
    scene: usize,
    depth: f32,
    time: Instant,
    status: String,
    custom: Option<ShapeMesh>,
    custom_name: String,
    thr: u8,
    // widgets
    w_clock: bool,
    w_date: bool,
    w_text: bool,
    w_text_val: String,
    w_size: f32,
    w_color: egui::Color32,
    w_opacity: f32,
    w_y: f32,
    // style
    vignette: bool,
    bloom: bool,
    auto: bool,
    auto_t: f32,
    stars: Vec<[f32; 2]>,
}

impl PoiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut st = (*cc.egui_ctx.style()).clone();
        st.visuals = egui::Visuals::dark();
        st.visuals.window_rounding = 8.0.into();
        st.visuals.widgets.noninteractive.rounding = 6.0.into();
        st.visuals.widgets.inactive.rounding = 6.0.into();
        st.visuals.widgets.hovered.rounding = 6.0.into();
        st.visuals.widgets.active.rounding = 6.0.into();
        st.spacing.item_spacing = egui::vec2(10.0, 8.0);
        cc.egui_ctx.set_style(st);

        let mut stars = Vec::new();
        for i in 0..80 {
            let x = ((i * 73) % 100) as f32 / 100.0;
            let y = ((i * 37) % 100) as f32 / 100.0;
            stars.push([x, y]);
        }

        Self {
            tab: Tab::Design,
            shape: 15,
            variant: 0,
            color: egui::Color32::from_rgb(90, 220, 255),
            accent: egui::Color32::from_rgb(255, 120, 200),
            bg: egui::Color32::from_rgb(8, 10, 18),
            speed: 1.0,
            scale: 1.05,
            quality: 2,
            glow: 1.2,
            soft: 1.0,
            rainbow: false,
            anim: Anim::Spin,
            scene: 1,
            depth: 1.0,
            time: Instant::now(),
            status: "Готово · объёмный рендер · настрой и поставь обои".into(),
            custom: None,
            custom_name: String::new(),
            thr: 100,
            w_clock: true,
            w_date: false,
            w_text: false,
            w_text_val: "PoiDesktop".into(),
            w_size: 36.0,
            w_color: egui::Color32::from_rgb(240, 248, 255),
            w_opacity: 0.9,
            w_y: 0.12,
            vignette: true,
            bloom: true,
            auto: false,
            auto_t: 0.0,
            stars,
        }
    }

    fn t(&self) -> f32 {
        self.time.elapsed().as_secs_f32() * self.speed
    }

    fn mesh(&self) -> ShapeMesh {
        if let Some(ref m) = self.custom {
            m.clone()
        } else {
            generate_mesh(self.shape, self.variant, self.quality)
        }
    }

    fn transform(&self, x: f32, y: f32, z: f32, t: f32) -> (f32, f32, f32) {
        let mut px = x - 0.5;
        let mut py = y - 0.5;
        match self.anim {
            Anim::Still => {}
            Anim::Spin => {
                let a = t * 0.55;
                let rx = px * a.cos() - py * a.sin();
                let ry = px * a.sin() + py * a.cos();
                px = rx;
                py = ry;
            }
            Anim::Pulse => {
                let s = 1.0 + 0.07 * (t * 2.2).sin();
                px *= s;
                py *= s;
            }
            Anim::Float => {
                py += 0.04 * (t * 1.3).sin();
                px += 0.02 * (t * 0.9).cos();
            }
            Anim::Swing => {
                let a = 0.25 * (t * 1.1).sin();
                let rx = px * a.cos() - py * a.sin();
                let ry = px * a.sin() + py * a.cos();
                px = rx;
                py = ry;
            }
            Anim::Morph => {
                let s = 1.0 + 0.05 * (t * 1.5).sin();
                let a = 0.15 * (t * 0.8).cos();
                px = (px * a.cos() - py * a.sin()) * s;
                py = (px * a.sin() + py * a.cos()) * s;
            }
        }
        let d = 1.0 + (z - 0.5) * 0.25 * self.depth;
        (px / d, py / d, z)
    }

    fn col_at(&self, z: f32, seed: f32, t: f32) -> (u8, u8, u8) {
        let light = (0.55 + 0.45 * z).clamp(0.35, 1.0);
        if self.rainbow {
            let h = (seed * 0.15 + t * 0.12 + z) % 1.0;
            let r = ((h * 255.0) as u8).wrapping_add(30);
            let g = (((h + 0.33) % 1.0) * 230.0) as u8;
            let b = (((h + 0.66) % 1.0) * 255.0) as u8;
            (
                (r as f32 * light) as u8,
                (g as f32 * light) as u8,
                (b as f32 * light) as u8,
            )
        } else {
            let m = z.clamp(0.0, 1.0);
            let r = (self.color.r() as f32 * (1.0 - m * 0.4) + self.accent.r() as f32 * m * 0.4) * light;
            let g = (self.color.g() as f32 * (1.0 - m * 0.4) + self.accent.g() as f32 * m * 0.4) * light;
            let b = (self.color.b() as f32 * (1.0 - m * 0.4) + self.accent.b() as f32 * m * 0.4) * light;
            (r as u8, g as u8, b as u8)
        }
    }

    fn apply_wp(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        match self.render_file(&path, 1920, 1080) {
            Ok(()) => match wallpaper::set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Ошибка: {e}"),
            },
            Err(e) => self.status = format!("Рендер: {e}"),
        }
    }

    fn render_file(&self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba};
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(w, h, Rgba([self.bg.r(), self.bg.g(), self.bg.b(), 255]));

        // scene
        if self.scene == 1 {
            for (i, s) in self.stars.iter().enumerate() {
                let x = (s[0] * w as f32) as u32;
                let y = (s[1] * h as f32) as u32;
                if x < w && y < h {
                    let b = 120 + (i % 100) as u8;
                    img.put_pixel(x, y, Rgba([b, b, b.saturating_add(30), 255]));
                }
            }
        } else if self.scene == 2 {
            for y in 0..h {
                let tt = y as f32 / h as f32;
                let r = mix(self.bg.r(), self.accent.r(), tt * 0.35);
                let g = mix(self.bg.g(), self.accent.g(), tt * 0.35);
                let b = mix(self.bg.b(), self.accent.b(), tt * 0.35);
                for x in 0..w {
                    img.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }

        let mut mesh = self.mesh();
        mesh.blobs.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
        let t = if self.anim == Anim::Still { 0.0 } else { self.t() };
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.48;
        let sc = self.scale * h as f32 * 0.42;

        for (i, bl) in mesh.blobs.iter().enumerate() {
            let (px, py, z) = self.transform(bl.x, bl.y, bl.z, t);
            let x = cx + px * sc * 2.0;
            let y = cy + py * sc * 2.0;
            let radius = bl.r * sc * 2.2 * (0.85 + self.soft * 0.3);
            let (cr, cg, cb) = self.col_at(z, i as f32, t);
            soft_circle(&mut img, w, h, x, y, radius, cr, cg, cb, self.glow);
        }

        // vignette
        if self.vignette {
            for y in 0..h {
                for x in 0..w {
                    let nx = x as f32 / w as f32 * 2.0 - 1.0;
                    let ny = y as f32 / h as f32 * 2.0 - 1.0;
                    let d = (nx * nx + ny * ny).sqrt();
                    let v = (1.0 - (d - 0.7).max(0.0) * 1.2).clamp(0.4, 1.0);
                    let p = img.get_pixel_mut(x, y);
                    p.0[0] = (p.0[0] as f32 * v) as u8;
                    p.0[1] = (p.0[1] as f32 * v) as u8;
                    p.0[2] = (p.0[2] as f32 * v) as u8;
                }
            }
        }

        // widgets bake
        // (simple text omitted in file bake for speed — clock drawn as blocks if needed)
        img.save(path).map_err(|e| e.to_string())
    }

    fn load_img(&mut self) {
        if let Some(p) = rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            match mesh_from_image(&p, self.thr) {
                Ok(m) => {
                    self.custom_name = p
                        .file_stem()
                        .map(|s| s.to_string_lossy().into())
                        .unwrap_or_else(|| "Фото".into());
                    self.status = format!("📷 {} · {} объёмов", self.custom_name, m.blobs.len());
                    self.custom = Some(m);
                }
                Err(e) => self.status = format!("Фото: {e}"),
            }
        }
    }
}

fn mix(a: u8, b: u8, t: f32) -> u8 {
    ((1.0 - t) * a as f32 + t * b as f32) as u8
}

fn soft_circle(
    img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    w: u32,
    h: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    r: u8,
    g: u8,
    b: u8,
    glow: f32,
) {
    let rad = radius * (1.0 + glow * 0.35);
    let min_x = ((cx - rad).floor() as i32).max(0) as u32;
    let max_x = ((cx + rad).ceil() as i32).min(w as i32 - 1) as u32;
    let min_y = ((cy - rad).floor() as i32).max(0) as u32;
    let max_y = ((cy + rad).ceil() as i32).min(h as i32 - 1) as u32;
    let inv = 1.0 / radius.max(1.0);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt() * inv;
            if d > 1.0 + glow * 0.5 {
                continue;
            }
            // smooth falloff for volume
            let core = (1.0 - d).max(0.0);
            let soft = (1.0 - d / (1.0 + glow * 0.5)).max(0.0).powf(1.6);
            let a = (core * 0.55 + soft * 0.45).clamp(0.0, 1.0) * 0.9;
            if a < 0.01 {
                continue;
            }
            let p = img.get_pixel_mut(x, y);
            p.0[0] = ((1.0 - a) * p.0[0] as f32 + a * r as f32) as u8;
            p.0[1] = ((1.0 - a) * p.0[1] as f32 + a * g as f32) as u8;
            p.0[2] = ((1.0 - a) * p.0[2] as f32 + a * b as f32) as u8;
        }
    }
}

impl eframe::App for PoiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.anim != Anim::Still {
            ctx.request_repaint();
        }
        if self.auto {
            self.auto_t += ctx.input(|i| i.stable_dt);
            if self.auto_t > 7.0 {
                self.auto_t = 0.0;
                self.shape = (self.shape + 1) % SHAPE_NAMES.len();
                self.variant = 0;
                self.custom = None;
            }
        }

        egui::TopBottomPanel::top("bar").frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(12, 14, 22))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
        ).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("PoiDesktop").size(20.0).strong().color(egui::Color32::from_rgb(120, 220, 255)));
                ui.add_space(12.0);
                for (t, n) in [
                    (Tab::Design, "Дизайн"),
                    (Tab::Motion, "Движение"),
                    (Tab::Widgets, "Виджеты"),
                    (Tab::Scene, "Сцена"),
                    (Tab::Export, "Обои"),
                ] {
                    let sel = self.tab == t;
                    let label = if sel {
                        egui::RichText::new(n).strong().color(egui::Color32::from_rgb(100, 230, 200))
                    } else {
                        egui::RichText::new(n).color(egui::Color32::from_gray(160))
                    };
                    if ui.selectable_label(sel, label).clicked() {
                        self.tab = t;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn = egui::Button::new(egui::RichText::new("  Установить обои  ").strong())
                        .fill(egui::Color32::from_rgb(30, 120, 100))
                        .rounding(8.0);
                    if ui.add(btn).clicked() {
                        self.apply_wp();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("st").frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(10, 12, 18))
                .inner_margin(egui::Margin::symmetric(14.0, 6.0)),
        ).show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.status).size(13.0).color(egui::Color32::from_rgb(140, 255, 190)));
        });

        egui::SidePanel::left("side")
            .default_width(350.0)
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 16, 26))
                    .inner_margin(egui::Margin::same(14.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                    Tab::Design => self.ui_design(ui),
                    Tab::Motion => self.ui_motion(ui),
                    Tab::Widgets => self.ui_widgets(ui),
                    Tab::Scene => self.ui_scene(ui),
                    Tab::Export => self.ui_export(ui),
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(self.bg))
            .show(ctx, |ui| {
                let avail = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(avail, egui::Sense::hover());
                let painter = ui.painter_at(rect);

                // stars
                if self.scene == 1 {
                    for s in &self.stars {
                        let p = egui::pos2(
                            rect.left() + s[0] * rect.width(),
                            rect.top() + s[1] * rect.height(),
                        );
                        painter.circle_filled(p, 1.2, egui::Color32::from_gray(180));
                    }
                }

                let mut mesh = self.mesh();
                mesh.blobs.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
                let t = if self.anim == Anim::Still { 0.0 } else { self.t() };
                let cx = rect.center().x;
                let cy = rect.center().y - 8.0;
                let sc = self.scale * rect.height().min(rect.width()) * 0.42;

                for (i, bl) in mesh.blobs.iter().enumerate() {
                    let (px, py, z) = self.transform(bl.x, bl.y, bl.z, t);
                    let pos = egui::pos2(cx + px * sc * 2.0, cy + py * sc * 2.0);
                    let radius = bl.r * sc * 2.2 * (0.85 + self.soft * 0.3);
                    let (r, g, b) = self.col_at(z, i as f32, t);

                    // soft glow layers for volume look
                    if self.bloom {
                        painter.circle_filled(
                            pos,
                            radius * (1.6 + self.glow * 0.4),
                            egui::Color32::from_rgba_unmultiplied(r, g, b, 28),
                        );
                        painter.circle_filled(
                            pos,
                            radius * 1.25,
                            egui::Color32::from_rgba_unmultiplied(r, g, b, 55),
                        );
                    }
                    painter.circle_filled(
                        pos,
                        radius,
                        egui::Color32::from_rgba_unmultiplied(r, g, b, 210),
                    );
                    // specular highlight
                    let hx = pos.x - radius * 0.28;
                    let hy = pos.y - radius * 0.28;
                    painter.circle_filled(
                        egui::pos2(hx, hy),
                        radius * 0.28,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 40),
                    );
                }

                // widgets
                let a = (self.w_opacity * 255.0) as u8;
                let wc = egui::Color32::from_rgba_unmultiplied(
                    self.w_color.r(),
                    self.w_color.g(),
                    self.w_color.b(),
                    a,
                );
                let wy = rect.top() + rect.height() * self.w_y;
                if self.w_clock {
                    let now = chrono::Local::now().format("%H:%M").to_string();
                    painter.text(
                        egui::pos2(rect.center().x, wy),
                        egui::Align2::CENTER_TOP,
                        now,
                        egui::FontId::monospace(self.w_size),
                        wc,
                    );
                }
                if self.w_date {
                    let d = chrono::Local::now().format("%d %B").to_string();
                    painter.text(
                        egui::pos2(rect.center().x, wy + self.w_size + 2.0),
                        egui::Align2::CENTER_TOP,
                        d,
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgba_unmultiplied(self.w_color.r(), self.w_color.g(), self.w_color.b(), a / 2),
                    );
                }
                if self.w_text {
                    painter.text(
                        egui::pos2(rect.center().x, rect.bottom() - 40.0),
                        egui::Align2::CENTER_BOTTOM,
                        &self.w_text_val,
                        egui::FontId::proportional(18.0),
                        wc,
                    );
                }

                let title = if self.custom.is_some() {
                    format!("📷 {}", self.custom_name)
                } else {
                    format!(
                        "{}  ·  {}",
                        SHAPE_NAMES.get(self.shape).unwrap_or(&"?"),
                        code_variants(self.shape).get(self.variant).unwrap_or(&"")
                    )
                };
                painter.text(
                    rect.left_top() + egui::vec2(16.0, 14.0),
                    egui::Align2::LEFT_TOP,
                    title,
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_white_alpha(140),
                );
            });
    }
}

impl PoiApp {
    fn ui_design(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Фигура").size(15.0).strong());
        if self.custom.is_some() {
            ui.horizontal(|ui| {
                ui.label(format!("📷 {}", self.custom_name));
                if ui.button("Сброс").clicked() {
                    self.custom = None;
                }
            });
        }
        egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, n) in SHAPE_NAMES.iter().enumerate() {
                    if ui.selectable_label(self.shape == i && self.custom.is_none(), *n).clicked() {
                        self.shape = i;
                        self.variant = 0;
                        self.custom = None;
                    }
                }
            });
        });
        ui.add_space(4.0);
        ui.label("Код-вариант");
        ui.horizontal_wrapped(|ui| {
            for (i, v) in code_variants(self.shape).iter().enumerate() {
                if ui.selectable_label(self.variant == i, *v).clicked() {
                    self.variant = i;
                }
            }
        });
        ui.separator();
        ui.label(egui::RichText::new("Фото → объём").strong());
        ui.add(egui::Slider::new(&mut self.thr, 30..=180).text("Порог"));
        if ui.button("📂 Загрузить изображение").clicked() {
            self.load_img();
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Цвет");
            ui.color_edit_button_srgba(&mut self.color);
            ui.label("Акцент");
            ui.color_edit_button_srgba(&mut self.accent);
            ui.label("Фон");
            ui.color_edit_button_srgba(&mut self.bg);
        });
        ui.checkbox(&mut self.rainbow, "Радуга");
        ui.add(egui::Slider::new(&mut self.scale, 0.5..=1.8).text("Масштаб"));
        ui.add(egui::Slider::new(&mut self.quality, 1..=3).text("Качество объёма"));
    }

    fn ui_motion(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Анимация").size(15.0).strong());
        for (a, n) in [
            (Anim::Spin, "Вращение"),
            (Anim::Pulse, "Пульс"),
            (Anim::Float, "Парение"),
            (Anim::Swing, "Качание"),
            (Anim::Morph, "Морфинг"),
            (Anim::Still, "Статично"),
        ] {
            if ui.selectable_label(self.anim == a, n).clicked() {
                self.anim = a;
            }
        }
        ui.add(egui::Slider::new(&mut self.speed, 0.2..=2.5).text("Скорость"));
        ui.separator();
        ui.label(egui::RichText::new("Объём и свет").strong());
        ui.add(egui::Slider::new(&mut self.depth, 0.0..=2.0).text("Глубина"));
        ui.add(egui::Slider::new(&mut self.glow, 0.0..=2.5).text("Свечение"));
        ui.add(egui::Slider::new(&mut self.soft, 0.4..=2.0).text("Мягкость"));
        ui.checkbox(&mut self.bloom, "Bloom / объёмное свечение");
        ui.checkbox(&mut self.auto, "Автосмена фигур");
    }

    fn ui_widgets(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Виджеты").size(15.0).strong());
        ui.checkbox(&mut self.w_clock, "Часы");
        ui.checkbox(&mut self.w_date, "Дата");
        ui.checkbox(&mut self.w_text, "Текст");
        if self.w_text {
            ui.text_edit_singleline(&mut self.w_text_val);
        }
        ui.add(egui::Slider::new(&mut self.w_size, 18.0..=72.0).text("Размер часов"));
        ui.add(egui::Slider::new(&mut self.w_y, 0.02..=0.4).text("Позиция по Y"));
        ui.add(egui::Slider::new(&mut self.w_opacity, 0.2..=1.0).text("Прозрачность"));
        ui.horizontal(|ui| {
            ui.label("Цвет");
            ui.color_edit_button_srgba(&mut self.w_color);
        });
    }

    fn ui_scene(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Сцена").size(15.0).strong());
        for (i, n) in ["Пустая", "Звёзды", "Градиент"].iter().enumerate() {
            if ui.selectable_label(self.scene == i, *n).clicked() {
                self.scene = i;
            }
        }
        ui.checkbox(&mut self.vignette, "Виньетка");
        ui.separator();
        ui.label("Пресеты");
        if ui.button("🌌 Планета Сатурн").clicked() {
            self.shape = 15; self.variant = 3; self.scene = 1; self.anim = Anim::Spin;
            self.color = egui::Color32::from_rgb(120, 200, 255); self.custom = None;
        }
        if ui.button("💚 Пульс сердца").clicked() {
            self.shape = 22; self.anim = Anim::Pulse; self.color = egui::Color32::from_rgb(255, 80, 140);
            self.glow = 1.6; self.custom = None;
        }
        if ui.button("🚀 Ракета").clicked() {
            self.shape = 17; self.anim = Anim::Float; self.color = egui::Color32::from_rgb(255, 160, 80);
            self.custom = None;
        }
        if ui.button("🐱 Котик").clicked() {
            self.shape = 23; self.anim = Anim::Swing; self.color = egui::Color32::from_rgb(255, 210, 140);
            self.custom = None;
        }
    }

    fn ui_export(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Обои").size(15.0).strong());
        if ui.button("✨ Установить на рабочий стол").clicked() {
            self.apply_wp();
        }
        if ui.button("↩ Вернуть предыдущие").clicked() {
            match wallpaper::restore_backup() {
                Ok(()) => self.status = "↩ Обои возвращены".into(),
                Err(e) => self.status = format!("{e}"),
            }
        }
        ui.separator();
        if ui.button("💾 PNG 1080p").clicked() {
            let p = wallpaper::output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%H%M%S")));
            match self.render_file(&p, 1920, 1080) {
                Ok(()) => self.status = format!("Сохранено {}", p.display()),
                Err(e) => self.status = e,
            }
        }
        if ui.button("💾 PNG 4K").clicked() {
            let p = wallpaper::output_dir().join(format!("poi_4k_{}.png", chrono::Local::now().format("%H%M%S")));
            match self.render_file(&p, 3840, 2160) {
                Ok(()) => self.status = format!("4K {}", p.display()),
                Err(e) => self.status = e,
            }
        }
        ui.separator();
        ui.label("Папка:");
        ui.monospace(wallpaper::output_dir().display().to_string());
    }
}

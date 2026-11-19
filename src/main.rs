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
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("PoiDesktop 2.0"),
        ..Default::default()
    };
    eframe::run_native(
        "PoiDesktop",
        options,
        Box::new(|cc| Box::new(PoiApp::new(cc))),
    )
}

#[derive(Clone, Copy, PartialEq)]
enum Tab { Design, Effects, Widgets, Scenes, Settings }

#[derive(Clone, Copy, PartialEq)]
enum AnimMode { Rotate, Pulse, Orbit, Wave, Bounce, Glitch }

struct WidgetCfg {
    show_clock: bool,
    show_date: bool,
    show_custom: bool,
    custom_text: String,
    clock_size: f32,
    widget_color: egui::Color32,
}

struct PoiApp {
    tab: Tab,
    shape_idx: usize,
    code_variant: usize,
    color: egui::Color32,
    bg_color: egui::Color32,
    accent2: egui::Color32,
    speed: f32,
    scale: f32,
    density: usize,
    glow: bool,
    rainbow: bool,
    static_mode: bool,
    depth_strength: f32,
    particle_amount: f32,
    anim: AnimMode,
    scene: usize,
    time: Instant,
    status: String,
    custom_mesh: Option<ShapeMesh>,
    custom_name: String,
    img_threshold: u8,
    widgets: WidgetCfg,
    fov: f32,
    light_angle: f32,
    trail: bool,
    mirror: bool,
    auto_cycle: bool,
    cycle_timer: f32,
    stars: Vec<[f32; 3]>,
}

impl PoiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(style);

        let mut stars = Vec::new();
        for _ in 0..120 {
            stars.push([
                rand_f() * 2.0 - 0.5,
                rand_f() * 2.0 - 0.5,
                0.2 + rand_f() * 0.8,
            ]);
        }

        Self {
            tab: Tab::Design,
            shape_idx: 15,
            code_variant: 0,
            color: egui::Color32::from_rgb(77, 255, 136),
            bg_color: egui::Color32::from_rgb(6, 8, 16),
            accent2: egui::Color32::from_rgb(60, 140, 255),
            speed: 1.0,
            scale: 1.0,
            density: 220,
            glow: true,
            rainbow: false,
            static_mode: false,
            depth_strength: 1.2,
            particle_amount: 0.5,
            anim: AnimMode::Rotate,
            scene: 0,
            time: Instant::now(),
            status: "PoiDesktop 2.0 — выбери фигуру, настрой, поставь обои".into(),
            custom_mesh: None,
            custom_name: String::new(),
            img_threshold: 90,
            widgets: WidgetCfg {
                show_clock: true,
                show_date: true,
                show_custom: false,
                custom_text: "PoiDesktop".into(),
                clock_size: 28.0,
                widget_color: egui::Color32::WHITE,
            },
            fov: 1.0,
            light_angle: 0.4,
            trail: false,
            mirror: false,
            auto_cycle: false,
            cycle_timer: 0.0,
            stars,
        }
    }

    fn t(&self) -> f32 {
        self.time.elapsed().as_secs_f32() * self.speed
    }

    fn current_mesh(&self) -> ShapeMesh {
        if let Some(ref m) = self.custom_mesh {
            return ShapeMesh { points: m.points.clone() };
        }
        generate_mesh(self.shape_idx, self.code_variant, self.density)
    }

    fn apply_wallpaper(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        match self.render_to_file(&path, 1920, 1080) {
            Ok(()) => match wallpaper::set_wallpaper(&path) {
                Ok(()) => self.status = "✅ Обои установлены".into(),
                Err(e) => self.status = format!("Ошибка установки: {e}"),
            },
            Err(e) => self.status = format!("Ошибка рендера: {e}"),
        }
    }

    fn restore(&mut self) {
        match wallpaper::restore_backup() {
            Ok(()) => self.status = "↩ Старые обои возвращены".into(),
            Err(e) => self.status = format!("Не удалось: {e}"),
        }
    }

    fn load_image_shape(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
            .pick_file()
        {
            match mesh_from_image(&path, self.density * 2, self.img_threshold) {
                Ok(mesh) => {
                    self.custom_name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Фото".into());
                    self.custom_mesh = Some(mesh);
                    self.status = format!("Фото→фигура: {} ({} точек)", self.custom_name, self.custom_mesh.as_ref().map(|m| m.points.len()).unwrap_or(0));
                }
                Err(e) => self.status = format!("Ошибка фото: {e}"),
            }
        }
    }

    fn clear_custom(&mut self) {
        self.custom_mesh = None;
        self.custom_name.clear();
        self.status = "Своя фигура сброшена".into();
    }

    fn project(&self, x: f32, y: f32, z: f32, t: f32) -> (f32, f32, f32) {
        let mut px = x - 0.5;
        let mut py = y - 0.5;
        let mut pz = z;

        // animation
        if !self.static_mode {
            match self.anim {
                AnimMode::Rotate => {
                    let a = t * 0.5;
                    let rx = px * a.cos() - py * a.sin();
                    let ry = px * a.sin() + py * a.cos();
                    px = rx; py = ry;
                }
                AnimMode::Pulse => {
                    let s = 1.0 + 0.08 * (t * 2.5).sin();
                    px *= s; py *= s;
                }
                AnimMode::Orbit => {
                    let a = t * 0.35;
                    let rx = px * a.cos() - pz * a.sin() * 0.5;
                    let rz = px * a.sin() + pz * a.cos();
                    px = rx; pz = rz;
                }
                AnimMode::Wave => {
                    py += 0.04 * (px * 8.0 + t * 3.0).sin();
                }
                AnimMode::Bounce => {
                    py += 0.06 * (t * 2.0).sin().abs();
                }
                AnimMode::Glitch => {
                    if (t * 10.0).sin() > 0.92 {
                        px += 0.03 * (t * 40.0).sin();
                    }
                }
            }
        }

        // simple perspective
        let depth = 1.0 + (pz - 0.5) * self.depth_strength * 0.35 * self.fov;
        px /= depth;
        py /= depth;

        if self.mirror {
            px = -px;
        }

        (px, py, pz)
    }

    fn render_to_file(&self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{ImageBuffer, Rgba};

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
            w, h,
            Rgba([self.bg_color.r(), self.bg_color.g(), self.bg_color.b(), 255]),
        );

        self.draw_scene_bg_img(&mut img, w, h);

        let mesh = self.current_mesh();
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.48;
        let sc = self.scale * (h as f32 * 0.36);
        let t = if self.static_mode { 0.0 } else { self.t() };

        for (i, p) in mesh.points.iter().enumerate() {
            let (px, py, pz) = self.project(p.x, p.y, p.z, t);
            let x = (cx + px * sc * 2.0) as i32;
            let y = (cy + py * sc * 2.0) as i32;

            let (r, g, b) = self.point_color(i, pz, t);
            let size = ((2.0 + pz * 4.0) * (1.0 + self.particle_amount * 0.5)) as i32;
            let alpha = ((0.45 + pz * 0.55) * 255.0) as u8;

            for dy in -size..=size {
                for dx in -size..=size {
                    if dx * dx + dy * dy <= size * size {
                        let xx = x + dx;
                        let yy = y + dy;
                        if xx >= 0 && yy >= 0 && (xx as u32) < w && (yy as u32) < h {
                            let pixel = img.get_pixel_mut(xx as u32, yy as u32);
                            let a = alpha as f32 / 255.0 * 0.85;
                            pixel.0[0] = ((1.0 - a) * pixel.0[0] as f32 + a * r as f32) as u8;
                            pixel.0[1] = ((1.0 - a) * pixel.0[1] as f32 + a * g as f32) as u8;
                            pixel.0[2] = ((1.0 - a) * pixel.0[2] as f32 + a * b as f32) as u8;
                        }
                    }
                }
            }
        }

        img.save(path).map_err(|e| e.to_string())
    }

    fn point_color(&self, i: usize, z: f32, t: f32) -> (u8, u8, u8) {
        let light = (0.55 + 0.45 * (z * self.light_angle + 0.5).sin()).clamp(0.3, 1.0);
        if self.rainbow {
            let hue = (i as f32 * 0.012 + t * 0.2) % 1.0;
            let r = ((hue * 255.0) as u8).wrapping_add(20);
            let g = (((hue + 0.33) % 1.0) * 220.0) as u8;
            let b = (((hue + 0.66) % 1.0) * 255.0) as u8;
            ((r as f32 * light) as u8, (g as f32 * light) as u8, (b as f32 * light) as u8)
        } else {
            let r = (self.color.r() as f32 * light) as u8;
            let g = (self.color.g() as f32 * light) as u8;
            let b = (self.color.b() as f32 * light) as u8;
            // mix accent by depth
            let m = (z * 0.35).clamp(0.0, 1.0);
            (
                ((1.0 - m) * r as f32 + m * self.accent2.r() as f32) as u8,
                ((1.0 - m) * g as f32 + m * self.accent2.g() as f32) as u8,
                ((1.0 - m) * b as f32 + m * self.accent2.b() as f32) as u8,
            )
        }
    }

    fn draw_scene_bg_img(&self, img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, w: u32, h: u32) {
        match self.scene {
            1 => { // stars
                for s in &self.stars {
                    let x = ((s[0] * 0.5 + 0.5) * w as f32) as i32;
                    let y = ((s[1] * 0.5 + 0.5) * h as f32) as i32;
                    let b = (180.0 * s[2]) as u8;
                    if x >= 0 && y >= 0 && (x as u32) < w && (y as u32) < h {
                        let p = img.get_pixel_mut(x as u32, y as u32);
                        p.0 = [b, b, b.saturating_add(20), 255];
                    }
                }
            }
            2 => { // gradient accent
                for y in 0..h {
                    let t = y as f32 / h as f32;
                    let r = ((1.0 - t) * self.bg_color.r() as f32 + t * self.accent2.r() as f32 * 0.3) as u8;
                    let g = ((1.0 - t) * self.bg_color.g() as f32 + t * self.accent2.g() as f32 * 0.3) as u8;
                    let b = ((1.0 - t) * self.bg_color.b() as f32 + t * self.accent2.b() as f32 * 0.3) as u8;
                    for x in 0..w {
                        img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    }
                }
            }
            _ => {}
        }
    }
}

fn rand_f() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut h = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut h);
    (h.finish() % 10000) as f32 / 10000.0
}

impl eframe::App for PoiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.static_mode {
            ctx.request_repaint();
        }

        // auto cycle shapes
        if self.auto_cycle && !self.static_mode {
            self.cycle_timer += ctx.input(|i| i.stable_dt);
            if self.cycle_timer > 8.0 {
                self.cycle_timer = 0.0;
                self.shape_idx = (self.shape_idx + 1) % SHAPE_NAMES.len();
                self.code_variant = 0;
                self.custom_mesh = None;
            }
        }

        // top bar
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("PoiDesktop 2.0");
                ui.separator();
                for (tab, label) in [
                    (Tab::Design, "Дизайн"),
                    (Tab::Effects, "Эффекты"),
                    (Tab::Widgets, "Виджеты"),
                    (Tab::Scenes, "Сцены"),
                    (Tab::Settings, "Настройки"),
                ] {
                    if ui.selectable_label(self.tab == tab, label).clicked() {
                        self.tab = tab;
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✨ Установить обои").clicked() {
                        self.apply_wallpaper();
                    }
                    if ui.button("↩ Вернуть").clicked() {
                        self.restore();
                    }
                });
            });
        });

        // bottom status
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.status).color(egui::Color32::from_rgb(140, 255, 180)));
        });

        // left controls
        egui::SidePanel::left("side").default_width(340.0).resizable(true).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab {
                    Tab::Design => self.ui_design(ui),
                    Tab::Effects => self.ui_effects(ui),
                    Tab::Widgets => self.ui_widgets(ui),
                    Tab::Scenes => self.ui_scenes(ui),
                    Tab::Settings => self.ui_settings(ui),
                }
            });
        });

        // preview
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let (rect, _) = ui.allocate_exact_size(avail, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 0.0, self.bg_color);

            // scene bg stars
            if self.scene == 1 {
                let t = self.t();
                for s in &self.stars {
                    let x = rect.left() + ((s[0] + t * 0.01 * s[2]).rem_euclid(1.2) - 0.1) * rect.width();
                    let y = rect.top() + s[1].rem_euclid(1.0) * rect.height();
                    let b = (100.0 + 155.0 * s[2]) as u8;
                    painter.circle_filled(egui::pos2(x, y), 1.0 + s[2], egui::Color32::from_gray(b));
                }
            }

            let mesh = self.current_mesh();
            let t = if self.static_mode { 0.0 } else { self.t() };
            let cx = rect.center().x;
            let cy = rect.center().y - 10.0;
            let sc = self.scale * rect.height().min(rect.width()) * 0.36;

            for (i, p) in mesh.points.iter().enumerate() {
                let (px, py, pz) = self.project(p.x, p.y, p.z, t);
                let pos = egui::pos2(cx + px * sc * 2.0, cy + py * sc * 2.0);
                let (r, g, b) = self.point_color(i, pz, t);
                let col = egui::Color32::from_rgb(r, g, b);
                let radius = 1.5 + pz * 3.5 * (0.7 + self.particle_amount * 0.5);

                if self.glow {
                    painter.circle_filled(pos, radius * 2.4, egui::Color32::from_rgba_unmultiplied(r, g, b, 45));
                }
                if self.trail && !self.static_mode {
                    let (px2, py2, _) = self.project(p.x, p.y, p.z, t - 0.08);
                    let pos2 = egui::pos2(cx + px2 * sc * 2.0, cy + py2 * sc * 2.0);
                    painter.line_segment([pos2, pos], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(r, g, b, 50)));
                }
                painter.circle_filled(pos, radius, col);
            }

            // widgets overlay
            let wc = self.widgets.widget_color;
            if self.widgets.show_clock {
                let now = chrono::Local::now().format("%H:%M:%S").to_string();
                painter.text(
                    egui::pos2(rect.center().x, rect.top() + 28.0),
                    egui::Align2::CENTER_TOP,
                    now,
                    egui::FontId::monospace(self.widgets.clock_size),
                    wc,
                );
            }
            if self.widgets.show_date {
                let d = chrono::Local::now().format("%d.%m.%Y").to_string();
                painter.text(
                    egui::pos2(rect.center().x, rect.top() + 28.0 + self.widgets.clock_size + 4.0),
                    egui::Align2::CENTER_TOP,
                    d,
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgba_unmultiplied(wc.r(), wc.g(), wc.b(), 180),
                );
            }
            if self.widgets.show_custom {
                painter.text(
                    egui::pos2(rect.center().x, rect.bottom() - 36.0),
                    egui::Align2::CENTER_BOTTOM,
                    &self.widgets.custom_text,
                    egui::FontId::proportional(18.0),
                    wc,
                );
            }

            let title = if self.custom_mesh.is_some() {
                format!("📷 {}", self.custom_name)
            } else {
                format!(
                    "{} · {}",
                    SHAPE_NAMES.get(self.shape_idx).unwrap_or(&"?"),
                    code_variants(self.shape_idx).get(self.code_variant).unwrap_or(&"")
                )
            };
            painter.text(
                rect.left_top() + egui::vec2(14.0, 12.0),
                egui::Align2::LEFT_TOP,
                title,
                egui::FontId::proportional(15.0),
                egui::Color32::from_white_alpha(160),
            );
        });
    }
}

impl PoiApp {
    fn ui_design(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Фигура").strong());
        if self.custom_mesh.is_some() {
            ui.label(format!("Своя: {}", self.custom_name));
            if ui.button("Сбросить фото-фигуру").clicked() {
                self.clear_custom();
            }
            ui.separator();
        }

        egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (i, name) in SHAPE_NAMES.iter().enumerate() {
                    if ui.selectable_label(self.shape_idx == i && self.custom_mesh.is_none(), *name).clicked() {
                        self.shape_idx = i;
                        self.code_variant = 0;
                        self.custom_mesh = None;
                        self.status = format!("Фигура: {name}");
                    }
                }
            });
        });

        ui.add_space(6.0);
        ui.label("Код-фигуры (меняются с фигурой)");
        let variants = code_variants(self.shape_idx);
        ui.horizontal_wrapped(|ui| {
            for (i, v) in variants.iter().enumerate() {
                if ui.selectable_label(self.code_variant == i, *v).clicked() {
                    self.code_variant = i;
                }
            }
        });

        ui.separator();
        ui.label(egui::RichText::new("Фото → фигура").strong());
        ui.add(egui::Slider::new(&mut self.img_threshold, 20..=200).text("Порог"));
        if ui.button("📂 Загрузить картинку").clicked() {
            self.load_image_shape();
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Цвет");
            ui.color_edit_button_srgba(&mut self.color);
            ui.label("Акцент");
            ui.color_edit_button_srgba(&mut self.accent2);
        });
        ui.horizontal(|ui| {
            ui.label("Фон");
            ui.color_edit_button_srgba(&mut self.bg_color);
        });
        ui.checkbox(&mut self.rainbow, "Радуга");
        ui.add(egui::Slider::new(&mut self.scale, 0.4..=2.2).text("Масштаб"));
        ui.add(egui::Slider::new(&mut self.density, 80..=500).text("Плотность"));
    }

    fn ui_effects(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Анимация").strong());
        ui.horizontal_wrapped(|ui| {
            for (m, name) in [
                (AnimMode::Rotate, "Вращение"),
                (AnimMode::Pulse, "Пульс"),
                (AnimMode::Orbit, "Орбита 3D"),
                (AnimMode::Wave, "Волна"),
                (AnimMode::Bounce, "Прыжок"),
                (AnimMode::Glitch, "Глитч"),
            ] {
                if ui.selectable_label(self.anim == m, name).clicked() {
                    self.anim = m;
                }
            }
        });
        ui.checkbox(&mut self.static_mode, "Статичный режим");
        ui.add(egui::Slider::new(&mut self.speed, 0.15..=3.0).text("Скорость"));

        ui.separator();
        ui.label(egui::RichText::new("3D и свет").strong());
        ui.add(egui::Slider::new(&mut self.depth_strength, 0.0..=2.5).text("Глубина 3D"));
        ui.add(egui::Slider::new(&mut self.fov, 0.6..=1.6).text("Перспектива"));
        ui.add(egui::Slider::new(&mut self.light_angle, 0.0..=1.5).text("Свет"));
        ui.add(egui::Slider::new(&mut self.particle_amount, 0.0..=1.5).text("Размер точек"));

        ui.separator();
        ui.checkbox(&mut self.glow, "Свечение");
        ui.checkbox(&mut self.trail, "Шлейф");
        ui.checkbox(&mut self.mirror, "Зеркало");
        ui.checkbox(&mut self.auto_cycle, "Автосмена фигур");
    }

    fn ui_widgets(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Виджеты на обоях").strong());
        ui.checkbox(&mut self.widgets.show_clock, "Часы");
        ui.checkbox(&mut self.widgets.show_date, "Дата");
        ui.checkbox(&mut self.widgets.show_custom, "Свой текст");
        if self.widgets.show_custom {
            ui.text_edit_singleline(&mut self.widgets.custom_text);
        }
        ui.add(egui::Slider::new(&mut self.widgets.clock_size, 16.0..=64.0).text("Размер часов"));
        ui.horizontal(|ui| {
            ui.label("Цвет виджетов");
            ui.color_edit_button_srgba(&mut self.widgets.widget_color);
        });
        ui.label("Виджеты рисуются и в превью, и на установленных обоях (в момент снимка).");
    }

    fn ui_scenes(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Сцена / фон").strong());
        for (i, name) in ["Пустой", "Звёзды", "Градиент"].iter().enumerate() {
            if ui.selectable_label(self.scene == i, *name).clicked() {
                self.scene = i;
            }
        }
        ui.separator();
        ui.label("Быстрые пресеты");
        if ui.button("🌌 Космос (планета + звёзды + орбита)").clicked() {
            self.shape_idx = 15; self.code_variant = 3; self.scene = 1;
            self.anim = AnimMode::Orbit; self.color = egui::Color32::from_rgb(100, 200, 255);
            self.glow = true; self.custom_mesh = None;
        }
        if ui.button("🚀 Ракета + глитч").clicked() {
            self.shape_idx = 17; self.code_variant = 0; self.anim = AnimMode::Glitch;
            self.color = egui::Color32::from_rgb(255, 120, 60); self.scene = 0; self.custom_mesh = None;
        }
        if ui.button("💚 Неоновое сердце").clicked() {
            self.shape_idx = 22; self.code_variant = 1; self.anim = AnimMode::Pulse;
            self.color = egui::Color32::from_rgb(255, 60, 120); self.rainbow = false; self.custom_mesh = None;
        }
        if ui.button("🐱 Котик").clicked() {
            self.shape_idx = 23; self.anim = AnimMode::Bounce; self.color = egui::Color32::from_rgb(255, 200, 120);
            self.custom_mesh = None;
        }
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Обои").strong());
        if ui.button("✨ Установить как обои").clicked() {
            self.apply_wallpaper();
        }
        if ui.button("↩ Вернуть старые обои").clicked() {
            self.restore();
        }
        if ui.button("💾 Сохранить PNG").clicked() {
            let path = wallpaper::output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S")));
            match self.render_to_file(&path, 1920, 1080) {
                Ok(()) => self.status = format!("Сохранено: {}", path.display()),
                Err(e) => self.status = format!("Ошибка: {e}"),
            }
        }
        if ui.button("💾 PNG 4K").clicked() {
            let path = wallpaper::output_dir().join(format!("poi_4k_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S")));
            match self.render_to_file(&path, 3840, 2160) {
                Ok(()) => self.status = format!("4K сохранено: {}", path.display()),
                Err(e) => self.status = format!("Ошибка: {e}"),
            }
        }
        ui.separator();
        ui.label("Файлы обоев:");
        ui.monospace(wallpaper::output_dir().display().to_string());
        ui.separator();
        ui.label("PoiDesktop 2.0 — Rust + egui");
        ui.label("Работает без активации Windows");
    }
}

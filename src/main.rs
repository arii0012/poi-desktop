#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shapes;
mod wallpaper;

use eframe::egui;
use shapes::{code_variants, generate_mesh, SHAPE_NAMES};
use std::path::PathBuf;
use std::time::Instant;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([800.0, 560.0])
            .with_title("PoiDesktop — Живые фигуры и обои"),
        ..Default::default()
    };
    eframe::run_native(
        "PoiDesktop",
        options,
        Box::new(|cc| Ok(Box::new(PoiApp::new(cc)))),
    )
}

struct PoiApp {
    shape_idx: usize,
    code_variant: usize,
    color: egui::Color32,
    bg_color: egui::Color32,
    speed: f32,
    scale: f32,
    density: usize,
    glow: bool,
    rainbow: bool,
    static_mode: bool,
    time: Instant,
    last_set_path: Option<PathBuf>,
    status: String,
    // анимация
    particles: Vec<Particle>,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

impl PoiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // тёмная тема по умолчанию
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        cc.egui_ctx.set_style(style);

        Self {
            shape_idx: 15, // планета по умолчанию
            code_variant: 0,
            color: egui::Color32::from_rgb(77, 255, 136),
            bg_color: egui::Color32::from_rgb(8, 10, 18),
            speed: 1.0,
            scale: 1.0,
            density: 180,
            glow: true,
            rainbow: false,
            static_mode: false,
            time: Instant::now(),
            last_set_path: None,
            status: "Готов. Выбери фигуру → код-фигуры меняются автоматически.".into(),
            particles: Vec::new(),
        }
    }

    fn t(&self) -> f32 {
        self.time.elapsed().as_secs_f32() * self.speed
    }

    fn apply_wallpaper(&mut self) {
        let path = wallpaper::output_dir().join("poi_current.png");
        match self.render_to_file(&path, 1920, 1080) {
            Ok(()) => {
                match wallpaper::set_wallpaper(&path) {
                    Ok(()) => {
                        self.last_set_path = Some(path);
                        self.status = "✅ Обои установлены (даже на неактивированной Windows)".into();
                    }
                    Err(e) => self.status = format!("Ошибка установки: {e}"),
                }
            }
            Err(e) => self.status = format!("Ошибка рендера: {e}"),
        }
    }

    fn restore(&mut self) {
        match wallpaper::restore_backup() {
            Ok(()) => self.status = "↩ Старые обои возвращены".into(),
            Err(e) => self.status = format!("Не удалось вернуть: {e}"),
        }
    }

    fn render_to_file(&self, path: &PathBuf, w: u32, h: u32) -> Result<(), String> {
        use image::{RgbImage, ImageBuffer, Rgba};

        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_pixel(
            w, h,
            Rgba([self.bg_color.r(), self.bg_color.g(), self.bg_color.b(), 255]),
        );

        let mesh = generate_mesh(self.shape_idx, self.code_variant, self.density);
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.48;
        let sc = self.scale * (h as f32 * 0.38);

        let t = if self.static_mode { 0.0 } else { self.t() };

        for (i, p) in mesh.points.iter().enumerate() {
            let d = mesh.depth.get(i).copied().unwrap_or(0.5);
            // лёгкое вращение / пульс
            let angle = t * 0.4;
            let px = p[0] - 0.5;
            let py = p[1] - 0.5;
            let rx = px * angle.cos() - py * angle.sin();
            let ry = px * angle.sin() + py * angle.cos();

            let x = (cx + rx * sc * 2.0) as i32;
            let y = (cy + ry * sc * 2.0) as i32;

            let mut col = self.color;
            if self.rainbow {
                let hue = (i as f32 * 0.02 + t * 0.3) % 1.0;
                col = egui::Color32::from_rgb(
                    ((hue * 255.0) as u8).wrapping_add(40),
                    (((hue + 0.33) % 1.0) * 200.0) as u8,
                    (((hue + 0.66) % 1.0) * 255.0) as u8,
                );
            }

            let alpha = (d * 255.0) as u8;
            let size = (2.0 + d * 3.5) as i32;

            for dy in -size..=size {
                for dx in -size..=size {
                    if dx * dx + dy * dy <= size * size {
                        let xx = x + dx;
                        let yy = y + dy;
                        if xx >= 0 && yy >= 0 && (xx as u32) < w && (yy as u32) < h {
                            let pixel = img.get_pixel_mut(xx as u32, yy as u32);
                            // простой glow / blend
                            let a = alpha as f32 / 255.0;
                            pixel.0[0] = ((1.0 - a) * pixel.0[0] as f32 + a * col.r() as f32) as u8;
                            pixel.0[1] = ((1.0 - a) * pixel.0[1] as f32 + a * col.g() as f32) as u8;
                            pixel.0[2] = ((1.0 - a) * pixel.0[2] as f32 + a * col.b() as f32) as u8;
                        }
                    }
                }
            }
        }

        img.save(path).map_err(|e| e.to_string())
    }
}

impl eframe::App for PoiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // анимация
        if !self.static_mode {
            ctx.request_repaint();
        }

        // ===== Левая панель — настройки =====
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("PoiDesktop");
                ui.label(egui::RichText::new("Фигуры • Код-варианты • Обои").weak());
                ui.separator();

                // Фигура
                ui.label("Фигура");
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for (i, name) in SHAPE_NAMES.iter().enumerate() {
                                let selected = self.shape_idx == i;
                                if ui.selectable_label(selected, *name).clicked() {
                                    self.shape_idx = i;
                                    self.code_variant = 0; // сброс варианта при смене фигуры
                                    self.status = format!("Выбрана: {} → код-фигуры обновлены", name);
                                }
                            }
                        });
                    });

                ui.add_space(6.0);
                ui.separator();

                // Код-фигуры — МЕНЯЮТСЯ в зависимости от выбранной фигуры
                let variants = code_variants(self.shape_idx);
                ui.label(format!("Код-фигуры для «{}»", SHAPE_NAMES[self.shape_idx]));
                ui.horizontal_wrapped(|ui| {
                    for (i, v) in variants.iter().enumerate() {
                        let selected = self.code_variant == i;
                        if ui.selectable_label(selected, *v).clicked() {
                            self.code_variant = i;
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                // Цвет
                ui.horizontal(|ui| {
                    ui.label("Цвет фигуры");
                    ui.color_edit_button_srgba(&mut self.color);
                });
                ui.horizontal(|ui| {
                    ui.label("Фон");
                    ui.color_edit_button_srgba(&mut self.bg_color);
                });
                ui.checkbox(&mut self.rainbow, "Радуга");
                ui.checkbox(&mut self.glow, "Свечение");
                ui.checkbox(&mut self.static_mode, "Статичный режим (без анимации)");

                ui.add_space(4.0);
                ui.add(egui::Slider::new(&mut self.speed, 0.2..=3.0).text("Скорость"));
                ui.add(egui::Slider::new(&mut self.scale, 0.4..=2.0).text("Масштаб"));
                ui.add(egui::Slider::new(&mut self.density, 60..=400).text("Плотность"));

                ui.add_space(12.0);
                ui.separator();

                // Кнопки действий
                if ui.add(egui::Button::new("✨ Установить обои").min_size([0.0, 36.0])).clicked() {
                    self.apply_wallpaper();
                }
                if ui.button("↩ Вернуть старые обои").clicked() {
                    self.restore();
                }
                if ui.button("💾 Сохранить превью в файл").clicked() {
                    let path = wallpaper::output_dir().join(format!("poi_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S")));
                    match self.render_to_file(&path, 1920, 1080) {
                        Ok(()) => self.status = format!("Сохранено: {}", path.display()),
                        Err(e) => self.status = format!("Ошибка: {e}"),
                    }
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.status).small().color(egui::Color32::LIGHT_GREEN));
            });

        // ===== Центральная область — живой превью =====
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let (rect, _resp) = ui.allocate_exact_size(avail, egui::Sense::hover());

            let painter = ui.painter_at(rect);
            // фон
            painter.rect_filled(rect, 0.0, self.bg_color);

            let mesh = generate_mesh(self.shape_idx, self.code_variant, self.density);
            let t = if self.static_mode { 0.0 } else { self.t() };
            let cx = rect.center().x;
            let cy = rect.center().y - 20.0;
            let sc = self.scale * rect.height().min(rect.width()) * 0.38;

            for (i, p) in mesh.points.iter().enumerate() {
                let d = mesh.depth.get(i).copied().unwrap_or(0.5);
                let angle = t * 0.45;
                let px = p[0] - 0.5;
                let py = p[1] - 0.5;
                let rx = px * angle.cos() - py * angle.sin();
                let ry = px * angle.sin() + py * angle.cos();

                let pos = egui::pos2(cx + rx * sc * 2.0, cy + ry * sc * 2.0);

                let mut col = self.color;
                if self.rainbow {
                    let hue = (i as f32 * 0.015 + t * 0.25) % 1.0;
                    col = egui::Color32::from_rgb(
                        ((hue * 255.0) as u8).wrapping_add(30),
                        (((hue + 0.33) % 1.0) * 220.0) as u8,
                        (((hue + 0.66) % 1.0) * 255.0) as u8,
                    );
                }

                let radius = 1.8 + d * 3.2;
                if self.glow {
                    painter.circle_filled(pos, radius * 2.2, col.linear_multiply(0.25));
                }
                painter.circle_filled(pos, radius, col);
            }

            // подпись
            painter.text(
                rect.left_top() + egui::vec2(12.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("{}  ·  {}", SHAPE_NAMES[self.shape_idx], code_variants(self.shape_idx).get(self.code_variant).unwrap_or(&"")),
                egui::FontId::proportional(16.0),
                egui::Color32::from_white_alpha(180),
            );
        });
    }
}

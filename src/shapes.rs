//! Фигуры с depth + контекстные код-варианты + загрузка из картинки.

use std::f32::consts::PI;
use std::path::Path;

pub const SHAPE_NAMES: &[&str] = &[
    "Капюшон", "Шар", "Сердце", "Звезда", "Череп", "Корона", "Молния", "Луна", "Алмаз",
    "Кошка", "Призрак", "Глаз", "Огонь", "Капля", "Облако", "Планета", "Курсор",
    "Ракета", "CS2", "Писюн", "Сиська", "НЛО", "Сердце 3D", "Котик", "Череп 2",
    "Молния 2", "Планета 2", "Ракета 2", "Глаз 2", "Огонь 2", "Звезда 2", "Алмаз 2",
];

pub fn code_variants(shape: usize) -> &'static [&'static str] {
    match shape {
        0  => &["Классика", "Неон", "Контур", "Глитч", "Мини"],
        1  => &["Шар", "Сфера", "Пузырь", "Орбита", "Ядро"],
        2  => &["Сердце", "Любовь", "Бит", "Двойное", "Пиксель"],
        3  => &["Звезда", "5-луч", "8-луч", "Комета", "Искра"],
        4  => &["Череп", "Пират", "Готика", "Неон-череп", "Маска"],
        5  => &["Корона", "Король", "Королева", "Тиара", "Шипы"],
        6  => &["Молния", "Гром", "Энергия", "Разряд", "Болт"],
        7  => &["Луна", "Полумесяц", "Полная", "Затмение", "Серп"],
        8  => &["Алмаз", "Кристалл", "Рубин", "Огранка", "Грань"],
        9  => &["Кошка", "Морда", "Силуэт", "Ушки", "Мяу"],
        10 => &["Призрак", "Бу", "Прозрачный", "Страх", "Тень"],
        11 => &["Глаз", "Зрачок", "Взгляд", "Кибер", "Око"],
        12 => &["Огонь", "Пламя", "Искра", "Вулкан", "Жар"],
        13 => &["Капля", "Вода", "Кровь", "Масло", "Слеза"],
        14 => &["Облако", "Тучка", "Дым", "Пар", "Туман"],
        15 => &["Планета", "Земля", "Марс", "Сатурн", "Юпитер", "Чёрная дыра"],
        16 => &["Курсор", "Стрелка", "Клик", "Указатель", "Мышь"],
        17 => &["Ракета", "Старт", "Космос", "Falcon", "Восток"],
        18 => &["CS2", "Counter", "AWP", "AK", "Terrorist"],
        19 => &["Писюн", "Классика", "Толстый", "Длинный", "Неон", "Пиксель"],
        20 => &["Сиська", "Классика", "Пара", "Неон", "Мягкая", "Пиксель"],
        21 => &["НЛО", "Тарелка", "Пришелец", "Луч", "Абдукция"],
        22 => &["Сердце 3D", "Объём", "Глубина", "Бит", "Пульс"],
        23 => &["Котик", "Мордочка", "Спящий", "Игривый", "Хвост"],
        24 => &["Череп 2", "Готика", "Пират", "Неон", "Маска"],
        25 => &["Молния 2", "Разряд", "Энергия", "Болт", "Гром"],
        26 => &["Планета 2", "Земля", "Марс", "Сатурн", "Газ"],
        27 => &["Ракета 2", "Старт", "Космос", "Boost", "Орбита"],
        28 => &["Глаз 2", "Кибер", "Зрачок", "Взгляд", "Око"],
        29 => &["Огонь 2", "Пламя", "Вулкан", "Жар", "Искра"],
        30 => &["Звезда 2", "5-луч", "Комета", "Искра", "Нова"],
        31 => &["Алмаз 2", "Кристалл", "Огранка", "Рубин", "Грань"],
        _  => &["Классика", "Неон", "Контур", "Глитч", "Мини"],
    }
}

#[derive(Clone)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32, // depth 0..1
}

#[derive(Clone)]
pub struct ShapeMesh {
    pub points: Vec<Point3>,
}

fn push_circle(out: &mut Vec<Point3>, cx: f32, cy: f32, r: f32, n: usize, z: f32) {
    for i in 0..n {
        let a = i as f32 / n as f32 * PI * 2.0;
        out.push(Point3 { x: cx + r * a.cos(), y: cy + r * a.sin(), z });
    }
}

fn push_ring(out: &mut Vec<Point3>, cx: f32, cy: f32, r: f32, n: usize, z: f32, aspect: f32) {
    for i in 0..n {
        let a = i as f32 / n as f32 * PI * 2.0;
        out.push(Point3 { x: cx + r * a.cos(), y: cy + r * aspect * a.sin(), z });
    }
}

pub fn generate_mesh(shape: usize, variant: usize, density: usize) -> ShapeMesh {
    let mut points = Vec::with_capacity(density * 3);
    let n = density.max(100);
    let ox = ((variant % 3) as f32 - 1.0) * 0.02;
    let oy = (((variant / 3) % 3) as f32 - 1.0) * 0.015;

    match shape {
        1 => { // Шар 3D
            for layer in 0..8 {
                let t = layer as f32 / 7.0;
                let rr = (1.0 - (t - 0.5).powi(2) * 4.0).max(0.0).sqrt() * 0.40;
                let yy = (t - 0.5) * 0.80;
                let z = 0.3 + t * 0.6;
                push_circle(&mut points, 0.5 + ox, 0.5 + yy + oy, rr, n / 6, z);
            }
        }
        2 | 22 => { // Сердце
            for i in 0..n {
                let t = i as f32 / n as f32 * PI * 2.0;
                let x = 16.0 * t.sin().powi(3);
                let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
                let z = 0.5 + 0.35 * ((x * 0.05).cos());
                points.push(Point3 { x: 0.5 + x * 0.026 + ox, y: 0.48 + y * 0.026 + oy, z });
            }
            if shape == 22 || variant > 1 {
                for i in 0..(n / 2) {
                    let t = i as f32 / (n / 2) as f32 * PI * 2.0;
                    let x = 16.0 * t.sin().powi(3) * 0.7;
                    let y = -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos()) * 0.7;
                    points.push(Point3 { x: 0.5 + x * 0.026 + ox, y: 0.48 + y * 0.026 + oy, z: 0.85 });
                }
            }
        }
        3 | 30 => {
            let rays = if variant % 2 == 0 { 5 } else { 8 };
            for ring in 0..4 {
                let scale = 1.0 - ring as f32 * 0.18;
                for i in 0..(rays * 2) {
                    let a = -PI / 2.0 + i as f32 * PI / rays as f32;
                    let r = if i % 2 == 0 { 0.45 * scale } else { 0.18 * scale };
                    let z = 0.4 + ring as f32 * 0.15;
                    points.push(Point3 { x: 0.5 + r * a.cos() + ox, y: 0.5 + r * a.sin() + oy, z });
                }
            }
        }
        4 | 24 => { // Череп
            push_circle(&mut points, 0.5 + ox, 0.42 + oy, 0.28, n / 3, 0.8);
            for i in 0..20 {
                let t = i as f32 / 19.0;
                points.push(Point3 { x: 0.35 + ox, y: 0.55 + t * 0.28 + oy, z: 0.6 });
                points.push(Point3 { x: 0.65 + ox, y: 0.55 + t * 0.28 + oy, z: 0.6 });
            }
            push_circle(&mut points, 0.40 + ox, 0.42 + oy, 0.07, 12, 0.3);
            push_circle(&mut points, 0.60 + ox, 0.42 + oy, 0.07, 12, 0.3);
            for i in 0..5 {
                let x = 0.38 + i as f32 * 0.06;
                points.push(Point3 { x: x + ox, y: 0.72 + oy, z: 0.4 });
                points.push(Point3 { x: x + ox, y: 0.80 + oy, z: 0.4 });
            }
        }
        15 | 26 => { // Планета 3D
            for layer in 0..10 {
                let t = layer as f32 / 9.0;
                let rr = (1.0 - (t - 0.5).powi(2) * 4.0).max(0.0).sqrt() * 0.30;
                let yy = (t - 0.5) * 0.60;
                push_circle(&mut points, 0.5 + ox, 0.5 + yy + oy, rr, n / 8, 0.3 + t * 0.5);
            }
            let rings = if variant >= 3 { 3 } else if variant >= 1 { 1 } else { 0 };
            for r in 0..rings {
                let rr = 0.38 + r as f32 * 0.07;
                push_ring(&mut points, 0.5 + ox, 0.5 + oy, rr, n / 2, 0.45 - r as f32 * 0.05, 0.28);
            }
            if variant == 5 {
                for i in 0..n {
                    let t = i as f32 / n as f32;
                    let a = t * PI * 8.0;
                    let r = 0.08 + t * 0.38;
                    points.push(Point3 { x: 0.5 + r * a.cos() + ox, y: 0.5 + r * a.sin() + oy, z: 0.2 + t * 0.7 });
                }
            }
        }
        17 | 27 => {
            let body = [[0.5, 0.10], [0.60, 0.28], [0.60, 0.62], [0.5, 0.88], [0.40, 0.62], [0.40, 0.28]];
            for p in &body {
                points.push(Point3 { x: p[0] + ox, y: p[1] + oy, z: 0.85 });
            }
            for i in 0..15 {
                let t = i as f32 / 14.0;
                points.push(Point3 { x: 0.42 + ox, y: 0.25 + t * 0.45 + oy, z: 0.7 });
                points.push(Point3 { x: 0.58 + ox, y: 0.25 + t * 0.45 + oy, z: 0.7 });
            }
            push_circle(&mut points, 0.5 + ox, 0.28 + oy, 0.06, 10, 0.4);
            points.push(Point3 { x: 0.40 + ox, y: 0.60 + oy, z: 0.5 });
            points.push(Point3 { x: 0.28 + ox, y: 0.78 + oy, z: 0.5 });
            points.push(Point3 { x: 0.40 + ox, y: 0.70 + oy, z: 0.5 });
            points.push(Point3 { x: 0.60 + ox, y: 0.60 + oy, z: 0.5 });
            points.push(Point3 { x: 0.72 + ox, y: 0.78 + oy, z: 0.5 });
            points.push(Point3 { x: 0.60 + ox, y: 0.70 + oy, z: 0.5 });
            for i in 0..16 {
                let t = i as f32 / 15.0;
                points.push(Point3 { x: 0.5 + (t - 0.5) * 0.18 + ox, y: 0.88 + t * 0.12 + oy, z: 0.3 + (1.0 - t) * 0.3 });
            }
        }
        19 => {
            for i in 0..25 {
                let t = i as f32 / 24.0;
                let w = 0.07 + (t * PI).sin().abs() * 0.02;
                points.push(Point3 { x: 0.5 - w + ox, y: 0.12 + t * 0.55 + oy, z: 0.8 });
                points.push(Point3 { x: 0.5 + w + ox, y: 0.12 + t * 0.55 + oy, z: 0.8 });
            }
            push_circle(&mut points, 0.5 + ox, 0.70 + oy, 0.11, 16, 0.95);
            push_circle(&mut points, 0.40 + ox, 0.82 + oy, 0.09, 14, 0.7);
            push_circle(&mut points, 0.60 + ox, 0.82 + oy, 0.09, 14, 0.7);
        }
        20 => {
            for side in [-1.0f32, 1.0] {
                for layer in 0..5 {
                    let t = layer as f32 / 4.0;
                    let rr = (1.0 - (t - 0.5).powi(2) * 3.5).max(0.0).sqrt() * 0.15;
                    let yy = (t - 0.5) * 0.28;
                    push_circle(&mut points, 0.5 + side * 0.17 + ox, 0.50 + yy + oy, rr, 14, 0.5 + t * 0.4);
                }
                push_circle(&mut points, 0.5 + side * 0.17 + ox, 0.50 + oy, 0.035, 8, 0.25);
            }
        }
        21 => {
            push_ring(&mut points, 0.5 + ox, 0.48 + oy, 0.36, n / 2, 0.85, 0.32);
            push_ring(&mut points, 0.5 + ox, 0.42 + oy, 0.20, n / 3, 0.7, 0.55);
            push_circle(&mut points, 0.5 + ox, 0.40 + oy, 0.07, 10, 0.4);
            for lx in [-0.18f32, 0.0, 0.18] {
                for i in 0..8 {
                    let t = i as f32 / 7.0;
                    points.push(Point3 { x: 0.5 + lx * (1.0 + t * 0.3) + ox, y: 0.55 + t * 0.35 + oy, z: 0.4 - t * 0.2 });
                }
            }
        }
        23 => {
            push_circle(&mut points, 0.5 + ox, 0.55 + oy, 0.30, n / 3, 0.8);
            points.push(Point3 { x: 0.32 + ox, y: 0.38 + oy, z: 0.7 });
            points.push(Point3 { x: 0.28 + ox, y: 0.15 + oy, z: 0.7 });
            points.push(Point3 { x: 0.42 + ox, y: 0.38 + oy, z: 0.7 });
            points.push(Point3 { x: 0.68 + ox, y: 0.38 + oy, z: 0.7 });
            points.push(Point3 { x: 0.72 + ox, y: 0.15 + oy, z: 0.7 });
            points.push(Point3 { x: 0.58 + ox, y: 0.38 + oy, z: 0.7 });
            push_circle(&mut points, 0.40 + ox, 0.52 + oy, 0.05, 8, 0.3);
            push_circle(&mut points, 0.60 + ox, 0.52 + oy, 0.05, 8, 0.3);
            points.push(Point3 { x: 0.48 + ox, y: 0.62 + oy, z: 0.5 });
            points.push(Point3 { x: 0.52 + ox, y: 0.62 + oy, z: 0.5 });
            points.push(Point3 { x: 0.50 + ox, y: 0.66 + oy, z: 0.5 });
        }
        12 | 29 => {
            for i in 0..n {
                let t = i as f32 / n as f32;
                let a = t * PI * 2.0;
                let wobble = 0.08 * (a * 3.0).sin();
                let r = 0.12 + t * 0.28 + wobble;
                let y = 0.85 - t * 0.70;
                points.push(Point3 { x: 0.5 + r * (a * 0.7).sin() + ox, y: y + oy, z: 0.3 + t * 0.5 });
                points.push(Point3 { x: 0.5 - r * 0.6 * (a * 0.5).cos() + ox, y: y + 0.02 + oy, z: 0.25 + t * 0.4 });
            }
        }
        7 => {
            push_circle(&mut points, 0.5 + ox, 0.5 + oy, 0.36, n / 2, 0.75);
            if variant != 2 {
                push_circle(&mut points, 0.62 + ox, 0.42 + oy, 0.28, n / 3, 0.2);
            }
        }
        _ => {
            for layer in 0..6 {
                let t = layer as f32 / 5.0;
                let rr = (1.0 - (t - 0.5).powi(2) * 3.0).max(0.15) * 0.36;
                let yy = (t - 0.5) * 0.55;
                push_circle(&mut points, 0.5 + ox, 0.52 + yy + oy, rr, n / 7, 0.35 + t * 0.5);
            }
        }
    }

    ShapeMesh { points }
}

/// Загрузка фигуры из картинки (фото → маска точек)
pub fn mesh_from_image(path: &Path, density: usize, threshold: u8) -> Result<ShapeMesh, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_luma8();
    let (w, h) = img.dimensions();
    if w < 4 || h < 4 {
        return Err("Картинка слишком маленькая".into());
    }

    let mut points = Vec::new();
    let step_x = (w as f32 / (density as f32).sqrt()).max(1.0) as u32;
    let step_y = (h as f32 / (density as f32).sqrt()).max(1.0) as u32;

    // bounding box of bright pixels
    let mut min_x = w;
    let mut max_x = 0u32;
    let mut min_y = h;
    let mut max_y = 0u32;
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y).0[0] > threshold {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x >= max_x || min_y >= max_y {
        return Err("Не удалось выделить фигуру (попробуй другой порог)".into());
    }

    let bw = (max_x - min_x) as f32;
    let bh = (max_y - min_y) as f32;

    for y in (min_y..=max_y).step_by(step_y as usize) {
        for x in (min_x..=max_x).step_by(step_x as usize) {
            let v = img.get_pixel(x, y).0[0];
            if v > threshold {
                let nx = (x - min_x) as f32 / bw;
                let ny = (y - min_y) as f32 / bh;
                // center and fit
                let px = 0.5 + (nx - 0.5) * 0.85;
                let py = 0.5 + (ny - 0.5) * 0.85;
                let z = (v as f32 / 255.0) * 0.7 + 0.2;
                points.push(Point3 { x: px, y: py, z });
            }
        }
    }

    if points.len() < 20 {
        return Err("Слишком мало точек — снизь порог или выбери другое фото".into());
    }
    Ok(ShapeMesh { points })
}

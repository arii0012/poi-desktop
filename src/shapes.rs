//! Объёмные фигуры: контуры + fill-диски с depth.

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
pub struct Blob {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32, // radius in normalized space
}

#[derive(Clone)]
pub struct ShapeMesh {
    pub blobs: Vec<Blob>,
}

fn add_sphere(out: &mut Vec<Blob>, cx: f32, cy: f32, r: f32, layers: usize) {
    for i in 0..layers {
        let t = i as f32 / layers as f32;
        let rr = r * (1.0 - t * 0.85);
        let z = 0.25 + t * 0.7;
        out.push(Blob { x: cx, y: cy, z, r: rr });
    }
}

fn add_circle_outline(out: &mut Vec<Blob>, cx: f32, cy: f32, r: f32, n: usize, z: f32, br: f32) {
    for i in 0..n {
        let a = i as f32 / n as f32 * PI * 2.0;
        out.push(Blob {
            x: cx + r * a.cos(),
            y: cy + r * a.sin(),
            z,
            r: br,
        });
    }
}

pub fn generate_mesh(shape: usize, variant: usize, quality: usize) -> ShapeMesh {
    let mut blobs = Vec::with_capacity(64);
    let q = quality.max(1).min(3);
    let ox = ((variant % 3) as f32 - 1.0) * 0.015;
    let oy = (((variant / 3) % 3) as f32 - 1.0) * 0.01;

    match shape {
        1 => { // Шар — настоящий объём
            add_sphere(&mut blobs, 0.5 + ox, 0.5 + oy, 0.38, 6 + q * 2);
        }
        2 | 22 => { // Сердце
            add_sphere(&mut blobs, 0.38 + ox, 0.40 + oy, 0.20, 5);
            add_sphere(&mut blobs, 0.62 + ox, 0.40 + oy, 0.20, 5);
            // bottom tip layers
            for i in 0..6 {
                let t = i as f32 / 5.0;
                let y = 0.48 + t * 0.32;
                let r = 0.22 * (1.0 - t * 0.85);
                add_sphere(&mut blobs, 0.5 + ox, y + oy, r, 3);
            }
            if shape == 22 {
                add_sphere(&mut blobs, 0.5 + ox, 0.45 + oy, 0.12, 4);
            }
        }
        3 | 30 => {
            let rays = if variant % 2 == 0 { 5 } else { 8 };
            add_sphere(&mut blobs, 0.5 + ox, 0.5 + oy, 0.12, 4);
            for i in 0..rays {
                let a = -PI / 2.0 + i as f32 * PI * 2.0 / rays as f32;
                for k in 1..5 {
                    let t = k as f32 / 4.0;
                    let r = 0.12 + t * 0.32;
                    blobs.push(Blob {
                        x: 0.5 + r * a.cos() + ox,
                        y: 0.5 + r * a.sin() + oy,
                        z: 0.5 + t * 0.3,
                        r: 0.06 * (1.2 - t * 0.5),
                    });
                }
            }
        }
        15 | 26 => { // Планета
            add_sphere(&mut blobs, 0.5 + ox, 0.5 + oy, 0.28, 7 + q);
            let rings = if variant >= 3 { 2 } else if variant >= 1 { 1 } else { 0 };
            for ri in 0..rings {
                let rr = 0.40 + ri as f32 * 0.08;
                add_circle_outline(&mut blobs, 0.5 + ox, 0.5 + oy, rr, 28 + q * 8, 0.55, 0.025);
            }
            if variant == 5 {
                for i in 0..20 {
                    let t = i as f32 / 19.0;
                    let a = t * PI * 5.0;
                    let r = 0.1 + t * 0.35;
                    blobs.push(Blob {
                        x: 0.5 + r * a.cos() + ox,
                        y: 0.5 + r * a.sin() + oy,
                        z: 0.3 + t * 0.5,
                        r: 0.03,
                    });
                }
            }
        }
        17 | 27 => { // Ракета
            for i in 0..8 {
                let t = i as f32 / 7.0;
                let y = 0.15 + t * 0.55;
                let r = 0.10 * (1.0 - (t - 0.3).abs() * 0.4);
                add_sphere(&mut blobs, 0.5 + ox, y + oy, r, 3);
            }
            add_sphere(&mut blobs, 0.5 + ox, 0.18 + oy, 0.07, 3);
            // fins
            add_sphere(&mut blobs, 0.38 + ox, 0.62 + oy, 0.06, 2);
            add_sphere(&mut blobs, 0.62 + ox, 0.62 + oy, 0.06, 2);
            // flame
            for i in 0..5 {
                let t = i as f32 / 4.0;
                add_sphere(&mut blobs, 0.5 + ox, 0.78 + t * 0.12 + oy, 0.07 * (1.0 - t), 2);
            }
        }
        19 => {
            for i in 0..7 {
                let t = i as f32 / 6.0;
                add_sphere(&mut blobs, 0.5 + ox, 0.18 + t * 0.42 + oy, 0.075, 3);
            }
            add_sphere(&mut blobs, 0.5 + ox, 0.62 + oy, 0.11, 4);
            add_sphere(&mut blobs, 0.40 + ox, 0.78 + oy, 0.09, 3);
            add_sphere(&mut blobs, 0.60 + ox, 0.78 + oy, 0.09, 3);
        }
        20 => {
            add_sphere(&mut blobs, 0.36 + ox, 0.50 + oy, 0.18, 6);
            add_sphere(&mut blobs, 0.64 + ox, 0.50 + oy, 0.18, 6);
            add_sphere(&mut blobs, 0.36 + ox, 0.50 + oy, 0.04, 2);
            add_sphere(&mut blobs, 0.64 + ox, 0.50 + oy, 0.04, 2);
        }
        21 => { // НЛО
            add_sphere(&mut blobs, 0.5 + ox, 0.42 + oy, 0.14, 4);
            for i in 0..16 {
                let a = i as f32 / 16.0 * PI * 2.0;
                blobs.push(Blob {
                    x: 0.5 + 0.32 * a.cos() + ox,
                    y: 0.50 + 0.10 * a.sin() + oy,
                    z: 0.7,
                    r: 0.05,
                });
            }
            for lx in [-0.15f32, 0.0, 0.15] {
                for i in 0..4 {
                    let t = i as f32 / 3.0;
                    blobs.push(Blob {
                        x: 0.5 + lx + ox,
                        y: 0.58 + t * 0.22 + oy,
                        z: 0.4 - t * 0.15,
                        r: 0.035 * (1.0 - t * 0.5),
                    });
                }
            }
        }
        23 => { // Котик
            add_sphere(&mut blobs, 0.5 + ox, 0.55 + oy, 0.26, 5);
            add_sphere(&mut blobs, 0.34 + ox, 0.32 + oy, 0.08, 2);
            add_sphere(&mut blobs, 0.66 + ox, 0.32 + oy, 0.08, 2);
            add_sphere(&mut blobs, 0.40 + ox, 0.52 + oy, 0.045, 2);
            add_sphere(&mut blobs, 0.60 + ox, 0.52 + oy, 0.045, 2);
            add_sphere(&mut blobs, 0.50 + ox, 0.62 + oy, 0.03, 2);
        }
        4 | 24 => {
            add_sphere(&mut blobs, 0.5 + ox, 0.40 + oy, 0.24, 5);
            add_sphere(&mut blobs, 0.5 + ox, 0.62 + oy, 0.16, 3);
            add_sphere(&mut blobs, 0.40 + ox, 0.40 + oy, 0.05, 2);
            add_sphere(&mut blobs, 0.60 + ox, 0.40 + oy, 0.05, 2);
        }
        7 => {
            add_sphere(&mut blobs, 0.5 + ox, 0.5 + oy, 0.32, 6);
            if variant != 2 {
                add_sphere(&mut blobs, 0.62 + ox, 0.42 + oy, 0.24, 4);
            }
        }
        12 | 29 => {
            for i in 0..10 {
                let t = i as f32 / 9.0;
                let y = 0.80 - t * 0.55;
                let r = 0.06 + t * 0.12;
                let w = 0.04 * (t * 4.0).sin();
                add_sphere(&mut blobs, 0.5 + w + ox, y + oy, r, 2);
            }
        }
        _ => {
            add_sphere(&mut blobs, 0.5 + ox, 0.52 + oy, 0.30, 6);
            add_sphere(&mut blobs, 0.5 + ox, 0.28 + oy, 0.18, 4);
        }
    }

    ShapeMesh { blobs }
}

pub fn mesh_from_image(path: &Path, threshold: u8) -> Result<ShapeMesh, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_luma8();
    let (w, h) = img.dimensions();
    if w < 8 || h < 8 {
        return Err("Картинка слишком маленькая".into());
    }

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
    if min_x >= max_x {
        return Err("Не удалось выделить объект — измени порог".into());
    }

    let bw = (max_x - min_x) as f32;
    let bh = (max_y - min_y) as f32;
    let step = ((bw.min(bh) / 28.0) as u32).max(2);

    let mut blobs = Vec::new();
    for y in (min_y..=max_y).step_by(step as usize) {
        for x in (min_x..=max_x).step_by(step as usize) {
            let v = img.get_pixel(x, y).0[0];
            if v > threshold {
                let nx = (x - min_x) as f32 / bw;
                let ny = (y - min_y) as f32 / bh;
                blobs.push(Blob {
                    x: 0.5 + (nx - 0.5) * 0.82,
                    y: 0.5 + (ny - 0.5) * 0.82,
                    z: 0.3 + (v as f32 / 255.0) * 0.55,
                    r: 0.028 + (v as f32 / 255.0) * 0.02,
                });
            }
        }
    }
    if blobs.len() < 8 {
        return Err("Слишком мало точек".into());
    }
    Ok(ShapeMesh { blobs })
}

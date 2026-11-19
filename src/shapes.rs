//! Маски фигур как на телефоне: сетка on/depth + контуры.

use std::f32::consts::PI;
use std::path::Path;

pub const COLS: usize = 64;
pub const SHAPE_NAMES: &[&str] = &[
    "Капюшон", "Шар", "Сердце", "Звезда", "Череп", "Корона", "Молния", "Луна", "Алмаз",
    "Кошка", "Призрак", "Глаз", "Огонь", "Капля", "Облако", "Планета", "Курсор",
    "Ракета", "CS2", "Писюн", "Сиська", "НЛО", "Сердце 3D", "Котик", "Череп 2",
    "Молния 2", "Планета 2", "Ракета 2", "Глаз 2", "Огонь 2", "Звезда 2", "Алмаз 2",
];

pub fn code_variants(shape: usize) -> &'static [&'static str] {
    match shape {
        15 | 26 => &["Планета", "Земля", "Марс", "Сатурн", "Юпитер", "Чёрная дыра"],
        17 | 27 => &["Ракета", "Старт", "Космос", "Falcon", "Восток"],
        2 | 22 => &["Сердце", "Любовь", "Бит", "Двойное", "Пульс"],
        19 => &["Писюн", "Классика", "Толстый", "Длинный", "Неон"],
        20 => &["Сиська", "Классика", "Пара", "Неон", "Мягкая"],
        18 => &["CS2", "Counter", "AWP", "AK", "Terrorist"],
        21 => &["НЛО", "Тарелка", "Пришелец", "Луч", "Абдукция"],
        23 => &["Котик", "Мордочка", "Спящий", "Игривый", "Хвост"],
        1 => &["Шар", "Сфера", "Пузырь", "Орбита", "Ядро"],
        3 | 30 => &["Звезда", "5-луч", "8-луч", "Комета", "Искра"],
        _ => &["Классика", "Неон", "Контур", "Глитч", "Мини"],
    }
}

/// Символы «кода» как на телефоне
pub const CODE_CHARS: &[char] = &[
    '0', '1', '{', '}', '<', '>', '/', '*', '+', '-', '=',
    'x', 'o', '.', ':', ';', '#', '@', '&', '%', '$',
    'A', 'B', 'C', 'X', 'Y', 'Z', 'λ', 'Σ', 'π',
];

#[derive(Clone)]
pub struct Mask {
    pub rows: usize,
    pub on: Vec<bool>,
    pub depth: Vec<f32>,
}

impl Mask {
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * COLS + x
    }
}

fn empty_mask(rows: usize) -> Mask {
    let n = COLS * rows;
    Mask {
        rows,
        on: vec![false; n],
        depth: vec![0.0; n],
    }
}

fn fill_circle(m: &mut Mask, cx: f32, cy: f32, r: f32) {
    let rows = m.rows as f32;
    let cols = COLS as f32;
    let r2 = r * r;
    for y in 0..m.rows {
        for x in 0..COLS {
            let px = x as f32 / cols;
            let py = y as f32 / rows;
            let d2 = (px - cx).powi(2) + (py - cy).powi(2);
            if d2 <= r2 {
                let i = m.idx(x, y);
                m.on[i] = true;
                let d = 1.0 - (d2.sqrt() / r).clamp(0.0, 1.0);
                m.depth[i] = m.depth[i].max(d);
            }
        }
    }
}

fn fill_ellipse(m: &mut Mask, cx: f32, cy: f32, rx: f32, ry: f32) {
    let rows = m.rows as f32;
    let cols = COLS as f32;
    for y in 0..m.rows {
        for x in 0..COLS {
            let px = (x as f32 / cols - cx) / rx;
            let py = (y as f32 / rows - cy) / ry;
            let d2 = px * px + py * py;
            if d2 <= 1.0 {
                let i = m.idx(x, y);
                m.on[i] = true;
                m.depth[i] = m.depth[i].max(1.0 - d2.sqrt());
            }
        }
    }
}

fn fill_poly(m: &mut Mask, pts: &[[f32; 2]]) {
    if pts.len() < 3 {
        return;
    }
    let rows = m.rows as f32;
    let cols = COLS as f32;
    for y in 0..m.rows {
        for x in 0..COLS {
            let px = x as f32 / cols;
            let py = y as f32 / rows;
            if point_in_poly(px, py, pts) {
                let i = m.idx(x, y);
                m.on[i] = true;
                // depth from center of poly
                let mut cx = 0.0;
                let mut cy = 0.0;
                for p in pts {
                    cx += p[0];
                    cy += p[1];
                }
                cx /= pts.len() as f32;
                cy /= pts.len() as f32;
                let d = ((px - cx).hypot(py - cy) * 2.0).clamp(0.0, 1.0);
                m.depth[i] = m.depth[i].max(1.0 - d);
            }
        }
    }
}

fn point_in_poly(x: f32, y: f32, pts: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let n = pts.len();
    let mut j = n - 1;
    for i in 0..n {
        let yi = pts[i][1];
        let yj = pts[j][1];
        let xi = pts[i][0];
        let xj = pts[j][0];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + 1e-6) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn clear_circle(m: &mut Mask, cx: f32, cy: f32, r: f32) {
    let rows = m.rows as f32;
    let cols = COLS as f32;
    let r2 = r * r;
    for y in 0..m.rows {
        for x in 0..COLS {
            let px = x as f32 / cols;
            let py = y as f32 / rows;
            if (px - cx).powi(2) + (py - cy).powi(2) <= r2 {
                let i = m.idx(x, y);
                m.on[i] = false;
                m.depth[i] = 0.0;
            }
        }
    }
}

pub fn generate_mask(shape: usize, variant: usize) -> Mask {
    let rows = 72;
    let mut m = empty_mask(rows);
    let ox = ((variant % 3) as f32 - 1.0) * 0.02;
    let oy = (((variant / 3) % 3) as f32 - 1.0) * 0.015;

    match shape {
        1 => fill_circle(&mut m, 0.5 + ox, 0.5 + oy, 0.40),
        2 | 22 => {
            fill_circle(&mut m, 0.36 + ox, 0.38 + oy, 0.20);
            fill_circle(&mut m, 0.64 + ox, 0.38 + oy, 0.20);
            fill_poly(
                &mut m,
                &[
                    [0.18 + ox, 0.42 + oy],
                    [0.82 + ox, 0.42 + oy],
                    [0.50 + ox, 0.88 + oy],
                ],
            );
        }
        3 | 30 => {
            let rays = if variant % 2 == 0 { 5 } else { 8 };
            let mut pts = Vec::new();
            for i in 0..(rays * 2) {
                let a = -PI / 2.0 + i as f32 * PI / rays as f32;
                let r = if i % 2 == 0 { 0.42 } else { 0.17 };
                pts.push([0.5 + r * a.cos() + ox, 0.5 + r * a.sin() + oy]);
            }
            fill_poly(&mut m, &pts);
        }
        15 | 26 => {
            fill_circle(&mut m, 0.5 + ox, 0.5 + oy, 0.30);
            // rings
            let rings = if variant >= 3 { 2 } else if variant >= 1 { 1 } else { 0 };
            for ri in 0..rings {
                let rr = 0.38 + ri as f32 * 0.07;
                fill_ellipse(&mut m, 0.5 + ox, 0.5 + oy, rr, rr * 0.28);
                clear_circle(&mut m, 0.5 + ox, 0.5 + oy, rr - 0.035);
                // restore planet body
                fill_circle(&mut m, 0.5 + ox, 0.5 + oy, 0.30);
            }
            if variant == 5 {
                // black hole spiral hint
                for i in 0..40 {
                    let t = i as f32 / 39.0;
                    let a = t * PI * 6.0;
                    let r = 0.08 + t * 0.32;
                    fill_circle(
                        &mut m,
                        0.5 + r * a.cos() + ox,
                        0.5 + r * a.sin() + oy,
                        0.025,
                    );
                }
            }
        }
        17 | 27 => {
            fill_poly(
                &mut m,
                &[
                    [0.50 + ox, 0.08 + oy],
                    [0.62 + ox, 0.30 + oy],
                    [0.62 + ox, 0.65 + oy],
                    [0.50 + ox, 0.90 + oy],
                    [0.38 + ox, 0.65 + oy],
                    [0.38 + ox, 0.30 + oy],
                ],
            );
            fill_poly(
                &mut m,
                &[
                    [0.38 + ox, 0.58 + oy],
                    [0.22 + ox, 0.78 + oy],
                    [0.38 + ox, 0.70 + oy],
                ],
            );
            fill_poly(
                &mut m,
                &[
                    [0.62 + ox, 0.58 + oy],
                    [0.78 + ox, 0.78 + oy],
                    [0.62 + ox, 0.70 + oy],
                ],
            );
            clear_circle(&mut m, 0.50 + ox, 0.28 + oy, 0.05);
            // flame
            fill_poly(
                &mut m,
                &[
                    [0.42 + ox, 0.88 + oy],
                    [0.50 + ox, 0.98 + oy],
                    [0.58 + ox, 0.88 + oy],
                ],
            );
        }
        19 => {
            fill_ellipse(&mut m, 0.5 + ox, 0.40 + oy, 0.09, 0.28);
            fill_circle(&mut m, 0.5 + ox, 0.62 + oy, 0.11);
            fill_ellipse(&mut m, 0.40 + ox, 0.78 + oy, 0.09, 0.08);
            fill_ellipse(&mut m, 0.60 + ox, 0.78 + oy, 0.09, 0.08);
        }
        20 => {
            fill_circle(&mut m, 0.36 + ox, 0.50 + oy, 0.18);
            fill_circle(&mut m, 0.64 + ox, 0.50 + oy, 0.18);
            clear_circle(&mut m, 0.36 + ox, 0.50 + oy, 0.04);
            clear_circle(&mut m, 0.64 + ox, 0.50 + oy, 0.04);
        }
        21 => {
            fill_ellipse(&mut m, 0.5 + ox, 0.48 + oy, 0.36, 0.12);
            fill_ellipse(&mut m, 0.5 + ox, 0.40 + oy, 0.18, 0.10);
            clear_circle(&mut m, 0.5 + ox, 0.38 + oy, 0.05);
            for lx in [-0.18f32, 0.0, 0.18] {
                fill_poly(
                    &mut m,
                    &[
                        [0.5 + lx + ox, 0.55 + oy],
                        [0.5 + lx * 1.2 + ox, 0.90 + oy],
                        [0.5 + lx + 0.04 + ox, 0.55 + oy],
                    ],
                );
            }
        }
        23 => {
            fill_circle(&mut m, 0.5 + ox, 0.55 + oy, 0.28);
            fill_poly(
                &mut m,
                &[
                    [0.30 + ox, 0.40 + oy],
                    [0.26 + ox, 0.18 + oy],
                    [0.42 + ox, 0.38 + oy],
                ],
            );
            fill_poly(
                &mut m,
                &[
                    [0.70 + ox, 0.40 + oy],
                    [0.74 + ox, 0.18 + oy],
                    [0.58 + ox, 0.38 + oy],
                ],
            );
            clear_circle(&mut m, 0.40 + ox, 0.52 + oy, 0.045);
            clear_circle(&mut m, 0.60 + ox, 0.52 + oy, 0.045);
        }
        4 | 24 => {
            fill_circle(&mut m, 0.5 + ox, 0.38 + oy, 0.26);
            fill_ellipse(&mut m, 0.5 + ox, 0.62 + oy, 0.18, 0.16);
            clear_circle(&mut m, 0.40 + ox, 0.38 + oy, 0.055);
            clear_circle(&mut m, 0.60 + ox, 0.38 + oy, 0.055);
        }
        7 => {
            fill_circle(&mut m, 0.5 + ox, 0.5 + oy, 0.36);
            if variant != 2 {
                clear_circle(&mut m, 0.64 + ox, 0.40 + oy, 0.28);
            }
        }
        12 | 29 => {
            for i in 0..12 {
                let t = i as f32 / 11.0;
                let y = 0.85 - t * 0.60;
                let w = 0.08 + t * 0.12;
                fill_ellipse(&mut m, 0.5 + ox + 0.03 * (t * 6.0).sin(), y + oy, w, 0.06);
            }
        }
        6 | 25 => {
            fill_poly(
                &mut m,
                &[
                    [0.55 + ox, 0.08 + oy],
                    [0.30 + ox, 0.48 + oy],
                    [0.48 + ox, 0.48 + oy],
                    [0.38 + ox, 0.92 + oy],
                    [0.72 + ox, 0.42 + oy],
                    [0.52 + ox, 0.42 + oy],
                    [0.68 + ox, 0.08 + oy],
                ],
            );
        }
        8 | 31 => {
            fill_poly(
                &mut m,
                &[
                    [0.50 + ox, 0.12 + oy],
                    [0.78 + ox, 0.40 + oy],
                    [0.50 + ox, 0.90 + oy],
                    [0.22 + ox, 0.40 + oy],
                ],
            );
        }
        _ => {
            fill_ellipse(&mut m, 0.5 + ox, 0.55 + oy, 0.32, 0.36);
            fill_circle(&mut m, 0.5 + ox, 0.30 + oy, 0.22);
            clear_circle(&mut m, 0.5 + ox, 0.32 + oy, 0.10);
        }
    }

    // refine depth: distance transform-ish boost at center
    for y in 1..m.rows - 1 {
        for x in 1..COLS - 1 {
            let i = m.idx(x, y);
            if m.on[i] {
                let mut edge = false;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= COLS as i32 || ny >= m.rows as i32 {
                            edge = true;
                        } else if !m.on[m.idx(nx as usize, ny as usize)] {
                            edge = true;
                        }
                    }
                }
                if edge {
                    m.depth[i] *= 0.55;
                } else {
                    m.depth[i] = (m.depth[i] * 0.7 + 0.45).min(1.0);
                }
            }
        }
    }
    m
}

pub fn mask_from_image(path: &Path, threshold: u8) -> Result<Mask, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_luma8();
    let (w, h) = img.dimensions();
    let rows = 72;
    let mut m = empty_mask(rows);
    for y in 0..rows {
        for x in 0..COLS {
            let sx = x * w as usize / COLS;
            let sy = y * h as usize / rows;
            let v = img.get_pixel(sx as u32, sy as u32).0[0];
            if v > threshold {
                let i = m.idx(x, y);
                m.on[i] = true;
                m.depth[i] = (v as f32 / 255.0).clamp(0.15, 1.0);
            }
        }
    }
    let count = m.on.iter().filter(|&&b| b).count();
    if count < 30 {
        return Err("Мало пикселей — снизь порог".into());
    }
    Ok(m)
}

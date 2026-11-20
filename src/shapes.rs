//! Shape masks for wallpaper figure.

use std::f32::consts::PI;

pub const COLS: usize = 40;
pub const ROWS: usize = 48;

pub const SHAPE_NAMES: &[&str] = &[
    "Шар", "Сердце", "Звезда", "Планета", "Ракета", "Луна", "Котик", "Молния", "Алмаз", "Огонь",
];

pub const CODE_CHARS: &[char] = &[
    '0', '1', '{', '}', '<', '>', '/', '*', '#', '@', 'x', 'o', '.', ':', 'A', 'B', 'X', 'Y',
];

#[derive(Clone)]
pub struct Mask {
    pub on: Vec<bool>,
    pub depth: Vec<f32>,
}

fn idx(x: usize, y: usize) -> usize {
    y * COLS + x
}

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

fn fill_poly(on: &mut [bool], depth: &mut [f32], pts: &[[f32; 2]]) {
    if pts.len() < 3 {
        return;
    }
    let mut cx = 0.0_f32;
    let mut cy = 0.0_f32;
    for p in pts {
        cx += p[0];
        cy += p[1];
    }
    cx /= pts.len() as f32;
    cy /= pts.len() as f32;
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

pub fn generate_mask(shape: usize) -> Mask {
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
            fill_poly(
                &mut on,
                &mut depth,
                &[
                    [0.50, 0.08],
                    [0.64, 0.30],
                    [0.64, 0.62],
                    [0.50, 0.90],
                    [0.36, 0.62],
                    [0.36, 0.30],
                ],
            );
            fill_poly(&mut on, &mut depth, &[[0.36, 0.58], [0.20, 0.78], [0.36, 0.70]]);
            fill_poly(&mut on, &mut depth, &[[0.64, 0.58], [0.80, 0.78], [0.64, 0.70]]);
        }
        5 => fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.34),
        6 => {
            fill_circle(&mut on, &mut depth, 0.5, 0.55, 0.26);
            fill_poly(&mut on, &mut depth, &[[0.30, 0.40], [0.25, 0.16], [0.42, 0.38]]);
            fill_poly(&mut on, &mut depth, &[[0.70, 0.40], [0.75, 0.16], [0.58, 0.38]]);
        }
        7 => {
            fill_poly(
                &mut on,
                &mut depth,
                &[
                    [0.55, 0.08],
                    [0.30, 0.48],
                    [0.48, 0.48],
                    [0.38, 0.92],
                    [0.72, 0.42],
                    [0.52, 0.42],
                ],
            );
        }
        8 => fill_poly(&mut on, &mut depth, &[[0.50, 0.10], [0.80, 0.40], [0.50, 0.90], [0.20, 0.40]]),
        9 => {
            for i in 0..10 {
                let t = i as f32 / 9.0;
                fill_ellipse(
                    &mut on,
                    &mut depth,
                    0.5 + 0.03 * (t * 5.0).sin(),
                    0.85 - t * 0.55,
                    0.08 + t * 0.10,
                    0.05,
                );
            }
        }
        _ => fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.36),
    }
    if !on.iter().any(|&b| b) {
        fill_circle(&mut on, &mut depth, 0.5, 0.5, 0.3);
    }
    Mask { on, depth }
}

pub fn code_variants(shape: usize) -> &'static [&'static str] {
    match shape {
        3 => &["Классика", "Сатурн", "Марс"],
        1 => &["Классика", "Пульс"],
        _ => &["Классика"],
    }
}

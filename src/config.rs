//! Сохранение / загрузка .poi конфигов

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PoiCfg {
    pub shape: usize,
    pub variant: usize,
    pub color: [u8; 3],
    pub accent: [u8; 3],
    pub bg: [u8; 3],
    pub speed: f32,
    pub scale: f32,
    pub glow: bool,
    pub rainbow: bool,
    pub anim: usize,
    pub scene: usize,
    pub render_style: usize,
    pub shader_id: usize,
    pub char_size: f32,
    pub density: f32,
    pub show_clock: bool,
    pub show_date: bool,
    pub show_text: bool,
    pub text_val: String,
    pub w_size: f32,
    pub w_color: [u8; 3],
    pub w_style: usize,
    pub w_x: f32,
    pub w_y: f32,
    pub w_anim_mode: usize,
    pub w_anim_speed: f32,
    pub w_anim_amount: f32,
    pub prompt: String,
    pub gen_style: usize,
}

pub fn save_poi(path: &Path, cfg: &PoiCfg) -> Result<(), String> {
    let s = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, s).map_err(|e| e.to_string())
}

pub fn load_poi(path: &Path) -> Result<PoiCfg, String> {
    let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}

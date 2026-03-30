use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub monitors: Vec<MonitorConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorConfig {
    pub device_name: String,
    pub friendly_name: Option<String>,
    pub position_x: i32,
    pub position_y: i32,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub bits_per_pel: u32,
    /// Raw value of DEVMODE_DISPLAY_ORIENTATION (0=0°, 1=90°, 2=180°, 3=270°)
    pub orientation: u32,
    pub is_primary: bool,
}

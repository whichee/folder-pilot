use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// 收藏的文件夹路径
    pub favorites: Vec<String>,
    /// 自动扫描的根目录
    pub root_dirs: Vec<String>,
    /// 全局热键
    pub hotkey: String,
    /// 扫描深度
    pub scan_depth: usize,
    /// 是否开机自启
    pub autostart: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            favorites: Vec::new(),
            root_dirs: Vec::new(),
            hotkey: "Alt+Shift+F".to_string(),
            scan_depth: 3,
            autostart: true,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs_home().join(".config"));
        base.join("folder-pilot")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::config_path(), json).map_err(|e| e.to_string())
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

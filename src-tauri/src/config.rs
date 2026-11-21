#![allow(dead_code)] // tymczasowo

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)] // brakujące pola w pliku = wartości domyślne
pub struct Config {
    pub enabled: bool,
    pub window_start: NaiveTime,
    pub window_end: NaiveTime,
    pub alert_time: NaiveTime,
    pub sound_file: Option<String>,
    pub volume: f32,
    pub fullscreen: bool,
    pub catch_up_on_start: bool,
    pub warn_days_before_end: u32,
    pub last_triggered_date: Option<NaiveDate>,
    pub autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        let t = |h, m| NaiveTime::from_hms_opt(h, m, 0).unwrap();
        Config {
            enabled: true,
            window_start: t(6, 0),
            window_end: t(7, 0),
            alert_time: t(6, 30),
            sound_file: None,
            volume: 0.8,
            fullscreen: false,
            catch_up_on_start: true,
            warn_days_before_end: 7,
            last_triggered_date: None,
            autostart: true,
        }
    }
}

impl Config {
    /// Brak pliku = domyślne ustawienia. Uszkodzony plik = błąd.
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let text = fs::read_to_string(path).map_err(|e| format!("config: {e}"))?;
        serde_json::from_str(&text).map_err(|e| format!("config: błąd JSON: {e}"))
    }

    /// Zapis atomowy: plik tymczasowy + rename
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("wr_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("config.json")
    }

    #[test]
    fn missing_file_gives_default() {
        let p = tmp_path("missing");
        assert_eq!(Config::load(&p).unwrap(), Config::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let p = tmp_path("roundtrip");
        let mut c = Config::default();
        c.volume = 0.3;
        c.last_triggered_date = NaiveDate::from_ymd_opt(2026, 11, 24);
        c.save(&p).unwrap();
        assert_eq!(Config::load(&p).unwrap(), c);
    }

    #[test]
    fn partial_json_fills_defaults() {
        let p = tmp_path("partial");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, r#"{ "volume": 0.5 }"#).unwrap();
        let c = Config::load(&p).unwrap();
        assert_eq!(c.volume, 0.5);
        assert!(c.enabled);
    }

    #[test]
    fn corrupted_file_is_error() {
        let p = tmp_path("corrupt");
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "{ nie json").unwrap();
        assert!(Config::load(&p).is_err());
    }
}

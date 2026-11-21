#![allow(dead_code)] // tymczasowo, do czasu użycia w scheduler.rs

use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
pub struct Fraction {
    pub label: String,
    pub image: String,
    pub dates: Vec<NaiveDate>,
}

// Nieznane pola w JSON (code, settings, every_4_weeks_dates) są ignorowane
#[derive(Debug, Clone, Deserialize)]
pub struct Schedule {
    pub schema_version: u32,
    pub region: String,
    pub year: i32,
    pub fractions: BTreeMap<String, Fraction>,
}

impl Schedule {
    pub fn parse(json: &str) -> Result<Self, String> {
        let s: Schedule = serde_json::from_str(json).map_err(|e| format!("Błąd JSON: {e}"))?;
        s.validate()?;
        Ok(s)
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "Nieobsługiwany schema_version {} (oczekiwano {})",
                self.schema_version, SCHEMA_VERSION
            ));
        }
        for (key, f) in &self.fractions {
            if let Some(d) = f.dates.iter().find(|d| d.year() != self.year) {
                return Err(format!("{key}: data {d} nie pasuje do roku {}", self.year));
            }
            let mut v = f.dates.clone();
            v.sort();
            v.dedup();
            if v.len() != f.dates.len() {
                return Err(format!("{key}: zduplikowane daty"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Schedules(pub Vec<Schedule>);

impl Schedules {
    /// Wczytuje wszystkie pliki trash_schedule_YYYY.json z folderu
    pub fn load_dir(dir: &Path) -> Result<Self, String> {
        let mut all = Vec::new();
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if !(name.starts_with("trash_schedule_") && name.ends_with(".json")) {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|e| format!("{name}: {e}"))?;
            let s = Schedule::parse(&text).map_err(|e| format!("{name}: {e}"))?;
            if name != format!("trash_schedule_{}.json", s.year) {
                return Err(format!(
                    "{name}: nazwa pliku nie zgadza się z year={}",
                    s.year
                ));
            }
            all.push(s);
        }
        all.sort_by_key(|s| s.year);
        Ok(Schedules(all))
    }

    /// Frakcje na dany dzień (0, 1 lub więcej)
    pub fn fractions_for(&self, date: NaiveDate) -> Vec<(&str, &Fraction)> {
        self.0
            .iter()
            .flat_map(|s| s.fractions.iter())
            .filter(|(_, f)| f.dates.contains(&date))
            .map(|(k, f)| (k.as_str(), f))
            .collect()
    }

    /// Najbliższy wywóz od `from` (włącznie)
    pub fn next_pickup(&self, from: NaiveDate) -> Option<NaiveDate> {
        self.all_dates().filter(|d| *d >= from).min()
    }

    pub fn last_date(&self) -> Option<NaiveDate> {
        self.all_dates().max()
    }

    fn all_dates(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        self.0
            .iter()
            .flat_map(|s| s.fractions.values())
            .flat_map(|f| f.dates.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{
      "schema_version": 1, "region": "Olesno II", "year": 2026,
      "fractions": {
        "komunalne": { "label": "Zmieszane", "image": "k.png", "dates": ["2026-01-13", "2026-12-30"] },
        "papier":    { "label": "Papier",    "image": "p.png", "dates": ["2026-01-13", "2026-02-03"] }
      }
    }"#;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn schedules() -> Schedules {
        Schedules(vec![Schedule::parse(JSON).unwrap()])
    }

    #[test]
    fn one_fraction() {
        let s = schedules();
        let r = s.fractions_for(d(2026, 2, 3));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, "papier");
    }

    #[test]
    fn two_fractions_same_day() {
        assert_eq!(schedules().fractions_for(d(2026, 1, 13)).len(), 2);
    }

    #[test]
    fn no_fraction() {
        assert!(schedules().fractions_for(d(2026, 1, 14)).is_empty());
    }

    #[test]
    fn next_pickup_and_last_date() {
        let s = schedules();
        assert_eq!(s.next_pickup(d(2026, 1, 14)), Some(d(2026, 2, 3)));
        assert_eq!(s.next_pickup(d(2026, 2, 3)), Some(d(2026, 2, 3)));
        assert_eq!(s.next_pickup(d(2027, 1, 1)), None);
        assert_eq!(s.last_date(), Some(d(2026, 12, 30)));
    }

    #[test]
    fn rejects_wrong_year() {
        let bad = JSON.replace("2026-02-03", "2027-02-03");
        assert!(Schedule::parse(&bad).is_err());
    }

    #[test]
    fn rejects_duplicates() {
        let bad = JSON.replace("\"2026-02-03\"", "\"2026-01-13\"");
        assert!(Schedule::parse(&bad).is_err());
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let bad = JSON.replace("\"schema_version\": 1", "\"schema_version\": 2");
        assert!(Schedule::parse(&bad).is_err());
    }
}

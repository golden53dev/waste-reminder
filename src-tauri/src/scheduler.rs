#![allow(dead_code)] // tymczasowo

use crate::config::Config;
use crate::schedule::Schedules;
use chrono::NaiveDateTime;

#[derive(Debug, PartialEq)]
pub enum Decision {
    Nothing,
    Alarm(Vec<String>), // klucze frakcji, np. ["komunalne", "papier"]
    Expired,            // brak danych na dziś (harmonogram wygasł)
}

/// Decyzja co zrobić w danej chwili. Wołana co 30 s oraz przy starcie appki.
pub fn decide(now: NaiveDateTime, cfg: &Config, s: &Schedules) -> Decision {
    if !cfg.enabled {
        return Decision::Nothing;
    }
    let today = now.date();
    if cfg.last_triggered_date == Some(today) {
        return Decision::Nothing; // już odhaczone przez X
    }
    let t = now.time();
    if t < cfg.alert_time {
        return Decision::Nothing;
    }
    // po oknie (np. PC włączony o 8:00) tylko gdy włączony catch-up
    if t > cfg.window_end && !cfg.catch_up_on_start {
        return Decision::Nothing;
    }
    match s.last_date() {
        Some(last) if today <= last => {}
        _ => return Decision::Expired,
    }
    let keys: Vec<String> = s
        .fractions_for(today)
        .iter()
        .map(|(k, _)| k.to_string())
        .collect();
    if keys.is_empty() {
        Decision::Nothing
    } else {
        Decision::Alarm(keys)
    }
}

/// Czy pokazać ostrzeżenie w tray (harmonogram zaraz się kończy)
pub fn is_warning(now: NaiveDateTime, cfg: &Config, s: &Schedules) -> bool {
    match s.last_date() {
        Some(last) => {
            let left = (last - now.date()).num_days();
            (0..=cfg.warn_days_before_end as i64).contains(&left)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schedule::Schedule;
    use chrono::NaiveDate;

    const JSON: &str = r#"{
      "schema_version": 1, "region": "Olesno II", "year": 2026,
      "fractions": {
        "komunalne": { "label": "Zmieszane", "image": "k.png", "dates": ["2026-01-13", "2026-12-30"] },
        "papier":    { "label": "Papier",    "image": "p.png", "dates": ["2026-01-13", "2026-02-03"] }
      }
    }"#;

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap()
    }
    fn sch() -> Schedules {
        Schedules(vec![Schedule::parse(JSON).unwrap()])
    }

    #[test]
    fn before_alert_time() {
        assert_eq!(
            decide(dt(2026, 2, 3, 6, 29), &Config::default(), &sch()),
            Decision::Nothing
        );
    }

    #[test]
    fn fires_at_alert_time() {
        assert_eq!(
            decide(dt(2026, 2, 3, 6, 30), &Config::default(), &sch()),
            Decision::Alarm(vec!["papier".to_string()])
        );
    }

    #[test]
    fn two_fractions_alarm() {
        match decide(dt(2026, 1, 13, 6, 30), &Config::default(), &sch()) {
            Decision::Alarm(k) => assert_eq!(k.len(), 2),
            other => panic!("oczekiwano Alarm, jest {other:?}"),
        }
    }

    #[test]
    fn acked_today() {
        let mut c = Config::default();
        c.last_triggered_date = NaiveDate::from_ymd_opt(2026, 2, 3);
        assert_eq!(decide(dt(2026, 2, 3, 6, 45), &c, &sch()), Decision::Nothing);
    }

    #[test]
    fn no_pickup_today() {
        assert_eq!(
            decide(dt(2026, 2, 4, 6, 30), &Config::default(), &sch()),
            Decision::Nothing
        );
    }

    #[test]
    fn catch_up_on_and_off() {
        let mut c = Config::default();
        assert!(matches!(
            decide(dt(2026, 2, 3, 8, 0), &c, &sch()),
            Decision::Alarm(_)
        ));
        c.catch_up_on_start = false;
        assert_eq!(decide(dt(2026, 2, 3, 8, 0), &c, &sch()), Decision::Nothing);
    }

    #[test]
    fn disabled() {
        let mut c = Config::default();
        c.enabled = false;
        assert_eq!(decide(dt(2026, 2, 3, 6, 30), &c, &sch()), Decision::Nothing);
    }

    #[test]
    fn expired_after_last_date() {
        assert_eq!(
            decide(dt(2026, 12, 31, 6, 30), &Config::default(), &sch()),
            Decision::Expired
        );
        assert_eq!(
            decide(
                dt(2026, 12, 31, 6, 30),
                &Config::default(),
                &Schedules::default()
            ),
            Decision::Expired
        );
    }

    #[test]
    fn warning_window() {
        let c = Config::default(); // 7 dni
        assert!(is_warning(dt(2026, 12, 23, 6, 0), &c, &sch()));
        assert!(!is_warning(dt(2026, 12, 22, 6, 0), &c, &sch()));
        assert!(!is_warning(dt(2026, 12, 31, 6, 0), &c, &sch()));
    }
}

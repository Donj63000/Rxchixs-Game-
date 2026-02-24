#![allow(dead_code)]

use std::collections::VecDeque;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogCategorie {
    Systeme,
    Deplacement,
    Travail,
    Social,
    Ordre,
    Etat,
    Debug,
}

impl LogCategorie {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Systeme => "Systeme",
            Self::Deplacement => "Deplacement",
            Self::Travail => "Travail",
            Self::Social => "Social",
            Self::Ordre => "Ordre",
            Self::Etat => "Etat",
            Self::Debug => "Debogage",
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogEntree {
    pub t_sim_s: f64,
    pub stamp: String,
    pub cat: LogCategorie,
    pub msg: String,
}

#[derive(Clone, Debug)]
pub struct HistoriqueLog {
    cap: usize,
    entries: VecDeque<LogEntree>,
}

impl HistoriqueLog {
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(64),
            entries: VecDeque::new(),
        }
    }

    pub fn push(&mut self, sim_time_s: f64, cat: LogCategorie, msg: impl Into<String>) {
        while self.entries.len() >= self.cap {
            self.entries.pop_front();
        }
        let stamp = format_timestamp_fr(sim_time_s);
        self.entries.push_back(LogEntree {
            t_sim_s: sim_time_s,
            stamp,
            cat,
            msg: msg.into(),
        });
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &LogEntree> {
        self.entries.iter()
    }
}

pub fn format_timestamp_fr(sim_time_s: f64) -> String {
    let seconds = sim_time_s.max(0.0);
    let minutes = (seconds / 60.0).floor() as i64;
    let sec = (seconds - minutes as f64 * 60.0).floor() as i64;
    format!("{minutes:02}:{sec:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_clamps_negative_and_formats_mm_ss() {
        assert_eq!(format_timestamp_fr(-10.0), "00:00");
        assert_eq!(format_timestamp_fr(0.0), "00:00");
        assert_eq!(format_timestamp_fr(65.9), "01:05");
        assert_eq!(format_timestamp_fr(600.0), "10:00");
    }

    #[test]
    fn categories_have_stable_labels() {
        assert_eq!(LogCategorie::Systeme.label(), "Systeme");
        assert_eq!(LogCategorie::Deplacement.label(), "Deplacement");
        assert_eq!(LogCategorie::Travail.label(), "Travail");
        assert_eq!(LogCategorie::Social.label(), "Social");
        assert_eq!(LogCategorie::Ordre.label(), "Ordre");
        assert_eq!(LogCategorie::Etat.label(), "Etat");
        assert_eq!(LogCategorie::Debug.label(), "Debogage");
    }

    #[test]
    fn min_capacity_is_enforced_and_oldest_entries_are_dropped() {
        let mut log = HistoriqueLog::new(1);
        for i in 0..70 {
            log.push(i as f64, LogCategorie::Debug, format!("evt-{i}"));
        }

        assert_eq!(log.len(), 64);
        let first = log.iter().next().expect("au moins une entree");
        let last = log.iter().next_back().expect("au moins une entree");
        assert_eq!(first.msg, "evt-6");
        assert_eq!(last.msg, "evt-69");
    }

    #[test]
    fn iterator_is_double_ended_and_preserves_order() {
        let mut log = HistoriqueLog::new(64);
        log.push(1.0, LogCategorie::Systeme, "A");
        log.push(2.0, LogCategorie::Systeme, "B");
        log.push(3.0, LogCategorie::Systeme, "C");

        let mut it = log.iter();
        assert_eq!(it.next().map(|e| e.msg.as_str()), Some("A"));
        assert_eq!(it.next_back().map(|e| e.msg.as_str()), Some("C"));
        assert_eq!(it.next().map(|e| e.msg.as_str()), Some("B"));
        assert!(it.next().is_none());
    }
}

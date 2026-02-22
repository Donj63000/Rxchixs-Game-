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

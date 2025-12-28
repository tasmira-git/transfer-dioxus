use dioxus::prelude::*;
use std::{
    fmt::Display,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Clone, PartialEq)]
pub enum Language {
    English,
    Chinese,
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Chinese => write!(f, "中文"),
        }
    }
}

#[derive(Clone)]
pub struct ReceiverState {
    pub port: Signal<u16>,
    pub dir: Signal<PathBuf>,
    pub logs: Signal<Vec<String>>,
    pub is_running: Signal<Arc<AtomicBool>>,
    pub log_tx: Signal<UnboundedSender<String>>,
}
impl ReceiverState {
    pub fn new(log_tx: Signal<UnboundedSender<String>>, logs: Signal<Vec<String>>) -> Self {
        Self {
            port: Signal::new(8000),
            dir: Signal::new(std::fs::canonicalize(".").unwrap()),
            logs,
            is_running: Signal::new(Arc::new(AtomicBool::new(false))),
            log_tx,
        }
    }
}

#[derive(Clone)]
pub struct SenderState {
    pub ip: Signal<String>,
    pub port: Signal<u16>,
    pub enable_directory: Signal<bool>,
    pub file: Signal<PathBuf>,
    pub logs: Signal<Vec<String>>,
    pub is_running: Signal<Arc<AtomicBool>>,
    pub log_tx: Signal<UnboundedSender<String>>,
    pub progress_tx: Signal<UnboundedSender<(f64, String)>>,
    pub progress: Signal<(f64, String)>,
}
impl SenderState {
    pub fn new(
        log_tx: Signal<UnboundedSender<String>>,
        logs: Signal<Vec<String>>,
        progress_tx: Signal<UnboundedSender<(f64, String)>>,
        progress: Signal<(f64, String)>,
    ) -> Self {
        Self {
            ip: Signal::new("127.0.0.1".to_string()),
            port: Signal::new(8000),
            enable_directory: Signal::new(false),
            file: Signal::new(PathBuf::new()),
            logs,
            is_running: Signal::new(Arc::new(AtomicBool::new(false))),
            log_tx,
            progress_tx,
            progress,
        }
    }
}

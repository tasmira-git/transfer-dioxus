use crate::form_field::{use_form_field, FormField};
use dioxus::prelude::*;
use rust_i18n::t;
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr},
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
    pub port_field: FormField<u16>,
    pub dir: Signal<PathBuf>,
    pub logs: Signal<Vec<String>>,
    pub is_running: Signal<Arc<AtomicBool>>,
    pub log_tx: Signal<UnboundedSender<String>>,
}
pub fn use_receiver_state(
    log_tx: Signal<UnboundedSender<String>>,
    logs: Signal<Vec<String>>,
) -> ReceiverState {
    let port_field = use_form_field(8000_u16, |s| s.parse().map_err(|_| t!("port_validation")));

    ReceiverState {
        port_field,
        dir: Signal::new(std::fs::canonicalize(".").unwrap()),
        logs,
        is_running: Signal::new(Arc::new(AtomicBool::new(false))),
        log_tx,
    }
}

#[derive(Clone)]
pub struct SenderState {
    pub ip_field: FormField<IpAddr>,
    pub port_field: FormField<u16>,
    pub enable_directory: Signal<bool>,
    pub file: Signal<PathBuf>,
    pub logs: Signal<Vec<String>>,
    pub is_running: Signal<Arc<AtomicBool>>,
    pub log_tx: Signal<UnboundedSender<String>>,
    pub progress_tx: Signal<UnboundedSender<(f64, String)>>,
    pub progress: Signal<(f64, String)>,
}

pub fn use_sender_state(
    log_tx: Signal<UnboundedSender<String>>,
    logs: Signal<Vec<String>>,
    progress_tx: Signal<UnboundedSender<(f64, String)>>,
    progress: Signal<(f64, String)>,
) -> SenderState {
    let port_field = use_form_field(8000_u16, |s| s.parse().map_err(|_| t!("port_validation")));
    let ip_field = use_form_field(Ipv4Addr::LOCALHOST.into(), |s| {
        s.parse().map_err(|_| t!("invalid_ip"))
    });

    SenderState {
        ip_field,
        port_field,
        enable_directory: use_signal(|| false),
        file: use_signal(PathBuf::new),
        logs,
        is_running: use_signal(|| Arc::new(AtomicBool::new(false))),
        log_tx,
        progress_tx,
        progress,
    }
}

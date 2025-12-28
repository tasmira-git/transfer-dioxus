use crate::transfer_protocol::receive_protocol::ReceiveProtocol;
use anyhow::Context;
use dioxus::hooks::UnboundedSender;
use rust_i18n::t;
use std::io::ErrorKind;
use std::sync::atomic::Ordering::Relaxed;
use std::{
    fs::create_dir_all,
    net::{TcpListener, ToSocketAddrs},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

pub fn handle_receive(
    addr: impl ToSocketAddrs,
    save_path: PathBuf,
    log_tx: UnboundedSender<String>,
    running: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    create_dir_all(&save_path)?;
    log_tx.unbounded_send(format!("{} : {save_path:?}", t!("save_path")))?;

    let listener = TcpListener::bind(addr).with_context(|| t!("change_port"))?;
    log_tx.unbounded_send(format!(
        "{} : {}",
        t!("start_server_success"),
        listener.local_addr()?
    ))?;

    listener
        .set_nonblocking(true)
        .with_context(|| "设置非阻塞模式失败")?;

    while running.load(Relaxed) {
        match listener.accept() {
            Ok((stream, a)) => {
                log_tx.unbounded_send(format!("{} : {}", t!("new_connection"), a))?;

                let save_path = save_path.clone();
                let log_thread = log_tx.clone();
                std::thread::spawn(move || {
                    let mut stream = ReceiveProtocol::new(stream);
                    match stream.receive_file_or_dir(&save_path, &log_thread) {
                        Ok(_) => {
                            _ = log_thread.unbounded_send(t!("receive_over").to_string());
                            _ = log_thread
                                .unbounded_send(format!("{} : {save_path:?}", t!("save_path")));
                        }
                        Err(e) => {
                            _ = log_thread.unbounded_send(format!(
                                "{} : {}",
                                t!("receive_fail"),
                                e
                            ));
                        }
                    };
                });
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                anyhow::bail!(e);
            }
        }
    }
    Ok(())
}

use crate::transfer_protocol::receive_protocol::ReceiveProtocol;
use anyhow::Context;
use dioxus::hooks::UnboundedSender;
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
    output_path: PathBuf,
    log_tx: UnboundedSender<String>,
    running: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    create_dir_all(&output_path)?;
    log_tx.unbounded_send(format!("保存文件的目录：{output_path:?}"))?;

    let listener = TcpListener::bind(addr).with_context(|| "无法启动服务器, 尝试更换端口")?;
    log_tx.unbounded_send(format!("服务器启动，监听{}", listener.local_addr()?))?;
    listener
        .set_nonblocking(true)
        .with_context(|| "设置非阻塞模式失败")?;

    while running.load(Relaxed) {
        match listener.accept() {
            Ok((stream, a)) => {
                log_tx.unbounded_send(format!("新连接：{}", a))?;

                log_tx.unbounded_send("接收文件中...".to_string())?;
                let output_path = output_path.clone();
                let log_thread = log_tx.clone();
                std::thread::spawn(move || {
                    let mut stream = ReceiveProtocol::new(stream);
                    match stream.receive_file_or_dir(&output_path, &log_thread) {
                        Ok(_) => {
                            _ = log_thread.unbounded_send("接收任务完成".to_string());
                            _ = log_thread.unbounded_send(format!("文件存放在：{output_path:?}"));
                        }
                        Err(e) => {
                            _ = log_thread.unbounded_send(format!("接收文件失败：{}", e));
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

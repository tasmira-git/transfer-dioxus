use crate::transfer_protocol::receive_protocol::ReceiveProtocol;
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
) {
    let logger = log_tx.clone();
    let log = move |msg: String| {
        _ = logger.unbounded_send(msg);
    };

    create_dir_all(&output_path).unwrap();
    log(format!("保存文件的目录：{output_path:?}"));

    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(e) => {
            running.store(false, Relaxed);
            log(format!("无法启动服务器：{}", e));
            return;
        }
    };
    log(format!(
        "服务器启动，监听{}",
        listener.local_addr().unwrap()
    ));
    if let Err(e) = listener.set_nonblocking(true) {
        running.store(false, Relaxed);
        log(format!("无法设置非阻塞模式：{}", e));
        return;
    }

    while running.load(Relaxed) {
        let output_path = output_path.clone();
        let log_thread = log_tx.clone();

        match listener.accept() {
            Ok((stream, a)) => {
                log(format!("新连接：{}", a));

                log("接收文件中...".to_string());
                std::thread::spawn(move || {
                    let mut stream = ReceiveProtocol::new(stream);
                    stream.receive_file_or_dir(&output_path, &log_thread);
                    _ = log_thread.unbounded_send("接收任务完成".to_string());
                    _ = log_thread.unbounded_send(format!("文件存放在：{output_path:?}"));
                });
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => {
                log(format!("无法接受连接：{}", e));
            }
        }
    }
    log("接收服务已停止".to_string());
}

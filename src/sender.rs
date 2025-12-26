use crate::transfer_protocol::send_protocol::SendProtocol;
use dioxus::hooks::UnboundedSender;
use std::sync::atomic::Ordering::Relaxed;
use std::{
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};
use walkdir::WalkDir;

pub fn handle_send(
    addr: impl ToSocketAddrs,
    send_path: PathBuf,
    is_running: Arc<AtomicBool>,
    log_tx: UnboundedSender<String>,
    progress_tx: UnboundedSender<(f64, String)>,
) -> anyhow::Result<()> {
    if !send_path.exists() {
        is_running.store(false, Relaxed);
        _ = log_tx.unbounded_send("发送路径不存在".to_string());
        anyhow::bail!("发送路径不存在");
        // return anyhow::anyhow!("发送路径不存在");
    }

    let total_size = WalkDir::new(&send_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file()) // 只计算文件
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum::<u64>();

    let stream = match TcpStream::connect(addr) {
        Ok(stream) => {
            _ = log_tx.unbounded_send("连接到服务器成功".to_string());
            stream
        }
        Err(e) => {
            is_running.store(false, Relaxed);
            _ = log_tx.unbounded_send(format!("连接到服务器失败：{}", e));
            anyhow::bail!("连接到服务器失败");
            // return;
        }
    };

    let root_dir = send_path.parent().unwrap();
    let paths = WalkDir::new(&send_path);

    let mut stream = SendProtocol::new(stream, total_size, progress_tx);

    _ = log_tx.unbounded_send("发送文件中...".to_string());
    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();

        stream.send_file_or_dir(path, root_dir, &log_tx);
    }

    stream.flush();
    stream.get_ref().send_process();

    is_running.store(false, Relaxed);
    _ = log_tx.unbounded_send(format!(
        "发送任务完成，耗时: {:?}",
        stream.get_ref().total_time()
    ));

    Ok(())
}

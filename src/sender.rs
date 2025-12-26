use crate::transfer_protocol::send_protocol::SendProtocol;
use anyhow::Context;
use dioxus::hooks::UnboundedSender;
use std::{
    fmt::Debug,
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    time::Duration,
};
use walkdir::WalkDir;

pub fn handle_send(
    addr: impl ToSocketAddrs + Debug,
    send_path: PathBuf,
    log_tx: UnboundedSender<String>,
    progress_tx: UnboundedSender<(f64, String)>,
) -> anyhow::Result<()> {
    if !send_path.exists() {
        anyhow::bail!("发送路径不存在");
    }

    let total_size = WalkDir::new(&send_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file()) // 只计算文件
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum::<u64>();

    let mut addr = addr
        .to_socket_addrs()
        .with_context(|| format!("{addr:?}不是一个有效的ip地址"))?;
    let socket_addr = addr.next().context("没有ip地址")?;
    let stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3))?;
    log_tx.unbounded_send("连接到服务器成功".to_string())?;

    let root_dir = send_path.parent().context("发送路径是根目录或空")?;
    let paths = WalkDir::new(&send_path);

    let mut stream = SendProtocol::new(stream, total_size, progress_tx);

    log_tx.unbounded_send("发送文件中...".to_string())?;
    for entry in paths {
        if let Ok(entry) = entry {
            let path = entry.path();
            stream.send_file_or_dir(path, root_dir, &log_tx)?;
        }
    }

    stream.flush()?;
    stream.get_ref().send_process();

    log_tx.unbounded_send(format!(
        "任务完成，耗时: {:?}",
        stream.get_ref().total_time()
    ))?;

    Ok(())
}

use crate::transfer_protocol::{progress_writer::ProgressWriter, TYPE_DIR, TYPE_FILE};
use anyhow::Context;
use dioxus::hooks::UnboundedSender;
use rust_i18n::t;
use std::{
    io::{BufWriter, Write},
    net::TcpStream,
    path::Path,
};

type MonitorStream = ProgressWriter<TcpStream>;

pub struct SendProtocol {
    writer: BufWriter<MonitorStream>,
}

impl SendProtocol {
    pub fn new(
        stream: TcpStream,
        total_size: u64,
        progress_tx: UnboundedSender<(f64, String)>,
    ) -> Self {
        let monitor = MonitorStream::new(stream, total_size, progress_tx);
        Self {
            writer: BufWriter::new(monitor),
        }
    }
    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
    pub fn get_ref(&self) -> &MonitorStream {
        self.writer.get_ref()
    }

    pub fn send_file_or_dir(
        &mut self,
        path: &Path,
        root_dir: &Path,
        log_tx: &UnboundedSender<String>,
    ) -> anyhow::Result<()> {
        self.send_file_type(path)?;

        let relative_path = path.strip_prefix(root_dir)?;
        self.send_path_name(&relative_path)?;

        if path.is_file() {
            log_tx.unbounded_send(format!("{} : {relative_path:?}", t!("send")))?;
            self.send_file(path)?;
        }
        Ok(())
    }

    fn send_file_type(&mut self, path: &Path) -> anyhow::Result<()> {
        if path.is_file() {
            self.writer.write_all(&[TYPE_FILE])?;
        } else {
            self.writer.write_all(&[TYPE_DIR])?;
        }
        Ok(())
    }

    fn send_path_name(&mut self, path_name: &Path) -> anyhow::Result<()> {
        let path_name = path_name
            .to_str()
            .with_context(|| format!("{path_name:?}不是有效的unicode"))?
            .as_bytes();
        let path_name_len: [u8; 2] = (path_name.len() as u16).to_be_bytes();

        self.writer.write_all(&path_name_len)?;
        self.writer.write_all(path_name)?;
        Ok(())
    }

    fn send_file(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let metadata = file_path.metadata()?;
        let file_size: [u8; 8] = metadata.len().to_be_bytes();

        self.writer.write_all(&file_size)?;

        let mut file = std::fs::File::open(file_path)?;

        self.writer.get_mut().start_monitor();
        std::io::copy(&mut file, &mut self.writer)?;
        self.writer.get_mut().stop_monitor();
        Ok(())
    }
}

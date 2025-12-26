use crate::{
    progress_writer::ProgressWriter,
    transfer_protocol::{TYPE_DIR, TYPE_FILE},
};
use dioxus::hooks::UnboundedSender;
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
    pub fn flush(&mut self) {
        self.writer.flush().unwrap();
    }
    pub fn get_ref(&self) -> &MonitorStream {
        self.writer.get_ref()
    }

    pub fn send_file_or_dir(
        &mut self,
        path: &Path,
        root_dir: &Path,
        log_tx: &UnboundedSender<String>,
    ) {
        self.send_file_type(path);

        let relative_path = path.strip_prefix(root_dir).unwrap();
        self.send_path_name(&relative_path);

        if path.is_file() {
            _ = log_tx.unbounded_send(format!("发送{relative_path:?}"));
            self.send_file(path);
        }
    }

    fn send_file_type(&mut self, path: &Path) {
        if path.is_file() {
            self.writer.write_all(&[TYPE_FILE]).unwrap();
        } else {
            self.writer.write_all(&[TYPE_DIR]).unwrap();
        }
    }

    fn send_path_name(&mut self, path_name: &Path) {
        let path_name = path_name.to_str().unwrap().as_bytes();
        let path_name_len: [u8; 2] = (path_name.len() as u16).to_be_bytes();

        self.writer.write_all(&path_name_len).unwrap();
        self.writer.write_all(path_name).unwrap();
    }

    fn send_file(&mut self, file_path: &Path) {
        let metadata = file_path.metadata().unwrap();
        let file_size: [u8; 8] = metadata.len().to_be_bytes();

        self.writer.write_all(&file_size).unwrap();

        let mut file = std::fs::File::open(file_path).unwrap();

        self.writer.get_mut().start_monitor();
        std::io::copy(&mut file, &mut self.writer).unwrap();
        self.writer.get_mut().stop_monitor();
    }
}

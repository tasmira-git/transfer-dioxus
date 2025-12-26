use dioxus::hooks::UnboundedSender;
use std::{
    io::Write,
    time::{Duration, Instant},
};

pub struct ProgressWriter<W: Write> {
    inner: W,
    total_size: u64,
    bytes_send: u64,
    last_send_time: Instant,
    start_time: Instant,
    progress_tx: UnboundedSender<(f64, String)>,
    monitor: bool,
}

impl<W: Write> ProgressWriter<W> {
    pub fn new(inner: W, total_size: u64, progress_tx: UnboundedSender<(f64, String)>) -> Self {
        Self {
            inner,
            total_size,
            bytes_send: 0,
            last_send_time: Instant::now(),
            start_time: Instant::now(),
            progress_tx,
            monitor: false,
        }
    }
    pub fn start_monitor(&mut self) {
        self.monitor = true;
    }
    pub fn stop_monitor(&mut self) {
        self.monitor = false;
    }
    pub fn total_time(&self) -> Duration {
        self.start_time.elapsed()
    }
    pub fn send_process(&self) {
        let percentage = if self.total_size > 0 {
            (self.bytes_send as f64 / self.total_size as f64) * 100.0
        } else {
            0.0
        };

        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        let speed = if elapsed_secs > 0.0 {
            self.bytes_send as f64 / elapsed_secs
        } else {
            0.0
        };
        let mut speed = Self::format_size(speed);
        speed.push_str("/s");

        _ = self.progress_tx.unbounded_send((percentage, speed));
    }

    fn format_size(mut size: f64) -> String {
        let display = ["B", "KB", "MB", "GB", "TB"];
        let mut display_index = 0;

        while size >= 1024_f64 {
            size /= 1024_f64;
            display_index += 1;
        }
        format!("{:.2}{}", size, display[display_index])
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;

        if self.monitor {
            self.bytes_send += n as u64;

            if self.last_send_time.elapsed() >= Duration::from_millis(500) {
                self.send_process();
                self.last_send_time = Instant::now();
            }
        }
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

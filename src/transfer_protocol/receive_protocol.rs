use std::{
    fs::create_dir_all,
    io::{BufReader, Read},
    net::TcpStream,
    path::{Path, PathBuf},
};

use dioxus::hooks::UnboundedSender;

use crate::transfer_protocol::TYPE_FILE;

pub struct ReceiveProtocol {
    reader: BufReader<TcpStream>,
}
impl ReceiveProtocol {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            reader: BufReader::new(stream),
        }
    }

    pub fn receive_file_or_dir(&mut self, save_path: &Path, log: &UnboundedSender<String>) {
        loop {
            let Some(is_file) = self.receive_file_type() else {
                break;
            };

            if is_file {
                // tracing::debug!("接收文件：");
            } else {
                // tracing::debug!("接收目录：");
            }

            let receive_path = self.receive_file_path();
            // tracing::debug!("   {}", receive_path.display());

            if is_file {
                let save_path = save_path.join(&receive_path);
                self.receive_file(&save_path);
                _ = log.unbounded_send(format!("成功接收{receive_path:?}"));
            } else {
                create_dir_all(save_path.join(receive_path)).unwrap();
            }
        }
    }

    fn receive_file_type(&mut self) -> Option<bool> {
        let mut type_buf = [0];

        match self.reader.read(&mut type_buf) {
            Ok(0) => None,
            Ok(1) => Some(type_buf[0] == TYPE_FILE),
            Ok(n) => {
                panic!("接收文件类型失败：读取了{}字节，预期1字节", n);
            }
            Err(e) => {
                panic!("接收文件类型失败: {}", e);
            }
        }
    }

    fn receive_file_path(&mut self) -> PathBuf {
        let mut len_buf = [0; 2];
        self.reader.read_exact(&mut len_buf).unwrap();
        let len = u16::from_be_bytes(len_buf);

        let mut path_buf = vec![0; len as usize];
        self.reader.read_exact(&mut path_buf).unwrap();
        let path = String::from_utf8_lossy(&path_buf).into_owned();

        PathBuf::from(path)
    }

    fn receive_file(&mut self, save_path: &Path) {
        let mut size_buf = [0; 8];
        self.reader.read_exact(&mut size_buf).unwrap();

        let size = u64::from_be_bytes(size_buf);

        create_dir_all(save_path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(save_path).unwrap();

        let mut limited_reader = (&mut self.reader).take(size);

        // tracing::debug!("   接收文件中，文件大小：{}", size_display(size));
        std::io::copy(&mut limited_reader, &mut file).unwrap();
    }
}

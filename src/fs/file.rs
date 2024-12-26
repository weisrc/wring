use std::io;

use crate::io::{Buf, BufMut, Fd, Fixed};

use super::open;

pub struct File {
    fd: Fd,
}

impl File {
    pub async fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let fd = open(path).await?;
        Ok(File { fd })
    }

    pub async fn read<T: BufMut>(&self, buf: &mut T, offset: u64) -> io::Result<usize> {
        self.fd.read(buf, offset).await
    }

    pub async fn write<T: Buf>(&self, buf: &T, offset: u64) -> io::Result<usize> {
        self.fd.write(buf, offset).await
    }

    pub async fn read_fixed<T: BufMut>(&self, buf: &mut Fixed<T>, offset: u64) -> io::Result<usize> {
        self.fd.read_fixed(buf, offset).await
    }

    pub async fn write_fixed<T: Buf>(&self, buf: &Fixed<T>, offset: u64) -> io::Result<usize> {
        self.fd.write_fixed(buf, offset).await
    }

    pub async fn close(&mut self) -> io::Result<()> {
        self.fd.close().await
    }

    pub fn into_fd(self) -> Fd {
        self.fd
    }

    pub fn from_fd(fd: Fd) -> Self {
        File { fd }
    }
}

use std::io;

use crate::io::{Buf, BufMut, ResultWithBuf, Fd};

use super::open;

pub struct File {
    fd: Fd,
}

impl File {
    pub async fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
        let fd = open(path).await?;
        Ok(File { fd })
    }

    pub async fn read<T: BufMut>(&self, buf: T, offset: u64) -> ResultWithBuf<T> {
        self.fd.read(buf, offset).await
    }

    pub async fn write<T: Buf>(&self, buf: T, offset: u64) -> ResultWithBuf<T> {
        self.fd.write(buf, offset).await
    }

    pub async fn close(&mut self) -> io::Result<()> {
        self.fd.close().await
    }
}

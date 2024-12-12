use crate::complete;
use io_uring::opcode;
use std::io;

use super::{buf::BufMut, Buf, Fd};

pub(crate) type ResultWithBuf<T> = (io::Result<usize>, T);

impl Fd {
    pub async fn read<T: BufMut>(&self, mut buf: T, offset: u64) -> ResultWithBuf<T> {
        let entry = opcode::Read::new(self.fd(), buf.as_mut_ptr(), buf.capacity() as _)
            .offset(offset)
            .build();
        let res = complete(entry).await.map(|out| out.result() as usize);
        (res, buf)
    }

    pub async fn write<T: Buf>(&self, buf: T, offset: u64) -> ResultWithBuf<T> {
        let entry = opcode::Write::new(self.fd(), buf.as_ptr(), buf.len() as _)
            .offset(offset)
            .build();
        let res = complete(entry).await.map(|out| out.result() as usize);
        (res, buf)
    }
}

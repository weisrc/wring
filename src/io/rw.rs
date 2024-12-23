use crate::complete::complete;
use io_uring::opcode;
use std::io;

use super::{buf::BufMut, Buf, Fd};

impl Fd {
    pub async fn read<T: BufMut>(&self, buf: &mut T, offset: u64) -> io::Result<usize> {
        let entry = opcode::Read::new(self.fd(), buf.as_mut_ptr(), buf.capacity() as _)
            .offset(offset)
            .build();
        let res = complete(entry).await.map(|out| out.result() as usize)?;

        unsafe {
            buf.set_len(res);
        }

        Ok(res)
    }

    pub async fn write<T: Buf>(&self, buf: &T, offset: u64) -> io::Result<usize> {
        let entry = opcode::Write::new(self.fd(), buf.as_ptr(), buf.len() as _)
            .offset(offset)
            .build();
        complete(entry).await.map(|out| out.result() as usize)
    }
}

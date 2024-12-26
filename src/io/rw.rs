use crate::complete::complete;
use io_uring::opcode;
use std::io;

use super::{buf::BufMut, Buf, Fd, Fixed};

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

    pub async fn read_fixed<T: BufMut>(
        &self,
        fixed_buf: &mut Fixed<T>,
        offset: u64,
    ) -> io::Result<usize> {
        let buf = fixed_buf.inner.as_mut().unwrap();

        let entry = opcode::ReadFixed::new(
            self.fd(),
            buf.as_mut_ptr(),
            buf.capacity() as _,
            fixed_buf.index,
        )
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

    pub async fn write_fixed<T: Buf>(&self, fixed_buf: &Fixed<T>, offset: u64) -> io::Result<usize> {
        let buf = fixed_buf.inner.as_ref().unwrap();

        let entry =
            opcode::WriteFixed::new(self.fd(), buf.as_ptr(), buf.len() as _, fixed_buf.index)
                .offset(offset)
                .build();

        complete(entry).await.map(|out| out.result() as usize)
    }
}

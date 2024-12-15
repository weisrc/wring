use std::{
    io,
    os::fd::{FromRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use io_uring::{opcode, types};

use crate::complete::complete;

pub struct Fd {
    inner: Arc<Inner>,
}

impl Fd {
    pub fn new(fd: RawFd) -> Self {
        let inner = Inner {
            fd,
            closed: AtomicBool::new(false),
        };
        let inner = Arc::new(inner);
        Self { inner }
    }

    pub(crate) fn fd(&self) -> types::Fd {
        types::Fd(self.inner.fd)
    }

    pub async fn close(&mut self) -> io::Result<()> {
        let entry = opcode::Close::new(self.fd()).build();
        complete(entry).await?;
        self.inner.to_owned().closed.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn from_raw_fd(fd: RawFd) -> Self {
        Self::new(fd)
    }

    pub fn into_raw_fd(self) -> RawFd {
        let fd = self.inner.fd;
        std::mem::forget(self);
        fd
    }
}

struct Inner {
    fd: RawFd,
    closed: AtomicBool,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if self.closed.load(Ordering::Relaxed) {
            return;
        }

        unsafe {
            std::fs::File::from_raw_fd(self.fd);
        }
    }
}

use std::{
    io,
    os::fd::{FromRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use io_uring::{opcode, types};

use crate::complete;

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

use std::{
    cell::OnceCell,
    io::{Error, Result},
    sync::atomic::{AtomicBool, Ordering},
};

use io_uring::{cqueue, squeue, IoUring};
use parking_lot::Mutex;

static ALREADY_EXISTS: &str = "uring already exists";
static NOT_INIT: &str = "uring not initialized";
static SQ_FULL: &str = "uring squeue full";
static SUBMIT_ERR: &str = "uring submit error";
static BUF_ERR: &str = "uring buffer already registered";

pub(crate) struct Driver {
    ring: IoUring,
    submit_lock: Mutex<()>,
    buffer_registered: AtomicBool,
}

static mut CURRENT: OnceCell<Driver> = OnceCell::new();

pub(crate) trait CQEHandler {
    fn handle_cqe(entry: cqueue::Entry);
}

impl Driver {
    pub fn new(entries: u32) -> Result<Self> {
        let ring = IoUring::new(entries)?;
        Ok(Self {
            ring,
            submit_lock: Mutex::new(()),
            buffer_registered: AtomicBool::new(false),
        })
    }

    pub fn submit(&mut self, entry: &squeue::Entry) {
        let _guard = self.submit_lock.lock();
        unsafe {
            self.ring.submission().push(entry).expect(SQ_FULL);
        }
        self.ring.submit().expect(SUBMIT_ERR);
    }

    pub fn enter<H: CQEHandler>(&mut self) -> Result<()> {
        self.ring
            .submit_and_wait(1)
            .map_err(|_| Error::other(SUBMIT_ERR))?;

        while let Some(cqe) = self.ring.completion().next() {
            H::handle_cqe(cqe);
        }

        Ok(())
    }

    pub fn into_current(self) {
        unsafe {
            CURRENT
                .set(self)
                .map_err(|_| Error::other(ALREADY_EXISTS))
                .unwrap();
        }
    }

    pub fn take_current() -> Self {
        unsafe { CURRENT.take().expect(NOT_INIT) }
    }

    pub fn current() -> &'static mut Self {
        unsafe { CURRENT.get_mut().expect(NOT_INIT) }
    }

    pub fn register_buffers(&self, iovecs: &[libc::iovec]) -> Result<()> {
        if self.buffer_registered.load(Ordering::Relaxed) {
            return Err(Error::other(BUF_ERR));
        }
        self.buffer_registered.store(true, Ordering::Relaxed);
        unsafe { self.ring.submitter().register_buffers(iovecs) }
    }

    pub fn unregister_buffers(&self) -> Result<()> {
        self.buffer_registered.store(false, Ordering::Relaxed);
        self.ring.submitter().unregister_buffers()
    }
}

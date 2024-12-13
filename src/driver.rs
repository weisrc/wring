use std::{
    cell::OnceCell,
    io::{Error, Result},
    sync::Mutex,
};

use io_uring::{cqueue, squeue, IoUring};

static mut RING: OnceCell<IoUring> = OnceCell::new();
static mut MUTEX: Mutex<()> = Mutex::new(());

static ALREADY_EXISTS: &str = "uring already exists";
static NOT_INIT: &str = "uring not initialized";
static SQ_FULL: &str = "uring squeue full";
static SUBMIT_ERR: &str = "uring submit error";

pub(crate) fn submit(entry: &squeue::Entry) {
    unsafe {
        let ring = RING.get_mut().expect(NOT_INIT);
        let _guard = MUTEX.lock().unwrap();
        ring.submission().push(entry).expect(SQ_FULL);
        ring.submit().expect(SUBMIT_ERR);
    }
}

pub(crate) fn init(entries: u32) -> Result<()> {
    let ring = IoUring::new(entries)?;
    let out = unsafe { RING.set(ring) };
    out.map_err(|_| Error::other(ALREADY_EXISTS))?;
    Ok(())
}

pub(crate) fn exit() {
    unsafe {
        let _ = RING.take();
    }
}

pub(crate) fn get_ring() -> Result<&'static mut IoUring> {
    unsafe { RING.get_mut().ok_or(Error::other(NOT_INIT)) }
}

pub(crate) trait CQEHandler {
    fn handle_cqe(entry: cqueue::Entry);
}

pub(crate) fn enter<H: CQEHandler>(ring: &mut IoUring) -> Result<()> {
    ring.submit_and_wait(1)
        .map_err(|_| Error::other(SUBMIT_ERR))?;

    while let Some(cqe) = ring.completion().next() {
        H::handle_cqe(cqe);
    }

    Ok(())
}

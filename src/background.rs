use std::{
    io::Result,
    sync::atomic::{AtomicBool, Ordering},
    thread::{self, JoinHandle},
};

use io_uring::opcode;

use crate::{
    complete::CompleteHandler,
    driver::{enter, exit, get_ring, init, submit},
};

static SHOULD_RUN: AtomicBool = AtomicBool::new(false);

pub struct BackgroundDropGuard {
    handle: Option<JoinHandle<()>>,
}

pub fn background(entries: u32) -> Result<BackgroundDropGuard> {
    init(entries)?;

    SHOULD_RUN.store(true, Ordering::Relaxed);
    let handle: JoinHandle<()> = thread::spawn(|| {
        let ring = get_ring().unwrap();
        while SHOULD_RUN.load(Ordering::Relaxed) {
            enter::<CompleteHandler>(ring).unwrap();
        }
    });

    let handle = Some(handle);
    let out = BackgroundDropGuard { handle };
    Ok(out)
}

impl Drop for BackgroundDropGuard {
    fn drop(&mut self) {
        SHOULD_RUN.store(false, Ordering::Relaxed);
        let nop = opcode::Nop::new().build();
        submit(&nop);
        self.handle.take().unwrap().join().unwrap();
        exit();
    }
}

use std::{
    io::Result,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle},
};

use io_uring::opcode;

use crate::{complete::CompleteHandler, driver::Driver};

pub struct Background {
    handle: Option<JoinHandle<()>>,
    should_run: Arc<AtomicBool>,
}

impl Background {
    pub fn new(entries: u32) -> Result<Background> {
        let driver = Driver::new(entries)?;
        driver.into_current();

        let should_run = AtomicBool::new(true);
        let should_run = Arc::new(should_run);
        let mut out = Background {
            handle: None,
            should_run: should_run.clone(),
        };

        out.should_run.store(true, Ordering::Relaxed);
        let handle: JoinHandle<()> = thread::spawn(move || {
            let driver = Driver::current();
            while should_run.load(Ordering::Relaxed) {
                driver.enter::<CompleteHandler>().unwrap();
            }
        });

        out.handle = Some(handle);
        Ok(out)
    }
}

impl Drop for Background {
    fn drop(&mut self) {
        self.should_run.store(false, Ordering::Relaxed);
        let nop = opcode::Nop::new().build();
        let driver = Driver::current();
        driver.submit(&nop);
        self.handle.take().unwrap().join().unwrap();
        let _ = Driver::take_current();
    }
}

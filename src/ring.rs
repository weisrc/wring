use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
};

use io_uring::{cqueue, squeue, IoUring};

static mut RING: Option<IoUring> = None;
static mut MUTEX: Mutex<()> = Mutex::new(());

struct WakerData {
    waker: Waker,
    data: Option<cqueue::Entry>,
}

struct Completion {
    entry: squeue::Entry,
    waker_data: Option<Arc<Mutex<WakerData>>>,
}

impl Future for Completion {
    type Output = io::Result<cqueue::Entry>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(waker_data) = &self.waker_data {
            let mut waker_data = waker_data.lock().unwrap();
            if let Some(data) = waker_data.data.take() {
                if data.result() < 0 {
                    return Poll::Ready(Err(io::Error::from_raw_os_error(-data.result())));
                }
                return Poll::Ready(Ok(data));
            }

            if !waker_data.waker.will_wake(cx.waker()) {
                waker_data.waker = cx.waker().clone();
            }
        } else {
            let waker_data = WakerData {
                waker: cx.waker().clone(),
                data: None,
            };
            let waker_data = Mutex::new(waker_data);
            let waker_data = Arc::new(waker_data);
            self.waker_data = Some(waker_data.clone());

            let ptr = Arc::into_raw(waker_data) as u64;
            let entry = self.entry.clone().user_data(ptr);

            unsafe {
                if let Err(err) = submit(&entry) {
                    return Poll::Ready(Err(err));
                }
            }
        }

        Poll::Pending
    }
}

unsafe fn submit(entry: &squeue::Entry) -> io::Result<()> {
    let ring = RING
        .as_mut()
        .ok_or(io::Error::new(io::ErrorKind::Other, "uring not started"))?;

    let _guard = MUTEX.lock().unwrap();

    ring.submission()
        .push(entry)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "squeue full"))?;

    ring.submit()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "submit error"))?;

    Ok(())
}

pub async fn complete(entry: squeue::Entry) -> io::Result<cqueue::Entry> {
    let completion = Completion {
        entry,
        waker_data: None,
    };

    completion.await
}

pub fn start(entries: u32) {

    if unsafe { RING.is_some() } {
        panic!("uring already started");
    }

    let ring = unsafe {
        RING = Some(IoUring::new(entries).expect(""));
        RING.as_mut().unwrap()
    };

    thread::spawn(|| loop {
        ring.submit_and_wait(1).unwrap();

        while let Some(cqe) = ring.completion().next() {
            let ptr = cqe.user_data() as *mut Mutex<WakerData>;
            let waker_data = unsafe { Arc::from_raw(ptr) };
            let mut waker_data = waker_data.lock().unwrap();
            waker_data.data = Some(cqe);
            waker_data.waker.wake_by_ref();
        }
    });
}

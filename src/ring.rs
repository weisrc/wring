use std::{
    cell::OnceCell,
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread::{self, JoinHandle},
};

use io_uring::{cqueue, squeue, IoUring};

static mut RING: OnceCell<IoUring> = OnceCell::new();
static mut MUTEX: Mutex<()> = Mutex::new(());

static ALREADY_STARTED: &str = "uring already started";
static NO_URING: &str = "no uring";
static SQ_FULL: &str = "uring squeue full";
static SUBMIT_ERR: &str = "uring submit error";

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
                submit(&entry);
            }
        }

        Poll::Pending
    }
}

unsafe fn submit(entry: &squeue::Entry) {
    let ring = RING.get_mut().expect(NO_URING);
    let _guard = MUTEX.lock().unwrap();
    ring.submission().push(entry).expect(SQ_FULL);
    ring.submit().expect(SUBMIT_ERR);
}

pub async fn complete(entry: squeue::Entry) -> io::Result<cqueue::Entry> {
    let completion = Completion {
        entry,
        waker_data: None,
    };

    completion.await
}

pub fn init(entries: u32) -> io::Result<()> {
    let ring = IoUring::new(entries)?;
    let out = unsafe { RING.set(ring) };
    out.map_err(|_| io::Error::other(ALREADY_STARTED))?;
    Ok(())
}

pub fn start(entries: u32) -> io::Result<JoinHandle<()>> {
    init(entries)?;

    let handle = thread::spawn(|| loop {
        enter().unwrap();
    });

    Ok(handle)
}

pub fn enter() -> io::Result<()> {
    let ring = unsafe { RING.get_mut() }.ok_or(io::Error::other(NO_URING))?;

    ring.submit_and_wait(1)
        .map_err(|_| io::Error::other(SUBMIT_ERR))?;

    while let Some(cqe) = ring.completion().next() {
        let ptr = cqe.user_data() as *mut Mutex<WakerData>;
        let waker_data = unsafe { Arc::from_raw(ptr) };
        let mut waker_data = waker_data.lock().unwrap();
        waker_data.data = Some(cqe);
        waker_data.waker.wake_by_ref();
    }

    Ok(())
}

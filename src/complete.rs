use std::{
    future::Future,
    io::{Error, Result},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use io_uring::{cqueue, squeue};

use crate::driver::{Driver, CQEHandler};

struct WakerData {
    waker: Waker,
    data: Option<cqueue::Entry>,
}

struct Complete {
    entry: squeue::Entry,
    waker_data: Option<Arc<Mutex<WakerData>>>,
}

impl Future for Complete {
    type Output = Result<cqueue::Entry>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(waker_data) = &self.waker_data {
            let mut waker_data = waker_data.lock().unwrap();
            if let Some(data) = waker_data.data.take() {
                if data.result() < 0 {
                    return Poll::Ready(Err(Error::from_raw_os_error(-data.result())));
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

            Driver::current().submit(&entry);
        }

        Poll::Pending
    }
}

pub(crate) async fn complete(entry: squeue::Entry) -> Result<cqueue::Entry> {
    let completion = Complete {
        entry,
        waker_data: None,
    };

    completion.await
}

pub(crate) struct CompleteHandler;
impl CQEHandler for CompleteHandler {
    fn handle_cqe(entry: cqueue::Entry) {
        let ptr = entry.user_data();

        if ptr == 0 {
            return;
        }

        let ptr = ptr as *mut Mutex<WakerData>;
        let waker_data = unsafe { Arc::from_raw(ptr) };
        let mut waker_data = waker_data.lock().unwrap();
        waker_data.data = Some(entry);
        waker_data.waker.wake_by_ref();
    }
}

use std::sync::Arc;

use crossbeam::queue::ArrayQueue;
use libc::iovec;

use crate::driver::Driver;

use super::{Buf, BufMut, Fixed};

pub struct FixedPool<T: Buf> {
    pub(crate) queue: ArrayQueue<Fixed<T>>,
}

impl<T: BufMut> FixedPool<T> {
    pub fn new(bufs: impl Iterator<Item = T>) -> Arc<Self> {
        let take_n = std::cmp::min(libc::UIO_MAXIOV as usize, u16::MAX as usize);
        let mut bufs: Vec<T> = bufs.take(take_n).collect();

        let iovecs: Vec<libc::iovec> = bufs
            .iter_mut()
            .map(|buf| iovec {
                iov_base: buf.as_mut_ptr() as *mut _,
                iov_len: buf.capacity(),
            })
            .collect();

        let queue: ArrayQueue<Fixed<T>> = ArrayQueue::new(bufs.len());

        let pool = Self { queue };

        let pool = Arc::new(pool);

        for (i, buf) in bufs.into_iter().enumerate() {
            let _ = pool.queue.push(Fixed {
                inner: Some(buf),
                index: i as u16,
                pool: pool.clone(),
            });
        }

        Driver::current().register_buffers(&iovecs).unwrap();

        pool
    }

    pub fn get(&self) -> Option<Fixed<T>> {
        self.queue.pop()
    }
}

impl<T: Buf> Drop for FixedPool<T> {
    fn drop(&mut self) {
        Driver::current().unregister_buffers().unwrap();
    }
}

use std::{ops::Deref, sync::Arc};

use super::{Buf, FixedPool};
pub struct Fixed<T: Buf> {
    pub(crate) index: u16,
    pub(crate) inner: Option<T>,
    pub(crate) pool: Arc<FixedPool<T>>,
}

impl<T: Buf> Drop for Fixed<T> {
    fn drop(&mut self) {
        let _ = self.pool.queue.push(Fixed {
            inner: self.inner.take(),
            index: self.index,
            pool: self.pool.clone(),
        });
    }
}

impl<T: Buf> Deref for Fixed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

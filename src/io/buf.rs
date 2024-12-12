pub trait Buf {
    fn as_ptr(&self) -> *const u8;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
}

pub unsafe trait BufMut: Buf {
    fn as_mut_ptr(&mut self) -> *mut u8;
    unsafe fn set_len(&mut self, len: usize);
}

impl Buf for Vec<u8> {
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

unsafe impl BufMut for Vec<u8> {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }

    unsafe fn set_len(&mut self, len: usize) {
        self.set_len(len)
    }
}

impl Buf for [u8] {
    fn as_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn capacity(&self) -> usize {
        self.len()
    }
}

#![no_std] 

// copy implementation from the answer of:
// https://stackoverflow.com/questions/50200268/how-can-i-use-the-format-macro-in-a-no-std-environment

use core::fmt;

pub struct ArrayWriter<const CAP: usize> {
    buf: [u8; CAP],
    cursor: usize,
}

impl<const CAP: usize> ArrayWriter<CAP> {
    #[inline]
    pub fn new() -> Self {
        Self { buf: [0u8; CAP], cursor: 0 }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn clear(&mut self){
        self.cursor = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub fn empty(&self) -> bool {
        self.cursor == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.cursor]
    }
}

impl<const CAP: usize> fmt::Write for ArrayWriter<CAP> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cap = self.capacity();
        for (i, &b) in self.buf[self.cursor..cap].iter_mut()
            .zip(s.as_bytes().iter())
        {
            *i = b;
        }
        self.cursor = usize::min(cap, self.cursor + s.as_bytes().len());
        Ok(())
    }
}